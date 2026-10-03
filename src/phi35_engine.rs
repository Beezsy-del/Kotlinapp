use crate::models::{AnalysisResult, Language, RiskLevel};
use candle_core::quantized::gguf_file;
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_llama::ModelWeights as QLlama;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Mutex;
use tokenizers::Tokenizer;

/// Model architecture flavor for on-device inference
#[derive(Debug, Clone, PartialEq)]
pub enum ModelArch {
    /// SmolLM2 360M (360M parameters, ~258MB, higher legal reasoning capability & fast on-device)
    SmolLM2_360M,
    /// Ultra-lightweight SmolLM2 (135M parameters, ~100MB, ultra-fast & memory-efficient)
    SmolLM2_135M,
    /// Microsoft Phi-3.5-mini (3.8B parameters, ~2.2GB)
    Phi35Mini,
}

/// Configuration parameters for on-device LLM inference
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub arch: ModelArch,
    pub model_path: PathBuf,
    pub tokenizer_path: PathBuf,
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub system_prompt: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        let home_dir = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        let models_dir = home_dir.join(".verdictedge").join("models");

        // Prefer SmolLM2-360M (258MB) for enhanced legal reasoning, falling back to 135M or Phi-3.5
        let smollm360_path = models_dir.join("SmolLM2-360M-Instruct-Q4_K_M.gguf");
        let smollm135_path = models_dir.join("SmolLM2-135M-Instruct-Q4_K_M.gguf");
        let phi35_path = models_dir.join("Phi-3.5-mini-instruct-Q4_K_M.gguf");
        let tokenizer_path = models_dir.join("tokenizer.json");

        if smollm360_path.exists() {
            Self {
                arch: ModelArch::SmolLM2_360M,
                model_path: smollm360_path,
                tokenizer_path,
                max_tokens: 180,
                temperature: 0.2, // Low temperature for factual legal reasoning
                top_p: 0.9,
                system_prompt: "You are VerdictEdge, an expert on-device legal AI assistant. Provide concise, clear, and actionable contract risk assessments.".into(),
            }
        } else if smollm135_path.exists() || !phi35_path.exists() {
            Self {
                arch: ModelArch::SmolLM2_135M,
                model_path: smollm135_path,
                tokenizer_path,
                max_tokens: 128,
                temperature: 0.2,
                top_p: 0.9,
                system_prompt: "You are VerdictEdge, an expert on-device legal AI assistant. Provide concise, clear, and actionable contract risk assessments.".into(),
            }
        } else {
            Self {
                arch: ModelArch::Phi35Mini,
                model_path: phi35_path,
                tokenizer_path,
                max_tokens: 256,
                temperature: 0.2,
                top_p: 0.9,
                system_prompt: "You are VerdictEdge, an expert on-device legal counsel assistant specializing in contract risk analysis and Indian statutory compliance.".into(),
            }
        }
    }
}

/// Lifecycle status of the local on-device neural model
#[derive(Debug, Clone, PartialEq)]
pub enum ModelStatus {
    /// Model weights found on disk and ready for real neural tensor inference
    Ready {
        arch: ModelArch,
        path: PathBuf,
        size_bytes: u64,
    },
    /// Local model weights file not yet downloaded to the expected path
    NotLoaded {
        expected_path: PathBuf,
        download_url: &'static str,
    },
}

// Backward-compatibility alias
pub type Phi35Config = ModelConfig;
pub type Phi35Status = ModelStatus;

/// On-Device Neural Tensor Legal Intelligence Inference Engine (Candle + GGUF)
#[derive(Debug)]
pub struct Phi35Engine {
    pub config: ModelConfig,
    /// Mutex for serialized on-device tensor execution
    _lock: Mutex<()>,
}

impl Clone for Phi35Engine {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            _lock: Mutex::new(()),
        }
    }
}

impl Default for Phi35Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Phi35Engine {
    pub const SMOLLM2_360M_URL: &'static str =
        "https://huggingface.co/bartowski/SmolLM2-360M-Instruct-GGUF/resolve/main/SmolLM2-360M-Instruct-Q4_K_M.gguf";
    pub const SMOLLM2_135M_URL: &'static str =
        "https://huggingface.co/bartowski/SmolLM2-135M-Instruct-GGUF/resolve/main/SmolLM2-135M-Instruct-Q4_K_M.gguf";
    pub const TOKENIZER_URL: &'static str =
        "https://huggingface.co/HuggingFaceTB/SmolLM2-135M-Instruct/resolve/main/tokenizer.json";
    pub const PHI35_URL: &'static str =
        "https://huggingface.co/bartowski/Phi-3.5-mini-instruct-GGUF/resolve/main/Phi-3.5-mini-instruct-Q4_K_M.gguf";

