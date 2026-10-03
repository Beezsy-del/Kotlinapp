use serde::{Deserialize, Serialize};

/// Supported interface and audio summary languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    English,
    Hindi,
    Kannada,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Hindi => "hi",
            Language::Kannada => "kn",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Hindi => "हिंदी (Hindi)",
            Language::Kannada => "ಕನ್ನಡ (Kannada)",
        }
    }
}

/// Overall risk classification of the analyzed contract
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
    Invalid,
}

impl RiskLevel {
    pub fn label(&self, lang: Language) -> &'static str {
        match (self, lang) {
            (RiskLevel::Low, Language::English) => "LOW RISK",
            (RiskLevel::Low, Language::Hindi) => "कम जोखिम",
            (RiskLevel::Low, Language::Kannada) => "ಕಡಿಮೆ ಅಪಾಯ",

            (RiskLevel::Medium, Language::English) => "MODERATE RISK",
            (RiskLevel::Medium, Language::Hindi) => "मध्यम जोखिम",
            (RiskLevel::Medium, Language::Kannada) => "ಮಧ್ಯಮ ಅಪಾಯ",

            (RiskLevel::High, Language::English) => "CRITICAL RISK",
            (RiskLevel::High, Language::Hindi) => "गंभीर जोखिम",
            (RiskLevel::High, Language::Kannada) => "ಹೆಚ್ಚಿನ ಅಪಾಯ",

            (RiskLevel::Invalid, Language::English) => "NON-CONTRACT / UNREADABLE",
            (RiskLevel::Invalid, Language::Hindi) => "गैर-अनुबंध / अपठनीय",
            (RiskLevel::Invalid, Language::Kannada) => "ಒಪ್ಪಂದವಲ್ಲದ / ಅಪಠ್ಯ",
        }
    }

    pub fn description(&self, lang: Language) -> &'static str {
        match (self, lang) {
            (RiskLevel::Low, Language::English) => {
                "Standard balanced agreement with minimal legal exposure."
            }
            (RiskLevel::Low, Language::Hindi) => {
                "न्यूनतम कानूनी जोखिम के साथ मानक संतुलित समझौता।"
            }
            (RiskLevel::Low, Language::Kannada) => {
                "ಕನಿಷ್ಠ ಕಾನೂನು ಅಪಾಯದೊಂದಿಗೆ ಸಾಮಾನ್ಯ ಸಮತೋಲಿತ ಒಪ್ಪಂದ."
            }

            (RiskLevel::Medium, Language::English) => {
                "Standard operational clauses requiring negotiation or review."
            }
            (RiskLevel::Medium, Language::Hindi) => {
                "मानक परिचालन शर्तें जिनके लिए बातचीत या समीक्षा की आवश्यकता है।"
            }
            (RiskLevel::Medium, Language::Kannada) => {
                "ಚರ್ಚೆ ಅಥವಾ ಪರಿಶೀಲನೆ ಅಗತ್ಯವಿರುವ ಸಾಮಾನ್ಯ ಕಾರ್ಯಾಚರಣೆಯ ನಿಯಮಗಳು."
            }

            (RiskLevel::High, Language::English) => {
                "Critical liability exposures and unilateral terms detected."
            }
            (RiskLevel::High, Language::Hindi) => {
                "गंभीर देनदारी जोखिम और एकपक्षीय शर्तें पाई गईं।"
            }
            (RiskLevel::High, Language::Kannada) => {
                "ಗಂಭೀರ ಜವಾಬ್ದಾರಿ ಅಪಾಯಗಳು ಮತ್ತು ಏಕಪಕ್ಷೀಯ ನಿಯಮಗಳನ್ನು ಗುರುತಿಸಲಾಗಿದೆ."
            }

            (RiskLevel::Invalid, Language::English) => {
                "No legal agreement terms or valid contract text detected."
            }
            (RiskLevel::Invalid, Language::Hindi) => {
                "दस्तावेज़ में कोई कानूनी समझौता या शर्तें नहीं मिलीं।"
            }
            (RiskLevel::Invalid, Language::Kannada) => {
                "ದಾಖಲೆಯಲ್ಲಿ ಯಾವುದೇ ಕಾನೂನು ಒಪ್ಪಂದದ ನಿಯಮಗಳು ಕಂಡುಬಂದಿಲ್ಲ."
            }
        }
    }

    pub fn hex_color(&self) -> &'static str {
        match self {
            RiskLevel::High => "#EF4444",    // Red
            RiskLevel::Medium => "#F59E0B",  // Amber
            RiskLevel::Low => "#10B981",     // Emerald green
            RiskLevel::Invalid => "#6B7280", // Slate gray
        }
    }

    pub fn bg_color(&self) -> &'static str {
        match self {
            RiskLevel::High => "rgba(239, 68, 68, 0.15)",
            RiskLevel::Medium => "rgba(245, 158, 11, 0.15)",
            RiskLevel::Low => "rgba(16, 185, 129, 0.15)",
            RiskLevel::Invalid => "rgba(107, 114, 128, 0.15)",
        }
    }
}

