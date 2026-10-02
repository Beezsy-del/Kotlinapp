use crate::models::*;
use regex::Regex;

/// Core rule-based legal intelligence engine.
/// Scans contract text for statutory voidability under Indian law,
/// unilateral vulnerabilities, ambiguous language, financial liabilities, and custom dealbreakers.
#[derive(Debug, Clone, Default)]
pub struct ContractEngine;

impl ContractEngine {
    pub fn new() -> Self {
        Self
    }

    /// Primary analysis entrypoint
    pub fn analyze_contract(
        &self,
        text: &str,
        custom_dealbreakers: &[String],
    ) -> AnalysisResult {
        self.analyze_contract_internal(text, custom_dealbreakers, None, Language::English)
    }

    /// Enhanced analysis entrypoint running both rule-based checks and Microsoft Phi-3.5-mini on-device LLM
    pub fn analyze_contract_with_llm(
        &self,
        text: &str,
        custom_dealbreakers: &[String],
        phi35: &crate::phi35_engine::Phi35Engine,
        language: Language,
    ) -> AnalysisResult {
        self.analyze_contract_internal(text, custom_dealbreakers, Some(phi35), language)
    }

    fn analyze_contract_internal(
        &self,
        text: &str,
        custom_dealbreakers: &[String],
        phi35: Option<&crate::phi35_engine::Phi35Engine>,
        language: Language,
    ) -> AnalysisResult {
        let trimmed = text.trim();
        let lower_text = trimmed.to_lowercase();

        if !Self::is_legitimate_contract_text(trimmed, &lower_text) {
            return AnalysisResult::invalid_fallback();
        }

        let mut red_flags = Vec::new();
        let mut deadlines = Vec::new();
        let mut financial_exposures = Vec::new();
        let mut statutory_voidabilities = Vec::new();
        let mut clause_breakdowns = Vec::new();
        let mut ambiguities = Vec::new();
        let mut dealbreaker_matches = Vec::new();
        let mut pre_signing_checklist = Vec::new();

        // 1. Clean OCR artifacts: normalize word-break hyphens across newlines
        let hyphen_regex = Regex::new(r"(\w+)-\s*\n\s*(\w+)").unwrap();
        let normalized_text = hyphen_regex.replace_all(trimmed, "$1$2").to_string();
        let norm_lower = normalized_text.to_lowercase();

        // 2. Custom Dealbreaker Rules
        for rule in custom_dealbreakers {
            let rule_clean = rule.trim();
            if !rule_clean.is_empty() && lower_text.contains(&rule_clean.to_lowercase()) {
                let quote = Self::extract_quote(rule_clean, &normalized_text, &norm_lower);
                dealbreaker_matches.push(DealbreakerMatch {
                    rule_keyword: rule_clean.to_string(),
                    matched_context: quote,
                });
            }
        }

        // 3. Statutory Voidability 1: Non-Compete (Section 27, Indian Contract Act 1872)
        if lower_text.contains("non-compete")
            || lower_text.contains("restraint of trade")
            || lower_text.contains("shall not engage")
        {
            let kw = if lower_text.contains("non-compete") {
                "non-compete"
            } else {
                "shall not engage"
            };
            let quote = Self::extract_quote(kw, &normalized_text, &norm_lower);

            statutory_voidabilities.push(StatutoryVoidability {
                act_section: "Section 27, Indian Contract Act 1872".into(),
                title_en: "Restraint of Trade / Employment Non-Compete".into(),
                title_hi: "व्यापार प्रतिबंध / रोजगार गैर-प्रतिस्पर्धा".into(),
                title_kn: "ವ್ಯಾಪಾರ ನಿರ್ಬಂಧ / ಉದ್ಯೋಗೇತರ ಸ್ಪರ್ಧೆ".into(),
                legal_reason_en: "Post-employment non-compete clauses are legally void under Indian law (Niranjan Shankar Golikari & Percept D'Mark precedents).".into(),
                legal_reason_hi: "भारतीय कानून के तहत रोजगार के बाद गैर-प्रतिस्पर्धा की शर्तें कानूनी रूप से अमान्य हैं।".into(),
                legal_reason_kn: "ಭಾರತೀಯ ಕಾನೂನಿನ ಪ್ರಕಾರ ಉದ್ಯೋಗದ ನಂತರದ ಸ್ಪರ್ಧಾತ್ಮಕವಲ್ಲದ ಶರತ್ತುಗಳು ಅಮಾನ್ಯವಾಗಿವೆ.".into(),
                quote_snippet: quote.clone(),
                status: "POTENTIALLY VOID".into(),
            });

            clause_breakdowns.push(ProblemSolutionBreakdown {
                original_snippet: quote,
                problem_en: "Restricts your lawful right to engage in business, trade, or employment after termination.".into(),
                problem_hi: "समाप्ति के बाद व्यवसाय, व्यापार या रोजगार में संलग्न होने के आपके कानूनी अधिकार को प्रतिबंधित करता है।".into(),
                problem_kn: "ಒಪ್ಪಂದದ ನಂತರ ನಿಮ್ಮ ವೃತ್ತಿ ಅಥವಾ ಉದ್ಯೋಗದ ಹಕ್ಕನ್ನು ನಿರ್ಬಂಧಿಸುತ್ತದೆ.".into(),
                solution_en: "Strike down post-term non-compete completely or limit strictly to non-solicitation of active clients.".into(),
                solution_hi: "गैर-प्रतिस्पर्धा क्लॉज को पूरी तरह से हटाएं या इसे केवल सक्रिय ग्राहकों तक सीमित रखें।".into(),
                solution_kn: "ಈ ಶರತ್ತನ್ನು ಪೂರ್ಣವಾಗಿ ತೆಗೆದುಹಾಕಿ ಅಥವಾ ಕೇವಲ ಪ್ರಸ್ತುತ ಗ್ರಾಹಕರಿಗೆ ಸೀಮಿತಗೊಳಿಸಿ.".into(),
                counter_offer_draft: "Proposed Counter-Clause: 'Nothing in this Agreement shall restrict either party from engaging in their trade or business, provided confidential information is maintained.'".into(),
            });
        }

        // 4. Statutory Voidability 2: Restraint of Legal Proceedings (Section 28, Indian Contract Act 1872)
        if lower_text.contains("exclusive jurisdiction")
            || lower_text.contains("shall not approach court")
            || lower_text.contains("waive right")
            || lower_text.contains("waive all rights to sue")
        {
            let kw = if lower_text.contains("jurisdiction") {
                "jurisdiction"
            } else {
                "waive"
            };
            let quote = Self::extract_quote(kw, &normalized_text, &norm_lower);

            statutory_voidabilities.push(StatutoryVoidability {
                act_section: "Section 28, Indian Contract Act 1872".into(),
                title_en: "Restraint of Legal Proceedings".into(),
                title_hi: "कानूनी कार्यवाही पर रोक".into(),
                title_kn: "ಕಾನೂನು ನಡಾವಳಿಗಳ ನಿರ್ಬಂಧ".into(),
                legal_reason_en: "Clauses restricting statutory court rights or specifying exclusive foreign/remote courts are void under Indian law.".into(),
                legal_reason_hi: "अदालती अधिकारों को प्रतिबंधित करने वाली या दूरस्थ अदालतों का चयन करने वाली धारा अमान्य है।".into(),
                legal_reason_kn: "ನ್ಯಾಯಾಲಯದ ಹಕ್ಕುಗಳನ್ನು ನಿರ್ಬಂಧಿಸುವ ಅಥವಾ ದೂರದ ನ್ಯಾಯಾಲಯವನ್ನು ಕಡ್ಡಾಯಗೊಳಿಸುವ ಶರತ್ತು ಅಮಾನ್ಯವಾಗಿದೆ.".into(),
                quote_snippet: quote,
                status: "POTENTIALLY VOID".into(),
            });
        }

        // 5. Vulnerability 3: Unbalanced Indemnity & Liability
        if lower_text.contains("indemnif")
            || lower_text.contains("hold harmless")
            || lower_text.contains("liable")
        {
            let kw = if lower_text.contains("indemnif") {
                "indemnif"
            } else if lower_text.contains("hold harmless") {
                "hold harmless"
            } else {
                "liable"
            };
            let quote = Self::extract_quote(kw, &normalized_text, &norm_lower);

            clause_breakdowns.push(ProblemSolutionBreakdown {
                original_snippet: quote.clone(),
                problem_en: "Unbalanced liability clause shifts broad, uncapped financial risk for losses onto you.".into(),
                problem_hi: "असंतुलित दायित्व धारा आपके ऊपर व्यापक, बिना सीमा वाला वित्तीय जोखिम डालती है।".into(),
                problem_kn: "ಅಸಮತೋಲಿತ ಜವಾಬ್ದಾರಿ ಶರತ್ತು ನಿಮ್ಮ ಮೇಲೆ ಅಪರಿಮಿತ ಆರ್ಥಿಕ ಅಪಾಯವನ್ನು ಬೀರುತ್ತದೆ.".into(),
                solution_en: "Insert a mutual liability cap limited to total fees paid in the preceding 12 months.".into(),
                solution_hi: "पिछले 12 महीनों में भुगतान किए गए कुल शुल्क तक सीमित परस्पर देयता सीमा (Liability Cap) शामिल करें।".into(),
                solution_kn: "ಕಳೆದ 12 ತಿಂಗಳುಗಳಲ್ಲಿ ಪಾವತಿಸಿದ ಒಟ್ಟು ಶುಲ್ಕಕ್ಕೆ ಸೀಮಿತವಾದ ಜವಾಬ್ದಾರಿ ಮಿತಿಯನ್ನು ಸೇರಿಸಿ.".into(),
                counter_offer_draft: "Proposed Counter-Clause: 'Mutual Liability Cap: Each party's maximum aggregate liability shall be capped at the total fees paid under this Agreement in the preceding 12 months.'".into(),
            });

            red_flags.push(RedFlag {
                title_en: "Unbalanced Indemnification".into(),
                title_hi: "असंतुलित क्षतिपूर्ति".into(),
                title_kn: "ಅಸಮತೋಲಿತ ನಷ್ಟ ಪರಿಹಾರ".into(),
                desc_en: "Unilateral indemnification protecting the counterparty unconditionally.".into(),
                desc_hi: "दूसरी पार्टी को बिना शर्त सुरक्षित करने वाला असंतुलित दायित्व।".into(),
                desc_kn: "ಇತರ ಪಕ್ಷವನ್ನು ಶರತ್ತಿಲ್ಲದೆ ರಕ್ಷಿಸುವ ಅಸಮತೋಲಿತ ಜವಾಬ್ದಾರಿ.".into(),
                quote_snippet: quote,
            });
        }

        // 6. Vulnerability 4: Unilateral Termination & Cancellation Rights
        if lower_text.contains("terminate")
            || lower_text.contains("cancellation")
            || lower_text.contains("right to terminate")
        {
            let quote = Self::extract_quote("terminate", &normalized_text, &norm_lower);
            clause_breakdowns.push(ProblemSolutionBreakdown {
                original_snippet: quote,
                problem_en: "Unilateral or immediate termination rights create sudden operational instability.".into(),
                problem_hi: "एकपक्षीय या तत्काल समाप्ति अधिकार अचानक व्यापार जोखिम और अनिश्चितता पैदा करते हैं।".into(),
                problem_kn: "ಏಕಪಕ್ಷೀಯ ರದ್ದತಿ ಹಕ್ಕುಗಳು ತಕ್ಷಣದ ಕಾರ್ಯಚರಣೆಯ ಅಪಾಯವನ್ನು ಉಂಟುಮಾಡುತ್ತವೆ.".into(),
                solution_en: "Require mutual 30-day written notice and a mandatory 15-day cure period for default.".into(),
                solution_hi: "रद्द करने से पहले 30 दिनों का लिखित नोटिस और 15 दिनों की सुधारात्मक अवधि (Cure Period) अनिवार्य करें।".into(),
                solution_kn: "ರದ್ದುಗೊಳಿಸುವ ಮುನ್ನ ಕಡ್ಡಾಯ 30 ದಿನಗಳ ಬರವಣಿಗೆಯ ನೋಟಿಸ್ ಮತ್ತು 15 ದಿನಗಳ ಕಾಲಾವಕಾಶ ಕೇಳಿ.".into(),
                counter_offer_draft: "Proposed Counter-Clause: 'Termination Notice: Either party may terminate this Agreement upon thirty (30) days' prior written notice, subject to a 15-day cure period for breach.'".into(),
            });
        }

        // 7. Vulnerability 5: Broad IP Ownership & Work-for-Hire Assignment
        if lower_text.contains("intellectual property")
            || lower_text.contains("work for hire")
            || lower_text.contains("assigns all rights")
            || lower_text.contains("ownership of deliverables")
        {
            let kw = if lower_text.contains("work for hire") {
                "work for hire"
            } else {
                "intellectual property"
            };
            let quote = Self::extract_quote(kw, &normalized_text, &norm_lower);

            clause_breakdowns.push(ProblemSolutionBreakdown {
                original_snippet: quote,
                problem_en: "Broad IP assignment transfers background tools, pre-existing assets, and rights prior to full payment.".into(),
                problem_hi: "व्यापक आईपी असाइनमेंट पूर्ण भुगतान से पहले ही आपके टूल, संपत्ति और अधिकारों को स्थानांतरित कर देता है।".into(),
                problem_kn: "ಪೂರ್ಣ ಪಾವತಿಗೆ ಮೊದಲೇ ನಿಮ್ಮ ಮೂಲ ಉಪಕರಣಗಳು ಮತ್ತು ಹಕ್ಕುಗಳನ್ನು ವರ್ಗಾಯಿಸುತ್ತದೆ.".into(),
                solution_en: "Retain background IP tools and stipulate that deliverable IP transfers only upon full payment.".into(),
                solution_hi: "मूल आईपी टूल्स अपने पास रखें और स्पष्ट करें कि आईपी केवल पूर्ण भुगतान प्राप्त होने पर ही स्थानांतरित होगा।".into(),
                solution_kn: "ಹಿನ್ನೆಲೆ IP ಅನ್ನು ನಿಮ್ಮಲ್ಲಿಯೇ ಇರಿಸಿಕೊಳ್ಳಿ ಮತ್ತು ಸಂಪೂರ್ಣ ಪಾವತಿಯ ನಂತರವೇ IP ಹಸ್ತಾಂತರವಾಗುತ್ತದೆ ಎಂದು ಸ್ಪಷ್ಟಪಡಿಸಿ.".into(),
                counter_offer_draft: "Proposed Counter-Clause: 'IP Rights Transfer: Deliverables shall be assigned to Client only upon receipt of full and final payment. Vendor retains all rights to pre-existing background tools and code.'".into(),
            });
        }

        // 8. Vulnerability 6: Unilateral Modifications / Policy Alteration
        if lower_text.contains("modify")
            || lower_text.contains("amend")
            || lower_text.contains("reserve the right")
        {
            let kw = if lower_text.contains("reserve the right") {
                "reserve the right"
            } else {
                "modify"
            };
            let quote = Self::extract_quote(kw, &normalized_text, &norm_lower);

            clause_breakdowns.push(ProblemSolutionBreakdown {
                original_snippet: quote,
                problem_en: "Allows the counterparty to alter agreement terms, pricing, or schedules unilaterally without consent.".into(),
                problem_hi: "दूसरी पार्टी को बिना आपकी सहमति के शर्तों, दरों या नीतियों को बदलने की अनुमति देता है।".into(),
                problem_kn: "ನಿಮ್ಮ ಸಮ್ಮತಿಯಿಲ್ಲದೆ ಒಪ್ಪಂದದ ನಿಯಮಗಳು ಮತ್ತು ಬೆಲೆಗಳನ್ನು ಮಾರ್ಪಡಿಸಲು ಅನುಮತಿಸುತ್ತದೆ.".into(),
                solution_en: "Require mutual written consent signed by both authorized representatives for all modifications.".into(),
                solution_hi: "सभी संशोधनों के लिए दोनों पक्षों के हस्ताक्षर वाले लिखित दस्तावेज़ को अनिवार्य करें।".into(),
                solution_kn: "ಎಲ್ಲಾ ತಿದ್ದುಪಡಿಗಳಿಗೆ ಎರಡೂ ಕಡೆಯವರ ಲಿಖಿತ ಸಹಿಯನ್ನು ಕಡ್ಡಾಯಗೊಳಿಸಿ.".into(),
                counter_offer_draft: "Proposed Counter-Clause: 'Amendments: No modification or amendment of this Agreement shall be effective unless made in writing and signed by authorized representatives of both parties.'".into(),
            });
        }

        // 9. Vulnerability 7: Auto-Renewal & Lock-in Clauses
        if lower_text.contains("automatically renew")
            || lower_text.contains("auto-renew")
            || lower_text.contains("perpetual renewal")
        {
            let kw = if lower_text.contains("automatically renew") {
                "automatically renew"
            } else {
                "auto-renew"
            };
            let quote = Self::extract_quote(kw, &normalized_text, &norm_lower);

            clause_breakdowns.push(ProblemSolutionBreakdown {
                original_snippet: quote,
                problem_en: "Automatic renewal locks you into extended contract terms and financial obligations unless cancelled early.".into(),
                problem_hi: "स्वचालित नवीनीकरण आपको समय सीमा से पहले रद्द न करने पर लंबी वित्तीय देनदारियों में बांध देता है।".into(),
                problem_kn: "ಸ್ವಯಂಚಾಲಿತ ನವೀಕರಣವು ನಿಮ್ಮನ್ನು ಹೆಚ್ಚುವರಿ ಶುಲ್ಕ ಮತ್ತು ಒಪ್ಪಂದದ ಚೌಕಟ್ಟಿನಲ್ಲಿ ಬಂಧಿಸುತ್ತದೆ.".into(),
                solution_en: "Require annual explicit opt-in confirmation or a flexible 30-day non-renewal notice window.".into(),
                solution_hi: "वार्षिक स्पष्ट स्वीकृति (Opt-in) या 30 दिन का आसान गैर-नवीनीकरण नोटिस जोड़ें।".into(),
                solution_kn: "ವಾರ್ಷಿಕ ಸ್ಪಷ್ಟ ಒಪ್ಪಿಗೆ ಅಥವಾ 30 ದಿನಗಳ ನೋಟಿಸ್ ಮೂಲಕ ರದ್ದುಗೊಳಿಸುವ ಅವಕಾಶವನ್ನು ಸೇರಿಸಿ.".into(),
                counter_offer_draft: "Proposed Counter-Clause: 'Term Renewal: This Agreement shall renew only upon explicit written agreement of both parties at least 30 days prior to the expiration of the current term.'".into(),
            });
        }

        // 10. Deadlines & Timeframe Obligations
        let duration_regex = Regex::new(r"(?i)\b\d+\s*(?:days|months|years|weeks|hours)\b").unwrap();
        for mat in duration_regex.find_iter(trimmed).take(4) {
            let val = mat.as_str();
            deadlines.push(DeadlineObligation {
                timeframe: val.to_string(),
                obligation_en: "Required notice period or operational deadline clause.".into(),
                obligation_hi: "आवश्यक नोटिस अवधि या परिचालन समय सीमा।".into(),
                obligation_kn: "ಅಗತ್ಯ ನೋಟಿಸ್ ಅವಧಿ ಅಥವಾ ಸಮಯದ ಜವಾಬ್ದಾರಿ.".into(),
                quote_snippet: Self::extract_quote(val, &normalized_text, &norm_lower),
            });
        }

        // 11. Financial Exposures & Fees
        let amount_regex = Regex::new(r"(?i)(?:rs\.?|inr|\$|usd|₹)\s*\d+(?:,\d+)*(?:\.\d+)?|\d+\s*(?:percent|%|per annum)").unwrap();
        for mat in amount_regex.find_iter(trimmed).take(4) {
            let val = mat.as_str();
            financial_exposures.push(FinancialExposure {
                title_en: "Payment Obligation / Fee Requirement".into(),
                title_hi: "भुगतान दायित्व / शुल्क आवश्यकता".into(),
                title_kn: "ಪಾವತಿ ಜವಾಬ್ದಾರಿ / ಶುಲ್ಕದ ಅಗತ್ಯತೆ".into(),
                amount_or_cost: val.to_string(),
                description_en: "Explicit payment obligation, fee, or financial penalty mentioned in text.".into(),
                description_hi: "अनुबंध में उल्लिखित स्पष्ट भुगतान दायित्व या जुर्माना।".into(),
                description_kn: "ಒಪ್ಪಂದದಲ್ಲಿ ಉಲ್ಲೇಖಿಸಲಾದ ಸ್ಪಷ್ಟ ಪಾವತಿ ಜವಾಬ್ದಾರಿ.".into(),
                quote_snippet: Self::extract_quote(val, &normalized_text, &norm_lower),
            });
        }

        if lower_text.contains("penalty") || lower_text.contains("late fee") || lower_text.contains("interest") {
            let kw = if lower_text.contains("penalty") { "penalty" } else { "late fee" };
            let quote = Self::extract_quote(kw, &normalized_text, &norm_lower);
            financial_exposures.push(FinancialExposure {
                title_en: "Uncapped Late Fee & Interest Penalty".into(),
                title_hi: "अनकैप्ड लेट फी और ब्याज जुर्माना".into(),
                title_kn: "ಮಿತಿಯಿಲ್ಲದ ತಡವಾದ ಪಾವತಿ ಬಡ್ಡಿ ದಂಡ".into(),
                amount_or_cost: "Variable Penalty".into(),
                description_en: "Contract specifies cumulative late payment penalties or compounding interest.".into(),
                description_hi: "अनुबंध में बिना अधिकतम सीमा के देय शुल्क या ब्याज निर्दिष्ट है।".into(),
                description_kn: "ಒಪ್ಪಂದವು ಮಿತಿಯಿಲ್ಲದೆ ತಡವಾದ ಪಾವತಿ ದಂಡವನ್ನು ನಿರ್ದಿಷ್ಟಪಡಿಸುತ್ತದೆ.".into(),
                quote_snippet: quote,
            });
        }

        // 12. Ambiguity & Discretion Clauses
        let ambiguity_map = [
            ("sole discretion", (
                "Grants absolute unilateral authority to alter terms or determine breach without appeal.",
                "बिना अपील के शर्तों को बदलने का एकपक्षीय अधिकार देता है।",
                "ಯಾವುದೇ ಮೇಲ್ಮನವಿಯಿಲ್ಲದೆ ನಿಯಮಗಳನ್ನು ಬದಲಾಯಿಸುವ ಏಕಪಕ್ಷೀಯ ಅಧಿಕಾರ ನೀಡುತ್ತದೆ."
            )),
            ("reasonable efforts", (
                "Vague legal standard that lacks enforceable metrics or performance KPIs.",
                "अस्पष्ट कानूनी मानक जिसमें लागू करने योग्य प्रदर्शन मेट्रिक्स का अभाव है।",
                "ಅಳೆಯಬಹುದಾದ ಕಾರ್ಯಕ್ಷಮತೆಯ ಮಾನದಂಡಗಳಿಲ್ಲದ ಅಸ್ಪಷ್ಟ ನಿಯಮ."
            )),
            ("from time to time", (
                "Allows repeated unilateral modification of policies without formal addendums.",
                "फॉर्मल ऐडेंडम के बिना नीतियों में बार-बार बदलाव की अनुमति देता है।",
                "ಲಿಖಿತ ಒಪ್ಪಂದವಿಲ್ಲದೆ ನಿಯಮಗಳನ್ನು ಬದಲಾಯಿಸಲು ಅನುಮತಿಸುತ್ತದೆ."
            )),
            ("as deemed fit", (
                "Subjective standard that severely weakens your legal defenses in dispute.",
                "विषयपरक धारा जो विवाद में आपके कानूनी बचाव को कमजोर करती है।",
                "ವಿವಾದದ ಸಂದರ್ಭದಲ್ಲಿ ನಿಮ್ಮ ರಕ್ಷಣೆಯನ್ನು ಬಲಹೀನಗೊಳಿಸುತ್ತದೆ."
            ))
        ];

        for (phrase, (en, hi, kn)) in ambiguity_map {
            if lower_text.contains(phrase) {
                let quote = Self::extract_quote(phrase, &normalized_text, &norm_lower);
                ambiguities.push(AmbiguityTerm {
                    phrase: format!("\"{}\"", phrase),
                    explanation_en: en.to_string(),
                    explanation_hi: hi.to_string(),
                    explanation_kn: kn.to_string(),
                    quote_snippet: quote,
                });
            }
        }

        if red_flags.is_empty() {
            red_flags.push(RedFlag {
                title_en: "Standard Operational Terms Detected".into(),
                title_hi: "मानक संचालन शर्तें पाई गईं".into(),
                title_kn: "ಸಾಮಾನ್ಯ ಒಪ್ಪಂದದ ನಿಯಮಗಳು".into(),
                desc_en: "Standard balanced legal provisions identified in document.".into(),
                desc_hi: "दस्तावेज़ में मानक संतुलित कानूनी शर्तें पाई गईं।".into(),
                desc_kn: "ದಾಖಲೆಯಲ್ಲಿ ಸಮತೋಲಿತ ಸಾಮಾನ್ಯ ನಿಯಮಗಳು ಕಂಡುಬಂದಿವೆ.".into(),
                quote_snippet: String::new(),
            });
        }

        // 13. Pre-Signing Checklist Generation
        let mut check_id = 1;
        if !clause_breakdowns.is_empty() {
            pre_signing_checklist.push(PreSigningCheckItem {
                id: check_id,
                task_en: "Negotiate counter-offer amendments for identified high-risk vulnerabilities.".into(),
                task_hi: "पहचाने गए जोखिमपूर्ण क्लॉज के लिए जवाबी प्रस्ताव संशोधनों पर बातचीत करें।".into(),
                task_kn: "ಗುರುತಿಸಲಾದ ಅಪಾಯಕಾರಿ ಶರತ್ತುಗಳಿಗೆ ಪ್ರತಿಸಲ್ಲಿಕೆ ತಿದ್ದುಪಡಿಗಳನ್ನು ಚರ್ಚಿಸಿ.".into(),
                is_resolved: false,
            });
            check_id += 1;
        }

        if let Some(first_deadline) = deadlines.first() {
            pre_signing_checklist.push(PreSigningCheckItem {
                id: check_id,
                task_en: format!("Verify notice period obligations ({}).", first_deadline.timeframe),
                task_hi: "नोटिस अवधि के दायित्वों की पुष्टि करें।".into(),
                task_kn: "ನೋಟಿಸ್ ಅವಧಿಯ ಜವಾಬ್ದಾರಿಗಳನ್ನು ಪರಿಶೀಲಿಸಿ.".into(),
                is_resolved: false,
            });
            check_id += 1;
        }

        if !financial_exposures.is_empty() {
            pre_signing_checklist.push(PreSigningCheckItem {
                id: check_id,
                task_en: "Confirm all fee caps, payment milestones, and penalty caps.".into(),
                task_hi: "सभी शुल्क सीमाओं और देय जुर्माने की पुष्टि करें।".into(),
                task_kn: "ಎಲ್ಲಾ ಶುಲ್ಕ ಮಿತಿಗಳು ಮತ್ತು ಪಾವತಿ ದಂಡಗಳನ್ನು ದೃಢೀಕರಿಸಿ.".into(),
                is_resolved: false,
            });
            check_id += 1;
        }

        pre_signing_checklist.push(PreSigningCheckItem {
            id: check_id,
            task_en: "Verify all attached schedules, annexures, and exhibits.".into(),
            task_hi: "सभी संलग्न अनुसूचियों और अनुलग्नकों को सत्यापित करें।".into(),
            task_kn: "ಎಲ್ಲಾ ವೇಳಾಪಟ್ಟಿಗಳು ಮತ್ತು ಲಗತ್ತುಗಳನ್ನು ಪರಿಶೀಲಿಸಿ.".into(),
            is_resolved: false,
        });

        // 14. Compute Overall Risk Level
        let risk_level = if !dealbreaker_matches.is_empty()
            || !statutory_voidabilities.is_empty()
            || clause_breakdowns.len() >= 2
        {
            RiskLevel::High
        } else if !deadlines.is_empty() || !financial_exposures.is_empty() {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        let summary_en = match risk_level {
            RiskLevel::High => "High risk! Critical legal vulnerabilities or statutory voidability issues detected. Resolve proposed counter-clauses before signing.",
            RiskLevel::Medium => "Moderate risk. Review payment obligations, deadlines, and ambiguous discretionary terms.",
            RiskLevel::Low => "Low risk. Standard agreement with balanced legal terms.",
            RiskLevel::Invalid => "",
        }.to_string();

        let summary_hi = match risk_level {
            RiskLevel::High => "उच्च जोखिम! गंभीर कानूनी कमियां या वैधानिक मुद्दे पाए गए। हस्ताक्षर करने से पहले प्रस्तावित जवाबी धाराओं पर बातचीत करें।",
            RiskLevel::Medium => "मध्यम जोखिम। भुगतान देनदारियों, समय सीमा और विवेकपूर्ण शर्तों की समीक्षा करें।",
            RiskLevel::Low => "कम जोखिम। संतुलित प्रावधानों के साथ मानक समझौता।",
            RiskLevel::Invalid => "",
        }.to_string();

        let summary_kn = match risk_level {
            RiskLevel::High => "ಹೆಚ್ಚಿನ ಅಪಾಯ! ಗಂಭೀರ ಕಾನೂನು ಲೋಪದೋಷಗಳಿವೆ. ಸಹಿ ಮಾಡುವ ಮುನ್ನ ಸೂಚಿಸಲಾದ ತಿದ್ದುಪಡಿಗಳನ್ನು ಪರಿಶೀಲಿಸಿ.",
            RiskLevel::Medium => "ಮಧ್ಯಮ ಅಪಾಯ. ಸಮಯದ ಮಿತಿಗಳು ಮತ್ತು ಪಾವತಿ ಜವಾಬ್ದಾರಿಗಳನ್ನು ಪರಿಶೀಲಿಸಿ.",
            RiskLevel::Low => "ಕಡಿಮೆ ಅಪಾಯ. ಸಮತೋಲಿತ ನಿಯಮಗಳೊಂದಿಗೆ ಸಾಮಾನ್ಯ ಒಪ್ಪಂದ.",
            RiskLevel::Invalid => "",
        }.to_string();

        let mut result = AnalysisResult {
            risk_level,
            summary_en,
            summary_hi,
            summary_kn,
            red_flags,
            deadlines,
            financial_exposures,
            statutory_voidabilities,
            clause_breakdowns,
            ambiguities,
            dealbreaker_matches,
            pre_signing_checklist,
            is_invalid: false,
            phi35_insight: None,
        };

        if let Some(llm) = phi35 {
            result.phi35_insight = Some(llm.analyze(trimmed, &result, language));
        }

        result
    }

    /// Check if the document appears to be a legitimate contract rather than garbage/image noise
    pub fn is_legitimate_contract_text(text: &str, lower_text: &str) -> bool {
        if text.len() < 15 {
            return false;
        }

        let legal_keywords = [
            "agreement", "contract", "party", "parties", "shall", "terms", "conditions",
            "clause", "liability", "section", "right", "rights", "service", "notice",
            "payment", "policy", "obligation", "lease", "rent", "tenant", "employee",
            "indemnify", "jurisdiction", "अनुबंध", "समझौता", "शर्तें", "पक्ष", "ಒಪ್ಪಂದ", "ನಿಯಮಗಳು"
        ];

        let count = legal_keywords
            .iter()
            .filter(|&&kw| lower_text.contains(kw))
            .count();

        count >= 1
    }

    /// Extract a contextual quote around a keyword without cutting words in half
    pub fn extract_quote(keyword: &str, source_text: &str, norm_lower: &str) -> String {
        let target = keyword.to_lowercase();
        let target_idx = match norm_lower.find(&target) {
            Some(idx) => idx,
            None => return String::new(),
        };

        // 1. Try sentence matching first
        for sentence in source_text.split(|c| c == '.' || c == ';' || c == '\n') {
            if sentence.to_lowercase().contains(&target) {
                let clean = sentence.split_whitespace().collect::<Vec<_>>().join(" ");
                if clean.len() >= 15 && clean.len() <= 600 {
                    return clean;
                }
            }
        }

        // 2. Fallback: Word-boundary-aware expansion
        let chars: Vec<char> = source_text.chars().collect();
        let total_len = chars.len();
        let idx = target_idx.min(total_len.saturating_sub(1));

        let mut start = idx;
        while start > 0 && chars[start - 1] != '.' && chars[start - 1] != '\n' && (idx - start) < 250 {
            start -= 1;
        }
        while start > 0 && !chars[start - 1].is_whitespace() && chars[start - 1] != '.' && chars[start - 1] != '\n' {
            start -= 1;
        }

        let mut end = (idx + target.chars().count()).min(total_len);
        while end < total_len && chars[end] != '.' && chars[end] != '\n' && (end - idx) < 350 {
            end += 1;
        }
        while end < total_len && !chars[end].is_whitespace() && chars[end] != '.' && chars[end] != '\n' {
            end += 1;
        }
        if end < total_len && chars[end] == '.' {
            end += 1;
        }

        let slice: String = chars[start..end].iter().collect();
        let clean = slice.split_whitespace().collect::<Vec<_>>().join(" ");

        let mut snippet = clean;
        if start > 0 && !snippet.starts_with('.') {
            snippet = format!("...{}", snippet);
        }
        if end < total_len && !snippet.ends_with('.') {
            snippet = format!("{}...", snippet);
        }

        snippet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_contract_rejection() {
        let engine = ContractEngine::new();
        let result = engine.analyze_contract("hello world image noise 123", &[]);
        assert!(result.is_invalid);
        assert_eq!(result.risk_level, RiskLevel::Invalid);
    }

    #[test]
    fn test_statutory_voidability_section_27() {
        let engine = ContractEngine::new();
        let contract = r#"
            EMPLOYMENT AGREEMENT
            Clause 5: The employee agrees that post-employment, a non-compete restriction shall apply
            for 24 months. The party shall not engage in any competing software development business.
        "#;
        let result = engine.analyze_contract(contract, &[]);
        assert_eq!(result.risk_level, RiskLevel::High);
        assert!(!result.statutory_voidabilities.is_empty());
        assert_eq!(result.statutory_voidabilities[0].act_section, "Section 27, Indian Contract Act 1872");
        assert!(!result.clause_breakdowns.is_empty());
        assert!(result.clause_breakdowns[0].counter_offer_draft.contains("Proposed Counter-Clause"));
    }

    #[test]
    fn test_custom_dealbreaker_and_discretion() {
        let engine = ContractEngine::new();
        let contract = r#"
            SERVICE AGREEMENT
            Clause 8: The vendor may terminate the contract at its sole discretion upon 30 days notice.
            Fee of INR 50,000 shall be payable upon signing.
        "#;
        let dealbreakers = vec!["sole discretion".to_string()];
        let result = engine.analyze_contract(contract, &dealbreakers);
        assert_eq!(result.risk_level, RiskLevel::High);
        assert_eq!(result.dealbreaker_matches.len(), 1);
        assert_eq!(result.dealbreaker_matches[0].rule_keyword, "sole discretion");
        assert!(!result.ambiguities.is_empty());
        assert!(!result.deadlines.is_empty());
        assert!(!result.financial_exposures.is_empty());
    }
}