    pub fn new() -> Self {
        Self {
            config: ModelConfig::default(),
            _lock: Mutex::new(()),
        }
    }

    pub fn with_config(config: ModelConfig) -> Self {
        Self {
            config,
            _lock: Mutex::new(()),
        }
    }

    /// Check if local weights are present
    pub fn status(&self) -> ModelStatus {
        if self.config.model_path.exists() {
            let metadata = std::fs::metadata(&self.config.model_path);
            let size_bytes = metadata.map(|m| m.len()).unwrap_or(0);
            ModelStatus::Ready {
                arch: self.config.arch.clone(),
                path: self.config.model_path.clone(),
                size_bytes,
            }
        } else {
            ModelStatus::NotLoaded {
                expected_path: self.config.model_path.clone(),
                download_url: match self.config.arch {
                    ModelArch::SmolLM2_360M => Self::SMOLLM2_360M_URL,
                    ModelArch::SmolLM2_135M => Self::SMOLLM2_135M_URL,
                    ModelArch::Phi35Mini => Self::PHI35_URL,
                },
            }
        }
    }

    /// Formats prompt according to model architecture chat template
    pub fn format_instruct_prompt(&self, user_prompt: &str) -> String {
        match self.config.arch {
            ModelArch::SmolLM2_360M | ModelArch::SmolLM2_135M => {
                // ChatML format for SmolLM2: <|im_start|>system\n...<|im_end|>\n<|im_start|>user\n...<|im_end|>\n<|im_start|>assistant\n
                format!(
                    "<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
                    self.config.system_prompt.trim(),
                    user_prompt.trim()
                )
            }
            ModelArch::Phi35Mini => {
                // Microsoft Phi-3.5 Instruct format: <|system|>\n...<|end|>\n<|user|>\n...<|end|>\n<|assistant|>\n
                format!(
                    "<|system|>\n{}<|end|>\n<|user|>\n{}<|end|>\n<|assistant|>\n",
                    self.config.system_prompt.trim(),
                    user_prompt.trim()
                )
            }
        }
    }

    /// Execute REAL neural network tensor inference on-device using Candle
    pub fn generate_neural_tokens(
        &self,
        prompt: &str,
        max_new_tokens: usize,
    ) -> Result<String, String> {
        let _guard = self
            ._lock
            .lock()
            .map_err(|e| format!("Mutex acquisition error: {}", e))?;

        if !self.config.model_path.exists() {
            return Err(format!(
                "Model file not found at {:?}",
                self.config.model_path
            ));
        }
        if !self.config.tokenizer_path.exists() {
            return Err(format!(
                "Tokenizer file not found at {:?}",
                self.config.tokenizer_path
            ));
        }

        // 1. Load Tokenizer
        let tokenizer = Tokenizer::from_file(&self.config.tokenizer_path)
            .map_err(|e| format!("Failed to load tokenizer: {}", e))?;

        // 2. Load GGUF Quantized Model Weights
        let mut file = File::open(&self.config.model_path)
            .map_err(|e| format!("Failed to open model file: {}", e))?;
        let content = gguf_file::Content::read(&mut file)
            .map_err(|e| format!("Failed to parse GGUF content: {}", e))?;

        let device = Device::Cpu;
        let mut model = QLlama::from_gguf(content, &mut file, &device)
            .map_err(|e| format!("Failed to load Llama weights into Candle: {}", e))?;

        // 3. Tokenize input prompt
        let formatted = self.format_instruct_prompt(prompt);
        let tokens = tokenizer
            .encode(formatted.as_str(), true)
            .map_err(|e| format!("Tokenization failed: {}", e))?;
        let prompt_tokens = tokens.get_ids();

        if prompt_tokens.is_empty() {
            return Err("Empty token sequence from prompt".into());
        }

        let mut logits_processor = LogitsProcessor::new(
            42,
            Some(self.config.temperature as f64),
            Some(self.config.top_p as f64),
        );
        let mut generated_tokens = Vec::new();

        // 4. Initial prefill forward pass
        let input = Tensor::new(prompt_tokens, &device)
            .map_err(|e| e.to_string())?
            .unsqueeze(0)
            .map_err(|e| e.to_string())?;

        let logits = model
            .forward(&input, 0)
            .map_err(|e| format!("Forward pass error: {}", e))?;
        let logits = logits.squeeze(0).map_err(|e| e.to_string())?;
        let mut next_token = logits_processor
            .sample(&logits)
            .map_err(|e| format!("Sampling error: {}", e))?;
        generated_tokens.push(next_token);

        // 5. Autoregressive token generation loop
        for pos in prompt_tokens.len()..(prompt_tokens.len() + max_new_tokens) {
            let input = Tensor::new(&[next_token], &device)
                .map_err(|e| e.to_string())?
                .unsqueeze(0)
                .map_err(|e| e.to_string())?;

            let logits = model
                .forward(&input, pos)
                .map_err(|e| format!("Autoregressive step error at pos {}: {}", pos, e))?;
            let logits = logits.squeeze(0).map_err(|e| e.to_string())?;
            next_token = logits_processor
                .sample(&logits)
                .map_err(|e| format!("Sampling error: {}", e))?;

            // Check EOS stop conditions
            if let Some(t) = tokenizer.id_to_token(next_token) {
                if t == "<|im_end|>"
                    || t == "<|endoftext|>"
                    || t == "<|end|>"
                    || t == "</s>"
                {
                    break;
                }
            }
            generated_tokens.push(next_token);
        }

        // 6. Detokenize output tokens to string
        let raw_text = tokenizer
            .decode(&generated_tokens, true)
            .map_err(|e| format!("Detokenization error: {}", e))?;

        let clean_text = raw_text
            .replace("<|im_end|>", "")
            .replace("<|endoftext|>", "")
            .replace("<|end|>", "")
            .trim()
            .to_string();

        Ok(clean_text)
    }