/// Identifies general dangerous contract clauses or unilateral provisions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedFlag {
    pub title_en: String,
    pub title_hi: String,
    pub title_kn: String,
    pub desc_en: String,
    pub desc_hi: String,
    pub desc_kn: String,
    #[serde(default)]
    pub quote_snippet: String,
}

impl RedFlag {
    pub fn title(&self, lang: Language) -> &str {
        match lang {
            Language::English => &self.title_en,
            Language::Hindi => &self.title_hi,
            Language::Kannada => &self.title_kn,
        }
    }

    pub fn desc(&self, lang: Language) -> &str {
        match lang {
            Language::English => &self.desc_en,
            Language::Hindi => &self.desc_hi,
            Language::Kannada => &self.desc_kn,
        }
    }
}

/// Notice period or deadline obligations (e.g., 30 days notice)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeadlineObligation {
    pub timeframe: String,
    pub obligation_en: String,
    pub obligation_hi: String,
    pub obligation_kn: String,
    #[serde(default)]
    pub quote_snippet: String,
}

impl DeadlineObligation {
    pub fn obligation(&self, lang: Language) -> &str {
        match lang {
            Language::English => &self.obligation_en,
            Language::Hindi => &self.obligation_hi,
            Language::Kannada => &self.obligation_kn,
        }
    }
}

/// Disclosed or hidden financial caps, penalties, or damages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinancialExposure {
    pub title_en: String,
    pub title_hi: String,
    pub title_kn: String,
    pub amount_or_cost: String,
    pub description_en: String,
    pub description_hi: String,
    pub description_kn: String,
    #[serde(default)]
    pub quote_snippet: String,
}

impl FinancialExposure {
    pub fn title(&self, lang: Language) -> &str {
        match lang {
            Language::English => &self.title_en,
            Language::Hindi => &self.title_hi,
            Language::Kannada => &self.title_kn,
        }
    }

    pub fn description(&self, lang: Language) -> &str {
        match lang {
            Language::English => &self.description_en,
            Language::Hindi => &self.description_hi,
            Language::Kannada => &self.description_kn,
        }
    }
}

/// Terms void under Indian Contract Act 1872 (Section 27, 28, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatutoryVoidability {
    pub act_section: String,
    pub title_en: String,
    pub title_hi: String,
    pub title_kn: String,
    pub legal_reason_en: String,
    pub legal_reason_hi: String,
    pub legal_reason_kn: String,
    #[serde(default)]
    pub quote_snippet: String,
    #[serde(default = "default_void_status")]
    pub status: String,
}

fn default_void_status() -> String {
    "POTENTIALLY VOID".to_string()
}

impl StatutoryVoidability {
    pub fn title(&self, lang: Language) -> &str {
        match lang {
            Language::English => &self.title_en,
            Language::Hindi => &self.title_hi,
            Language::Kannada => &self.title_kn,
        }
    }

    pub fn legal_reason(&self, lang: Language) -> &str {
        match lang {
            Language::English => &self.legal_reason_en,
            Language::Hindi => &self.legal_reason_hi,
            Language::Kannada => &self.legal_reason_kn,
        }
    }
}

