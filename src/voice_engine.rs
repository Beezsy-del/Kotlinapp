use crate::models::{AnalysisResult, Language};
use regex::Regex;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

/// Multilingual Voice & Audio Narrator Engine
/// Provides text-to-speech normalization, pronunciation tuning, and Indian narrator cadence.
#[derive(Debug, Clone)]
pub struct VoiceEngine {
    current_process: Arc<Mutex<Option<Child>>>,
}

impl PartialEq for VoiceEngine {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.current_process, &other.current_process)
    }
}

impl Default for VoiceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceEngine {
    pub fn new() -> Self {
        Self {
            current_process: Arc::new(Mutex::new(None)),
        }
    }

    /// Prepares raw contract findings into a speech-ready, natural audio briefing
    pub fn build_audio_summary(&self, result: &AnalysisResult, language: Language) -> String {
        if result.is_invalid {
            return match language {
                Language::English => "Unreadable or non-contract document detected. Please scan an actual legal agreement.".to_string(),
                Language::Hindi => "गैर-अनुबंध या अपठनीय दस्तावेज़ पाया गया। कृपया एक वास्तविक कानूनी समझौता स्कैन करें।".to_string(),
                Language::Kannada => "ಅಪಠ್ಯ ಅಥವಾ ಒಪ್ಪಂದವಲ್ಲದ ದಾಖಲೆ ಪತ್ತೆಯಾಗಿದೆ. ದಯವಿಟ್ಟು ನಿಜವಾದ ಕಾನೂನು ಒಪ್ಪಂದವನ್ನು ಸ್ಕ್ಯಾನ್ ಮಾಡಿ.".to_string(),
            };
        }

        let mut script = String::new();

        match language {
            Language::English => {
                script.push_str(&format!(
                    "VerdictEdge Legal Briefing. Contract Risk Assessment is {}. ",
                    result.risk_level.label(Language::English)
                ));
                script.push_str(&result.summary(Language::English));
                script.push_str(". ");

                if !result.statutory_voidabilities.is_empty() {
                    script.push_str("Critical Statutory Warning: ");
                    for sv in &result.statutory_voidabilities {
                        script.push_str(&format!(
                            "Under {}, {} is detected. {}. ",
                            sv.act_section, sv.title_en, sv.legal_reason_en
                        ));
                    }
                }

                if let Some(first_breakdown) = result.clause_breakdowns.first() {
                    script.push_str(&format!(
                        "Key vulnerability detected: {}. Suggested action: {}. ",
                        first_breakdown.problem_en, first_breakdown.solution_en
                    ));
                }

                if !result.financial_exposures.is_empty() {
                    script.push_str(&format!(
                        "Payment exposure identified: {}. ",
                        result.financial_exposures[0].amount_or_cost
                    ));
                }
            }
            Language::Hindi => {
                script.push_str(&format!(
                    "वर्डिक्टएज कानूनी रिपोर्ट। अनुबंध जोखिम स्तर {} है। ",
                    result.risk_level.label(Language::Hindi)
                ));
                script.push_str(&result.summary(Language::Hindi));
                script.push_str("। ");

                if !result.statutory_voidabilities.is_empty() {
                    for sv in &result.statutory_voidabilities {
                        script.push_str(&format!(
                            "वैधानिक चेतावनी: {} के तहत {}। {}। ",
                            sv.act_section, sv.title_hi, sv.legal_reason_hi
                        ));
                    }
                }
            }
            Language::Kannada => {
                script.push_str(&format!(
                    "ವರ್ಡಿಕ್ಟ್‌ಎಡ್ಜ್ ಕಾನೂನು ವರದಿ. ಒಪ್ಪಂದದ ಅಪಾಯದ ಮಟ್ಟ {} ಆಗಿದೆ. ",
                    result.risk_level.label(Language::Kannada)
                ));
                script.push_str(&result.summary(Language::Kannada));
                script.push_str(". ");

                if !result.statutory_voidabilities.is_empty() {
                    for sv in &result.statutory_voidabilities {
                        script.push_str(&format!(
                            "ಶಾಸನಬದ್ಧ ಎಚ್ಚರಿಕೆ: {} ಪ್ರಕಾರ {} ಕಂಡುಬಂದಿದೆ. {} ",
                            sv.act_section, sv.title_kn, sv.legal_reason_kn
                        ));
                    }
                }
            }
        }

        Self::normalize_for_speech(&script, language)
    }