    /// Perform on-device legal reasoning and risk synthesis using real neural LLM when available
    pub fn analyze(
        &self,
        contract_text: &str,
        rule_findings: &AnalysisResult,
        language: Language,
    ) -> String {
        // Construct targeted prompt
        let prompt = self.construct_analysis_prompt(contract_text, rule_findings, language);

        // Check if neural weights and tokenizer exist on disk
        if self.config.model_path.exists() && self.config.tokenizer_path.exists() {
            match self.generate_neural_tokens(&prompt, self.config.max_tokens) {
                Ok(neural_output) if !neural_output.trim().is_empty() => {
                    let mut result = String::new();
                    let arch_name = match self.config.arch {
                        ModelArch::SmolLM2_360M => "SmolLM2-360M",
                        ModelArch::SmolLM2_135M => "SmolLM2-135M",
                        ModelArch::Phi35Mini => "Phi-3.5-mini",
                    };
                    result.push_str(&format!("### 🧠 On-Device Neural LLM Analysis ({} / Candle)\n\n", arch_name));
                    result.push_str(&neural_output);
                    result.push_str("\n\n---\n");
                    result.push_str(&self.generate_legal_insight(rule_findings, language));
                    return result;
                }
                Err(err) => {
                    eprintln!("Candle neural execution notice: {}. Using statutory synthesis.", err);
                }
                _ => {}
            }
        }

        // High-fidelity statutory synthesis fallback if model file is pending download
        self.generate_legal_insight(rule_findings, language)
    }

    /// Generate tailored counter-clause draft for a specific disputed clause
    pub fn generate_counter_clause(
        &self,
        original_clause: &str,
        legal_issue: &str,
    ) -> String {
        let prompt = format!(
            "Clause: \"{}\"\nProblem: {}\nProvide a 1-sentence balanced counter-clause:",
            original_clause, legal_issue
        );

        if self.config.model_path.exists() && self.config.tokenizer_path.exists() {
            if let Ok(tokens) = self.generate_neural_tokens(&prompt, 64) {
                if !tokens.trim().is_empty() {
                    return format!("Proposed Neural Counter-Clause: '{}'", tokens.trim());
                }
            }
        }

        format!(
            "Proposed Counter-Clause: 'Notwithstanding anything to the contrary herein, the obligations under this provision shall be mutual, reasonable, and capped in scope. Neither party shall be liable for indirect or consequential damages, and any restriction shall require prior written notice with an opportunity to cure.'"
        )
    }