/// A breakdown of high-risk clauses with actionable solutions and counter-clause drafts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProblemSolutionBreakdown {
    pub original_snippet: String,
    pub problem_en: String,
    pub problem_hi: String,
    pub problem_kn: String,
    pub solution_en: String,
    pub solution_hi: String,
    pub solution_kn: String,
    pub counter_offer_draft: String,
}

impl ProblemSolutionBreakdown {
    pub fn problem(&self, lang: Language) -> &str {
        match lang {
            Language::English => &self.problem_en,
            Language::Hindi => &self.problem_hi,
            Language::Kannada => &self.problem_kn,
        }
    }

    pub fn solution(&self, lang: Language) -> &str {
        match lang {
            Language::English => &self.solution_en,
            Language::Hindi => &self.solution_hi,
            Language::Kannada => &self.solution_kn,
        }
    }
}

/// Subjective discretionary phrases that weaken legal defense
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AmbiguityTerm {
    pub phrase: String,
    pub explanation_en: String,
    pub explanation_hi: String,
    pub explanation_kn: String,
    #[serde(default)]
    pub quote_snippet: String,
}

impl AmbiguityTerm {
    pub fn explanation(&self, lang: Language) -> &str {
        match lang {
            Language::English => &self.explanation_en,
            Language::Hindi => &self.explanation_hi,
            Language::Kannada => &self.explanation_kn,
        }
    }
}

/// Match against user-defined dealbreakers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DealbreakerMatch {
    pub rule_keyword: String,
    pub matched_context: String,
}

/// Pre-signing checklist verification item
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreSigningCheckItem {
    pub id: usize,
    pub task_en: String,
    pub task_hi: String,
    pub task_kn: String,
    #[serde(default)]
    pub is_resolved: bool,
}

impl PreSigningCheckItem {
    pub fn task(&self, lang: Language) -> &str {
        match lang {
            Language::English => &self.task_en,
            Language::Hindi => &self.task_hi,
            Language::Kannada => &self.task_kn,
        }
    }
}

/// Comprehensive analysis payload
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub risk_level: RiskLevel,
    pub summary_en: String,
    pub summary_hi: String,
    pub summary_kn: String,
    pub red_flags: Vec<RedFlag>,
    #[serde(default)]
    pub deadlines: Vec<DeadlineObligation>,
    #[serde(default)]
    pub financial_exposures: Vec<FinancialExposure>,
    #[serde(default)]
    pub statutory_voidabilities: Vec<StatutoryVoidability>,
    #[serde(default)]
    pub clause_breakdowns: Vec<ProblemSolutionBreakdown>,
    #[serde(default)]
    pub ambiguities: Vec<AmbiguityTerm>,
    #[serde(default)]
    pub dealbreaker_matches: Vec<DealbreakerMatch>,
    #[serde(default)]
    pub pre_signing_checklist: Vec<PreSigningCheckItem>,
    #[serde(default)]
    pub is_invalid: bool,
    /// Detailed on-device reasoning output produced by Microsoft Phi-3.5-mini
    #[serde(default)]
    pub phi35_insight: Option<String>,

    /// Contract family classified by structure parser
    #[serde(default)]
    pub contract_family: ContractFamily,

    /// Evidence-backed canonical findings (Hard Invariant: must contain exact source quote)
    #[serde(default)]
    pub canonical_findings: Vec<Finding>,

    /// Cross-clause relationship graph (detects exceptions, overrides, un-capped liabilities)
    #[serde(default)]
    pub relations: Vec<Relation>,

    /// Canonical structured money and dates ledger
    #[serde(default)]
    pub ledger: MoneyDateLedger,

    /// Missing critical clauses expected for this contract family
    #[serde(default)]
    pub missing_clauses: Vec<MissingClause>,

    /// User-defined dealbreakers compiled into typed policy rules
    #[serde(default)]
    pub compiled_policies: Vec<CompiledPolicyRule>,

    /// Specific policy threshold violations
    #[serde(default)]
    pub policy_violations: Vec<Finding>,
}

/// Contract family classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ContractFamily {
    #[default]
    ServicesVendor,
    Employment,
    Nda,
    SaaS,
    General,
}

