use crate::models::{SemanticChangeType, SemanticRedlineDelta};
use regex::Regex;

/// Semantic Redline & Contract Comparison Engine
/// Compares two versions of a contract and identifies legal risk changes, scope shifts, and numeric drift
#[derive(Debug, Clone, Default)]
pub struct SemanticRedline;

impl SemanticRedline {
    pub fn new() -> Self {
        Self
    }

    /// Compare Version A (Original/Baseline) with Version B (Revised/Counter-Draft)
    pub fn compare(&self, original: &str, revised: &str) -> Vec<SemanticRedlineDelta> {
        let mut deltas = Vec::new();

        let orig_lower = original.to_lowercase();
        let rev_lower = revised.to_lowercase();

        // 1. Check Liability Cap Changes
        let orig_has_cap = orig_lower.contains("liability shall not exceed") || orig_lower.contains("aggregate liability") || orig_lower.contains("liability cap");
        let rev_has_cap = rev_lower.contains("liability shall not exceed") || rev_lower.contains("aggregate liability") || rev_lower.contains("liability cap");

        if orig_has_cap && !rev_has_cap {
            deltas.push(SemanticRedlineDelta {
                change_type: SemanticChangeType::RiskIncreased,
                title: "Liability Cap Removed".into(),
                clause_ref: "Limitation of Liability".into(),
                before_snippet: "Liability capped in original version".into(),
                after_snippet: "[Deleted in revised version]".into(),
                explanation: "The liability cap was removed in the revised draft, exposing your balance sheet to uncapped damages.".into(),
            });
        } else if !orig_has_cap && rev_has_cap {
            deltas.push(SemanticRedlineDelta {
                change_type: SemanticChangeType::RiskDecreased,
                title: "Liability Cap Added".into(),
                clause_ref: "Limitation of Liability".into(),
                before_snippet: "[No cap in original version]".into(),
                after_snippet: "Liability capped in revised version".into(),
                explanation: "The revised draft successfully establishes a finite ceiling on potential damages exposure.".into(),
            });
        }

        // 2. Check Indemnity Symmetry Shift
        let orig_unilateral_indemnity = orig_lower.contains("vendor shall indemnify client") || orig_lower.contains("employee shall indemnify");
        let rev_mutual_indemnity = rev_lower.contains("each party shall indemnify") || rev_lower.contains("mutual indemnification") || rev_lower.contains("parties agree to indemnify each other");

        if orig_unilateral_indemnity && rev_mutual_indemnity {
            deltas.push(SemanticRedlineDelta {
                change_type: SemanticChangeType::RiskDecreased,
                title: "Indemnity Converted from Unilateral to Mutual".into(),
                clause_ref: "Indemnification".into(),
                before_snippet: "One-sided indemnity obligation".into(),
                after_snippet: "Bilateral mutual indemnification".into(),
                explanation: "The one-sided indemnity risk was successfully negotiated into a balanced, mutual protection covenant.".into(),
            });
        }

        // 3. Check Termination Notice Shifts (e.g. 30 -> 90 or 90 -> 30)
        let notice_re = Regex::new(r"(\d+)\s*(?:calendar\s*)?days\s*(?:prior\s*)?(?:written\s*)?notice").unwrap();
        let orig_notice = notice_re.captures(&orig_lower).and_then(|c| c.get(1)).and_then(|m| m.as_str().parse::<u32>().ok());
        let rev_notice = notice_re.captures(&rev_lower).and_then(|c| c.get(1)).and_then(|m| m.as_str().parse::<u32>().ok());

        if let (Some(on), Some(rn)) = (orig_notice, rev_notice) {
            if on != rn {
                let change_type = if rn > on {
                    SemanticChangeType::RiskIncreased
                } else {
                    SemanticChangeType::RiskDecreased
                };
                deltas.push(SemanticRedlineDelta {
                    change_type,
                    title: format!("Termination Notice Changed: {} -> {} Days", on, rn),
                    clause_ref: "Termination Notice".into(),
                    before_snippet: format!("{} days notice", on),
                    after_snippet: format!("{} days notice", rn),
                    explanation: format!("Contract notice timeframe was altered from {} days to {} days, affecting operational flexibility.", on, rn),
                });
            }
        }

        // 4. Check for New Obligations (e.g. Audit Rights, Regular Reporting)
        if !orig_lower.contains("audit right") && rev_lower.contains("audit") {
            deltas.push(SemanticRedlineDelta {
                change_type: SemanticChangeType::NewObligation,
                title: "New Right to Audit Inserted".into(),
                clause_ref: "Audit & Compliance".into(),
                before_snippet: "[No audit clause in original]".into(),
                after_snippet: "Counterparty reserves right to audit records".into(),
                explanation: "Counterparty inserted an operational audit requirement permitting inspections of books, records, or premises.".into(),
            });
        }

        // 5. Check for Deleted Protections (e.g. Termination for Convenience)
        let orig_conv = orig_lower.contains("for convenience") || orig_lower.contains("without cause");
        let rev_conv = rev_lower.contains("for convenience") || rev_lower.contains("without cause");

        if orig_conv && !rev_conv {
            deltas.push(SemanticRedlineDelta {
                change_type: SemanticChangeType::DeletedProtection,
                title: "Termination for Convenience Removed".into(),
                clause_ref: "Term & Termination".into(),
                before_snippet: "Either party may terminate for convenience".into(),
                after_snippet: "[Stripped out in revised draft]".into(),
                explanation: "Your right to exit the contract for commercial convenience was deleted, locking you into the full contract duration.".into(),
            });
        }

        // 6. Check Scope Expansion of Confidentiality
        if !orig_lower.contains("all discussions and ideas") && rev_lower.contains("all discussions and ideas") {
            deltas.push(SemanticRedlineDelta {
                change_type: SemanticChangeType::ScopeBroadened,
                title: "Confidential Information Scope Broadened".into(),
                clause_ref: "Definition of Confidential Information".into(),
                before_snippet: "Standard marked confidential information".into(),
                after_snippet: "Broadened to all discussions, ideas, and concepts".into(),
                explanation: "The definition of proprietary information was broadened substantially, increasing risk of accidental breach.".into(),
            });
        }

        deltas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_contract_redline() {
        let redline = SemanticRedline::new();
        let orig = r#"
            Party A may terminate upon 30 days written notice.
            Either party may terminate for convenience.
            Liability shall not exceed INR 5,00,000.
        "#;
        let rev = r#"
            Party A may terminate upon 90 days written notice.
            Liability is uncapped without limitation.
            Client reserves the right to audit Vendor records quarterly.
        "#;

        let deltas = redline.compare(orig, rev);
        assert!(!deltas.is_empty());
        assert!(deltas.iter().any(|d| d.change_type == SemanticChangeType::RiskIncreased));
        assert!(deltas.iter().any(|d| d.change_type == SemanticChangeType::NewObligation));
        assert!(deltas.iter().any(|d| d.change_type == SemanticChangeType::DeletedProtection));
    }
}