    /// Builds structured prompt combining raw text with discovered statutory issues
    fn construct_analysis_prompt(
        &self,
        contract_text: &str,
        rule_findings: &AnalysisResult,
        _language: Language,
    ) -> String {
        let mut prompt = String::new();
        prompt.push_str("Summarize the legal risks and suggest 1 key negotiation point for this contract:\n\n");
        prompt.push_str("--- CONTRACT TEXT ---\n");

        let truncated = if contract_text.len() > 1500 {
            format!("{}...\n[Truncated]", &contract_text[..1500])
        } else {
            contract_text.to_string()
        };
        prompt.push_str(&truncated);
        prompt.push_str("\n--- END ---\n");

        prompt.push_str(&format!("Identified Risk: {:?}\n", rule_findings.risk_level));
        for sv in &rule_findings.statutory_voidabilities {
            prompt.push_str(&format!("Statutory Issue: {}\n", sv.act_section));
        }

        prompt
    }

    /// Deterministic statutory fallback synthesis
    fn generate_legal_insight(
        &self,
        findings: &AnalysisResult,
        language: Language,
    ) -> String {
        let mut output = String::new();

        match language {
            Language::English => {
                output.push_str("### ⚖️ Statutory Legal Analysis & Counsel Briefing\n\n");
                output.push_str("**1. Executive Risk Synthesis:**\n");
                match findings.risk_level {
                    RiskLevel::High => {
                        output.push_str("This document presents **critical legal exposure**. Several unilateral covenants fail the standard of bilateral reciprocity, and specific terms appear legally unenforceable under statutory provisions of the Indian Contract Act, 1872.\n\n");
                    }
                    RiskLevel::Medium => {
                        output.push_str("This document exhibits **moderate operational risk**. While standard operational mechanisms exist, strict payment penalties, tight notice periods, and broad discretionary language require targeted adjustments.\n\n");
                    }
                    RiskLevel::Low => {
                        output.push_str("This agreement appears **balanced and standard**. Obligations are reciprocal, and no aggressive statutory or unilateral liabilities were identified.\n\n");
                    }
                    RiskLevel::Invalid => {
                        output.push_str("The scanned document does not exhibit enforceable legal agreement structure.\n\n");
                        return output;
                    }
                }

                if !findings.statutory_voidabilities.is_empty() {
                    output.push_str("**2. Statutory Non-Enforceability Warnings:**\n");
                    for sv in &findings.statutory_voidabilities {
                        output.push_str(&format!(
                            "* **{}**: {}\n  *Statutory Basis:* {}\n",
                            sv.act_section, sv.title_en, sv.legal_reason_en
                        ));
                    }
                    output.push_str("\n");
                }

                if !findings.clause_breakdowns.is_empty() {
                    output.push_str("**3. Strategic Negotiation Leverage:**\n");
                    for (i, cb) in findings.clause_breakdowns.iter().take(3).enumerate() {
                        output.push_str(&format!(
                            "{}. **Focus Area:** {}\n   *Recommended Solution:* {}\n",
                            i + 1, cb.problem_en, cb.solution_en
                        ));
                    }
                    output.push_str("\n");
                }

                output.push_str("**4. Final Counsel Recommendation:**\n");
                output.push_str("Do not execute in its current unilateral form. Present the proposed counter-offer clauses to the counterparty before signing.");
            }
            Language::Hindi => {
                output.push_str("### ⚖️ वैधानिक कानूनी विश्लेषण और ब्रीफिंग\n\n");
                output.push_str("**1. कार्यकारी जोखिम सारांश:**\n");
                match findings.risk_level {
                    RiskLevel::High => {
                        output.push_str("यह दस्तावेज़ **गंभीर कानूनी जोखिम** दर्शाता है। कई एकतरफा शर्तें भारतीय अनुबंध अधिनियम, 1872 के तहत कानूनी रूप से अप्रवर्तनीय हैं।\n\n");
                    }
                    RiskLevel::Medium => {
                        output.push_str("यह दस्तावेज़ **मध्यम जोखिम** दर्शाता है। भुगतान दंड और विवेकाधीन शर्तों में बातचीत की आवश्यकता है।\n\n");
                    }
                    RiskLevel::Low => {
                        output.push_str("यह समझौता **संतुलित और मानक** प्रतीत होता है।\n\n");
                    }
                    RiskLevel::Invalid => {
                        output.push_str("स्कैन किया गया दस्तावेज़ वैध अनुबंध नहीं है।\n\n");
                        return output;
                    }
                }

                if !findings.statutory_voidabilities.is_empty() {
                    output.push_str("**2. वैधानिक अमान्यता चेतावनी:**\n");
                    for sv in &findings.statutory_voidabilities {
                        output.push_str(&format!(
                            "* **{}**: {}\n  *कारण:* {}\n",
                            sv.act_section, sv.title_hi, sv.legal_reason_hi
                        ));
                    }
                    output.push_str("\n");
                }
            }
            Language::Kannada => {
                output.push_str("### ⚖️ ಶಾಸನಬದ್ಧ ಕಾನೂನು ವಿಶ್ಲೇಷಣೆ\n\n");
                output.push_str("**1. ಪ್ರಮುಖ ಅಪಾಯದ ಸಾರಾಂಶ:**\n");
                match findings.risk_level {
                    RiskLevel::High => {
                        output.push_str("ಈ ದಾಖಲೆಯು **ಗಂಭೀರ ಕಾನೂನು ಅಪಾಯವನ್ನು** ಒಳಗೊಂಡಿದೆ. ಭಾರತೀಯ ಒಪ್ಪಂದ ಕಾಯಿದೆ 1872 ರ ಪ್ರಕಾರ ಕೆಲವು ಶರತ್ತುಗಳು ಅಮಾನ್ಯವಾಗಿವೆ.\n\n");
                    }
                    RiskLevel::Medium => {
                        output.push_str("ಈ ದಾಖಲೆಯು **ಮಧ್ಯಮ ಅಪಾಯವನ್ನು** ಹೊಂದಿದೆ. ಶುಲ್ಕ ಮತ್ತು ನೋಟಿಸ್ ಅವಧಿಯನ್ನು ಪರಿಶೀಲಿಸಿ.\n\n");
                    }
                    RiskLevel::Low => {
                        output.push_str("ಈ ಒಪ್ಪಂದವು **ಸಮತೋಲಿತ ಮತ್ತು ಸಾಮಾನ್ಯವಾಗಿದೆ**.\n\n");
                    }
                    RiskLevel::Invalid => {
                        output.push_str("ಇದು ಮಾನ್ಯ ಕಾನೂನು ಒಪ್ಪಂದವಲ್ಲ.\n\n");
                        return output;
                    }
                }

                if !findings.statutory_voidabilities.is_empty() {
                    output.push_str("**2. ಶಾಸನಬದ್ಧ ಅಮಾನ್ಯತೆಯ ಎಚ್ಚರಿಕೆ:**\n");
                    for sv in &findings.statutory_voidabilities {
                        output.push_str(&format!(
                            "* **{}**: {}\n  *ಕಾರಣ:* {}\n",
                            sv.act_section, sv.title_kn, sv.legal_reason_kn
                        ));
                    }
                    output.push_str("\n");
                }
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract_engine::ContractEngine;

    #[test]
    fn test_instruct_prompt_format() {
        let engine = Phi35Engine::new();
        let prompt = engine.format_instruct_prompt("Analyze this non-compete clause");
        assert!(prompt.contains("Analyze this non-compete clause"));
    }

    #[test]
    fn test_analysis_generation() {
        let rule_engine = ContractEngine::new();
        let contract = r#"
            AGREEMENT
            Clause 1: Post-employment non-compete for 2 years.
            Clause 2: Indemnify company against all damages.
        "#;
        let findings = rule_engine.analyze_contract(contract, &[]);

        let engine = Phi35Engine::new();
        let insight_en = engine.analyze(contract, &findings, Language::English);
        assert!(insight_en.contains("Section 27, Indian Contract Act 1872") || insight_en.contains("SmolLM2"));

        let insight_hi = engine.analyze(contract, &findings, Language::Hindi);
        assert!(insight_hi.contains("कानूनी"));

        let insight_kn = engine.analyze(contract, &findings, Language::Kannada);
        assert!(insight_kn.contains("ಕಾನೂನು"));
    }

    #[test]
    fn test_counter_clause_generation() {
        let engine = Phi35Engine::new();
        let draft = engine.generate_counter_clause("Vendor shall have sole liability", "Uncapped liability");
        assert!(draft.contains("Counter-Clause"));
    }

    #[test]
    fn test_real_neural_token_generation_if_weights_present() {
        let engine = Phi35Engine::new();
        if let ModelStatus::Ready { .. } = engine.status() {
            let result = engine.generate_neural_tokens("What is a legal contract in 5 words?", 15);
            assert!(result.is_ok(), "Real neural token generation should succeed when weights are present");
            let text = result.unwrap();
            println!("Real generated tokens: {:?}", text);
            assert!(!text.trim().is_empty());
        }
    }
}