impl ContractFamily {
    pub fn display_name(&self) -> &'static str {
        match self {
            ContractFamily::ServicesVendor => "Services / Vendor Agreement",
            ContractFamily::Employment => "Employment / Consulting Agreement",
            ContractFamily::Nda => "Non-Disclosure Agreement (NDA)",
            ContractFamily::SaaS => "SaaS / Cloud Agreement",
            ContractFamily::General => "General Commercial Agreement",
        }
    }
}

/// Relationship between clauses in the cross-clause relationship graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationType {
    References,
    Defines,
    Modifies,
    Excludes,
    Overrides,
    Caps,
    Uncaps,
    Survives,
}

impl RelationType {
    pub fn label(&self) -> &'static str {
        match self {
            RelationType::References => "REFERENCES",
            RelationType::Defines => "DEFINES",
            RelationType::Modifies => "MODIFIES",
            RelationType::Excludes => "EXCLUDES",
            RelationType::Overrides => "OVERRIDES",
            RelationType::Caps => "CAPS",
            RelationType::Uncaps => "UNCAPS",
            RelationType::Survives => "SURVIVES",
        }
    }
}

/// Directed edge in the cross-clause relationship graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relation {
    pub from_clause: String,
    pub relation_type: RelationType,
    pub to_clause: String,
    pub evidence_quote: String,
    pub reason: String,
}

/// Evidence-backed canonical legal finding
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub rule_id: String,
    pub category: String, // STATUTORY, FINANCIAL, POLICY, TERMINATION, IP, COMMERCIAL
    pub severity: RiskLevel,
    pub confidence_pct: u8,
    pub affected_party: String,
    pub evidence_quote: String,
    pub related_clauses: Vec<String>,
    pub rationale_key: String,
    pub why_it_matters: String,
    pub questions_to_ask: String,
    pub action_recommendation: String,
}

/// Canonical structured financial and dates ledger
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MoneyDateLedger {
    pub contract_value: Option<String>,
    pub liability_cap: Option<String>,
    pub late_fee_rate: Option<String>,
    pub payment_terms_days: Option<u32>,
    pub termination_notice_days: Option<u32>,
    pub cure_period_days: Option<u32>,
    pub auto_renewal: bool,
    pub auto_renewal_opt_out_days: Option<u32>,
    pub cross_check_warnings: Vec<String>,
}

/// Typed operators for the constrained policy compiler
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyOperator {
    Gt,
    Gte,
    Lt,
    Lte,
    Eq,
    Ne,
    Contains,
    Forbidden,
}

/// Typed policy rule compiled from natural-language dealbreakers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledPolicyRule {
    pub raw_input: String,
    pub field: String,
    pub operator: PolicyOperator,
    pub value_num: Option<i64>,
    pub value_str: Option<String>,
    pub severity: RiskLevel,
    pub interpretation: String,
}

/// Missing critical clause detection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingClause {
    pub name: String,
    pub importance: String,
    pub rationale: String,
    pub suggested_clause_snippet: String,
}

/// Semantic redline change types between contract versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticChangeType {
    RiskIncreased,
    RiskDecreased,
    NewObligation,
    DeletedProtection,
    NumericShift,
    ScopeBroadened,
}

/// Semantic delta entry in contract comparison
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticRedlineDelta {
    pub change_type: SemanticChangeType,
    pub title: String,
    pub clause_ref: String,
    pub before_snippet: String,
    pub after_snippet: String,
    pub explanation: String,
}

impl AnalysisResult {
    pub fn summary(&self, lang: Language) -> &str {
        match lang {
            Language::English => &self.summary_en,
            Language::Hindi => &self.summary_hi,
            Language::Kannada => &self.summary_kn,
        }
    }