    /// Normalizes currency symbols, legal abbreviations, percentages, and punctuation for natural pauses
    pub fn normalize_for_speech(text: &str, language: Language) -> String {
        let mut clean = text.to_string();

        match language {
            Language::English => {
                // Currency conversion
                let inr_re = Regex::new(r"(?i)(?:₹|INR)\s*([0-9][0-9,]*)").unwrap();
                clean = inr_re.replace_all(&clean, "$1 Rupees").to_string();
                clean = clean.replace('₹', " Rupees ");
                clean = Regex::new(r"(?i)\bINR\b").unwrap().replace_all(&clean, " Rupees ").to_string();
                clean = clean.replace('$', " Dollars ");

                // Symbols & pauses
                clean = clean.replace('%', " percent ");
                clean = clean.replace(" / ", ", ");
                clean = clean.replace('•', ", ");
                clean = clean.replace('—', ", ");
                clean = clean.replace('–', ", ");
                clean = clean.replace(';', ", ");

                // Voice tone cadence mappings
                clean = clean.replace("CRITICAL RISK", "Critical Risk");
                clean = clean.replace("MODERATE RISK", "Moderate Risk");
                clean = clean.replace("LOW RISK", "Low Risk");
            }
            Language::Hindi => {
                let inr_re = Regex::new(r"(?i)(?:₹|INR)\s*([0-9][0-9,]*)").unwrap();
                clean = inr_re.replace_all(&clean, "$1 रुपये").to_string();
                clean = clean.replace('₹', " रुपये ");
                clean = Regex::new(r"(?i)\bINR\b").unwrap().replace_all(&clean, " रुपये ").to_string();
                clean = clean.replace('%', " प्रतिशत ");
                clean = clean.replace(" / ", ", ");
                clean = clean.replace('•', " ");
                clean = clean.replace('—', ", ");
                clean = clean.replace('–', ", ");
                clean = clean.replace(';', "। ");
            }
            Language::Kannada => {
                let inr_re = Regex::new(r"(?i)(?:₹|INR)\s*([0-9][0-9,]*)").unwrap();
                clean = inr_re.replace_all(&clean, "$1 ರೂಪಾಯಿ").to_string();
                clean = clean.replace('₹', " ರೂಪಾಯಿ ");
                clean = Regex::new(r"(?i)\bINR\b").unwrap().replace_all(&clean, " ರೂಪಾಯಿ ").to_string();
                clean = clean.replace('%', " ಪ್ರತಿಶತ ");
                clean = clean.replace(" / ", ", ");
                clean = clean.replace('•', " ");
                clean = clean.replace('—', ", ");
                clean = clean.replace('–', ", ");
                clean = clean.replace(';', ", ");
            }
        }

        // Collapse duplicate whitespaces
        clean.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Speaks the provided text asynchronously without blocking the caller or GUI
    pub fn speak_async(&self, text: &str, language: Language) {
        self.stop();

        let clean_text = Self::normalize_for_speech(text, language);
        let process_slot = Arc::clone(&self.current_process);

        std::thread::spawn(move || {
            let voice_name = match language {
                Language::English => "Aman",    // Indian English Accent on macOS
                Language::Hindi => "Lekha",      // Hindi (hi_IN) on macOS
                Language::Kannada => "Soumya",  // Kannada (kn_IN) on macOS
            };

            #[cfg(target_os = "macos")]
            {
                // Try targeted Indian voice first, fallback to default system voice
                let child = Command::new("say")
                    .arg("-v")
                    .arg(voice_name)
                    .arg(&clean_text)
                    .spawn()
                    .or_else(|_| {
                        Command::new("say")
                            .arg(&clean_text)
                            .spawn()
                    });

                if let Ok(child_proc) = child {
                    if let Ok(mut lock) = process_slot.lock() {
                        *lock = Some(child_proc);
                    }
                }
            }

            #[cfg(target_os = "windows")]
            {
                let powershell_cmd = format!(
                    "Add-Type -AssemblyName System.Speech; $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer; $synth.Speak('{}');",
                    clean_text.replace("'", "''")
                );
                let _ = Command::new("powershell")
                    .arg("-Command")
                    .arg(&powershell_cmd)
                    .spawn();
            }

            #[cfg(target_os = "linux")]
            {
                let _ = Command::new("spd-say")
                    .arg(&clean_text)
                    .spawn();
            }
        });
    }

    /// Stop any ongoing speech playback
    pub fn stop(&self) {
        if let Ok(mut lock) = self.current_process.lock() {
            if let Some(mut child) = lock.take() {
                let _ = child.kill();
            }
        }
    }

    /// Check if audio is currently playing
    pub fn is_playing(&self) -> bool {
        if let Ok(mut lock) = self.current_process.lock() {
            if let Some(child) = lock.as_mut() {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        *lock = None;
                        false
                    }
                    Ok(None) => true,
                    Err(_) => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ContractFamily, FinancialExposure, MoneyDateLedger, RedFlag, RiskLevel, StatutoryVoidability};

    #[test]
    fn test_text_normalization_english() {
        let raw = "Fee is ₹ 50,000 or INR 20000; late penalty is 5% per annum • CRITICAL RISK";
        let normalized = VoiceEngine::normalize_for_speech(raw, Language::English);
        assert!(normalized.contains("50,000 Rupees"));
        assert!(normalized.contains("20000 Rupees"));
        assert!(normalized.contains("5 percent"));
        assert!(normalized.contains("Critical Risk"));
        assert!(!normalized.contains('₹'));
        assert!(!normalized.contains('•'));
    }

    #[test]
    fn test_text_normalization_hindi() {
        let raw = "कुल देय राशि ₹ 25,000 है और ब्याज 10% लगेगा;";
        let normalized = VoiceEngine::normalize_for_speech(raw, Language::Hindi);
        assert!(normalized.contains("25,000 रुपये"));
        assert!(normalized.contains("10 प्रतिशत"));
        assert!(normalized.contains("।"));
    }

    #[test]
    fn test_text_normalization_kannada() {
        let raw = "ಶುಲ್ಕ ₹ 10,000 ಮತ್ತು ದಂಡ 15% ಆಗಿದೆ;";
        let normalized = VoiceEngine::normalize_for_speech(raw, Language::Kannada);
        assert!(normalized.contains("10,000 ರೂಪಾಯಿ"));
        assert!(normalized.contains("15 ಪ್ರತಿಶತ"));
    }

    #[test]
    fn test_build_audio_summary() {
        let engine = VoiceEngine::new();
        let result = AnalysisResult {
            risk_level: RiskLevel::High,
            summary_en: "Critical issues detected".into(),
            summary_hi: "गंभीर मुद्दे पाए गए".into(),
            summary_kn: "ಗಂಭೀರ ಸಮಸ್ಯೆಗಳು ಕಂಡುಬಂದಿವೆ".into(),
            red_flags: vec![RedFlag {
                title_en: "Flag".into(),
                title_hi: "फ्लैग".into(),
                title_kn: "ಫ್ಲ್ಯಾಗ್".into(),
                desc_en: "Desc".into(),
                desc_hi: "विवरण".into(),
                desc_kn: "ವಿವರಣೆ".into(),
                quote_snippet: "".into(),
            }],
            deadlines: vec![],
            financial_exposures: vec![FinancialExposure {
                title_en: "Fee".into(),
                title_hi: "शुल्क".into(),
                title_kn: "ಶುಲ್ಕ".into(),
                amount_or_cost: "INR 50,000".into(),
                description_en: "".into(),
                description_hi: "".into(),
                description_kn: "".into(),
                quote_snippet: "".into(),
            }],
            statutory_voidabilities: vec![StatutoryVoidability {
                act_section: "Section 27".into(),
                title_en: "Non-compete".into(),
                title_hi: "गैर-प्रतिस्पर्धा".into(),
                title_kn: "ಸ್ಪರ್ಧಾತ್ಮಕವಲ್ಲದ".into(),
                legal_reason_en: "Void in India".into(),
                legal_reason_hi: "अमान्य".into(),
                legal_reason_kn: "ಅಮಾನ್ಯ".into(),
                quote_snippet: "".into(),
                status: "VOID".into(),
            }],
            clause_breakdowns: vec![],
            ambiguities: vec![],
            dealbreaker_matches: vec![],
            pre_signing_checklist: vec![],
            is_invalid: false,
            phi35_insight: None,
            contract_family: ContractFamily::General,
            canonical_findings: vec![],
            relations: vec![],
            ledger: MoneyDateLedger::default(),
            missing_clauses: vec![],
            compiled_policies: vec![],
            policy_violations: vec![],
        };

        let summary_en = engine.build_audio_summary(&result, Language::English);
        assert!(summary_en.contains("VerdictEdge Legal Briefing"));
        assert!(summary_en.contains("Section 27"));
        assert!(summary_en.contains("50,000 Rupees"));

        let summary_hi = engine.build_audio_summary(&result, Language::Hindi);
        assert!(summary_hi.contains("वर्डिक्टएज कानूनी रिपोर्ट"));

        let summary_kn = engine.build_audio_summary(&result, Language::Kannada);
        assert!(summary_kn.contains("ವರ್ಡಿಕ್ಟ್‌ಎಡ್ಜ್ ಕಾನೂನು ವರದಿ"));
    }
}