    pub fn invalid_fallback() -> Self {
        Self {
            risk_level: RiskLevel::Invalid,
            summary_en: "Unreadable or non-contract document detected. Please scan an actual legal agreement.".into(),
            summary_hi: "गैर-अनुबंध या अपठनीय दस्तावेज़ पाया गया। कृपया एक वास्तविक कानूनी समझौता स्कैन करें।".into(),
            summary_kn: "ಅಪಠ್ಯ ಅಥವಾ ಒಪ್ಪಂದವಲ್ಲದ ದಾಖಲೆ ಪತ್ತೆಯಾಗಿದೆ. ದಯವಿಟ್ಟು ನಿಜವಾದ ಕಾನೂನು ಒಪ್ಪಂದವನ್ನು ಸ್ಕ್ಯಾನ್ ಮಾಡಿ.".into(),
            red_flags: Vec::new(),
            deadlines: Vec::new(),
            financial_exposures: Vec::new(),
            statutory_voidabilities: Vec::new(),
            clause_breakdowns: Vec::new(),
            ambiguities: Vec::new(),
            dealbreaker_matches: Vec::new(),
            pre_signing_checklist: Vec::new(),
            is_invalid: true,
            phi35_insight: None,
            contract_family: ContractFamily::General,
            canonical_findings: Vec::new(),
            relations: Vec::new(),
            ledger: MoneyDateLedger::default(),
            missing_clauses: Vec::new(),
            compiled_policies: Vec::new(),
            policy_violations: Vec::new(),
        }
    }
}

/// Stored history record for offline archival
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub id: String,
    pub timestamp: i64,
    pub formatted_date: String,
    pub risk_level: RiskLevel,
    pub title: String,
    pub snippet: String,
    pub full_text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_labels() {
        assert_eq!(RiskLevel::High.label(Language::English), "CRITICAL RISK");
        assert_eq!(RiskLevel::High.label(Language::Hindi), "गंभीर जोखिम");
        assert_eq!(RiskLevel::High.label(Language::Kannada), "ಹೆಚ್ಚಿನ ಅಪಾಯ");
        assert_eq!(RiskLevel::High.hex_color(), "#EF4444");
    }

    #[test]
    fn test_serde_roundtrip() {
        let result = AnalysisResult {
            risk_level: RiskLevel::High,
            summary_en: "Test summary".into(),
            summary_hi: "टेस्ट सारांश".into(),
            summary_kn: "ಪರೀಕ್ಷಾ ಸಾರಾಂಶ".into(),
            red_flags: vec![RedFlag {
                title_en: "Flag".into(),
                title_hi: "फ्लैग".into(),
                title_kn: "ಫ್ಲ್ಯಾಗ್".into(),
                desc_en: "Desc".into(),
                desc_hi: "विवरण".into(),
                desc_kn: "ವಿವರಣೆ".into(),
                quote_snippet: "Snippet".into(),
            }],
            deadlines: vec![],
            financial_exposures: vec![],
            statutory_voidabilities: vec![],
            clause_breakdowns: vec![],
            ambiguities: vec![],
            dealbreaker_matches: vec![],
            pre_signing_checklist: vec![],
            is_invalid: false,
            phi35_insight: Some("Phi-3.5 analysis complete.".into()),
            contract_family: ContractFamily::ServicesVendor,
            canonical_findings: vec![Finding {
                id: "F-001".into(),
                rule_id: "VE-LIAB-UNCAPPED-001".into(),
                category: "FINANCIAL".into(),
                severity: RiskLevel::High,
                confidence_pct: 95,
                affected_party: "Signing Party".into(),
                evidence_quote: "Customer liability shall be unlimited.".into(),
                related_clauses: vec!["Clause 12".into()],
                rationale_key: "liability.uncapped".into(),
                why_it_matters: "Shifts unlimited exposure to signing party.".into(),
                questions_to_ask: "Can we cap liability at 12 months fees?".into(),
                action_recommendation: "Negotiate mutual cap.".into(),
            }],
            relations: vec![Relation {
                from_clause: "Clause 8 Liability Cap".into(),
                relation_type: RelationType::Excludes,
                to_clause: "Clause 12 Indemnity".into(),
                evidence_quote: "excluding liabilities arising under indemnity".into(),
                reason: "Indemnity is excluded from the overall cap".into(),
            }],
            ledger: MoneyDateLedger::default(),
            missing_clauses: vec![],
            compiled_policies: vec![],
            policy_violations: vec![],
        };

        let json = serde_json::to_string(&result).expect("Failed to serialize");
        let deserialized: AnalysisResult = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(result, deserialized);
    }
}
