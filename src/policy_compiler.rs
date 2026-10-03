use crate::models::{CompiledPolicyRule, Finding, MoneyDateLedger, PolicyOperator, RiskLevel};
use regex::Regex;

/// Constrained Policy Compiler
/// Transforms natural-language dealbreakers into typed predicate rules and executes deterministic evaluations
#[derive(Debug, Clone, Default)]
pub struct PolicyCompiler;

impl PolicyCompiler {
    pub fn new() -> Self {
        Self
    }

    /// Compile a user-entered dealbreaker string into a typed Policy Rule
    pub fn compile(&self, input: &str) -> CompiledPolicyRule {
        let trimmed = input.trim();
        let lower = trimmed.to_lowercase();

        // 1. Notice days policy (e.g. "notice > 30 days", "notice longer than 30 days", "notice <= 30")
        if lower.contains("notice") && (lower.contains("day") || lower.contains(">") || lower.contains("<")) {
            let num_re = Regex::new(r"(\d+)").unwrap();
            let val = num_re
                .find(&lower)
                .and_then(|m| m.as_str().parse::<i64>().ok())
                .unwrap_or(30);

            let op = if lower.contains(">") || lower.contains("longer") || lower.contains("more than") || lower.contains("exceed") {
                PolicyOperator::Gt
            } else if lower.contains("<=") || lower.contains("at most") {
                PolicyOperator::Lte
            } else {
                PolicyOperator::Gt
            };

            return CompiledPolicyRule {
                raw_input: trimmed.to_string(),
                field: "termination.notice_days".into(),
                operator: op,
                value_num: Some(val),
                value_str: None,
                severity: RiskLevel::High,
                interpretation: format!("Flag contracts where termination notice period is {:?} {} days", op, val),
            };
        }

        // 2. Auto-renewal prohibition ("no auto renewal", "auto renewal forbidden", "no automatic renewal")
        if lower.contains("auto") && (lower.contains("renew") || lower.contains("extension")) {
            return CompiledPolicyRule {
                raw_input: trimmed.to_string(),
                field: "contract.auto_renewal".into(),
                operator: PolicyOperator::Forbidden,
                value_num: None,
                value_str: Some("true".into()),
                severity: RiskLevel::High,
                interpretation: "Prohibit any automatic term renewal without explicit affirmative consent".into(),
            };
        }

        // 3. Exclusivity prohibition ("no exclusivity", "exclusive forbidden")
        if lower.contains("exclusiv") {
            return CompiledPolicyRule {
                raw_input: trimmed.to_string(),
                field: "commercial.exclusivity".into(),
                operator: PolicyOperator::Forbidden,
                value_num: None,
                value_str: Some("true".into()),
                severity: RiskLevel::High,
                interpretation: "Prohibit exclusivity restrictions or lock-outs on business operations".into(),
            };
        }

        // 4. Liability cap requirement ("liability <= contract value", "no unlimited liability", "capped liability")
        if lower.contains("liab") && (lower.contains("unlimited") || lower.contains("uncapped") || lower.contains("cap")) {
            return CompiledPolicyRule {
                raw_input: trimmed.to_string(),
                field: "liability.uncapped".into(),
                operator: PolicyOperator::Forbidden,
                value_num: None,
                value_str: Some("true".into()),
                severity: RiskLevel::High,
                interpretation: "Prohibit uncapped liability; require mutual liability ceiling".into(),
            };
        }

        // 5. Non-compete prohibition ("no non-compete", "non compete")
        if lower.contains("compete") || lower.contains("restraint") {
            return CompiledPolicyRule {
                raw_input: trimmed.to_string(),
                field: "statutory.non_compete".into(),
                operator: PolicyOperator::Forbidden,
                value_num: None,
                value_str: Some("true".into()),
                severity: RiskLevel::High,
                interpretation: "Prohibit post-termination employment or business non-compete covenants".into(),
            };
        }

        // Default: Exact substring match rule
        CompiledPolicyRule {
            raw_input: trimmed.to_string(),
            field: "text.content".into(),
            operator: PolicyOperator::Forbidden,
            value_num: None,
            value_str: Some(trimmed.to_string()),
            severity: RiskLevel::High,
            interpretation: format!("Flag any occurrence of keyword phrase '{}'", trimmed),
        }
    }

    /// Evaluates compiled policy rules against extracted contract facts and text
    pub fn evaluate(
        &self,
        rules: &[CompiledPolicyRule],
        ledger: &MoneyDateLedger,
        contract_text: &str,
    ) -> Vec<Finding> {
        let mut violations = Vec::new();
        let lower = contract_text.to_lowercase();

        for (i, rule) in rules.iter().enumerate() {
            match rule.field.as_str() {
                "termination.notice_days" => {
                    if let (Some(contract_notice), Some(target_notice)) =
                        (ledger.termination_notice_days, rule.value_num)
                    {
                        let violated = match rule.operator {
                            PolicyOperator::Gt => (contract_notice as i64) > target_notice,
                            PolicyOperator::Gte => (contract_notice as i64) >= target_notice,
                            PolicyOperator::Lt => (contract_notice as i64) < target_notice,
                            PolicyOperator::Lte => (contract_notice as i64) <= target_notice,
                            PolicyOperator::Eq => (contract_notice as i64) == target_notice,
                            PolicyOperator::Ne => (contract_notice as i64) != target_notice,
                            _ => false,
                        };

                        if violated {
                            violations.push(Finding {
                                id: format!("POLICY-VIOLATION-{}", i + 1),
                                rule_id: "VE-POLICY-NOTICE-001".into(),
                                category: "POLICY".into(),
                                severity: rule.severity,
                                confidence_pct: 98,
                                affected_party: "Signing Party".into(),
                                evidence_quote: format!("Contract specifies {} days termination notice", contract_notice),
                                related_clauses: vec!["Termination Clause".into()],
                                rationale_key: "policy.notice_period_breached".into(),
                                why_it_matters: format!(
                                    "Your playbook policy requires termination notice {:?} {} days, but contract requires {} days.",
                                    rule.operator, target_notice, contract_notice
                                ),
                                questions_to_ask: format!("Can the notice period be adjusted to {} days to comply with company playbook?", target_notice),
                                action_recommendation: format!("Replace notice period with {} days.", target_notice),
                            });
                        }
                    }
                }
                "contract.auto_renewal" => {
                    if ledger.auto_renewal {
                        violations.push(Finding {
                            id: format!("POLICY-VIOLATION-{}", i + 1),
                            rule_id: "VE-POLICY-AUTO-RENEWAL-002".into(),
                            category: "POLICY".into(),
                            severity: rule.severity,
                            confidence_pct: 96,
                            affected_party: "Signing Party".into(),
                            evidence_quote: "Agreement automatically renews for successive periods".into(),
                            related_clauses: vec!["Term & Renewal".into()],
                            rationale_key: "policy.auto_renewal_forbidden".into(),
                            why_it_matters: "Your playbook explicitly prohibits automatic renewals. Automatic rollover causes inadvertent budget commitments.".into(),
                            questions_to_ask: "Can this contract require mutual written renewal instead of automatic rollover?".into(),
                            action_recommendation: "Strike auto-renewal clause and substitute: 'This Agreement shall terminate at the end of the initial term unless explicitly renewed in writing.'".into(),
                        });
                    }
                }
                "commercial.exclusivity" => {
                    if lower.contains("exclusive") || lower.contains("sole provider") || lower.contains("exclusivity") {
                        violations.push(Finding {
                            id: format!("POLICY-VIOLATION-{}", i + 1),
                            rule_id: "VE-POLICY-EXCLUSIVITY-003".into(),
                            category: "POLICY".into(),
                            severity: rule.severity,
                            confidence_pct: 95,
                            affected_party: "Signing Party".into(),
                            evidence_quote: "Vendor shall act as exclusive provider / grant exclusive rights".into(),
                            related_clauses: vec!["Exclusivity".into()],
                            rationale_key: "policy.exclusivity_forbidden".into(),
                            why_it_matters: "Your playbook forbids exclusivity clauses, which legally block you from engaging alternative vendors or working with other clients.".into(),
                            questions_to_ask: "Can this agreement be made non-exclusive?".into(),
                            action_recommendation: "Replace with: 'This engagement is non-exclusive; Client remains free to engage other providers for similar services.'".into(),
                        });
                    }
                }
                "liability.uncapped" => {
                    if let Some(ref cap) = ledger.liability_cap {
                        if cap.contains("UNLIMITED") {
                            violations.push(Finding {
                                id: format!("POLICY-VIOLATION-{}", i + 1),
                                rule_id: "VE-POLICY-UNCAPPED-LIABILITY-004".into(),
                                category: "POLICY".into(),
                                severity: rule.severity,
                                confidence_pct: 99,
                                affected_party: "Signing Party".into(),
                                evidence_quote: "Liability shall not be limited / uncapped liability".into(),
                                related_clauses: vec!["Limitation of Liability".into()],
                                rationale_key: "policy.uncapped_liability_breached".into(),
                                why_it_matters: "Playbook rule strictly requires a finite liability cap. Uncapped liability exposes corporate balance sheet.".into(),
                                questions_to_ask: "Can liability be capped at the 12-month contract fees?".into(),
                                action_recommendation: "Insert mutual 12-month fee liability cap.".into(),
                            });
                        }
                    }
                }
                "statutory.non_compete" => {
                    if lower.contains("non-compete") || lower.contains("not compete") || lower.contains("restraint of trade") {
                        violations.push(Finding {
                            id: format!("POLICY-VIOLATION-{}", i + 1),
                            rule_id: "VE-POLICY-NON-COMPETE-005".into(),
                            category: "POLICY".into(),
                            severity: rule.severity,
                            confidence_pct: 99,
                            affected_party: "Employee / Consultant".into(),
                            evidence_quote: "shall not engage in competing business post-termination".into(),
                            related_clauses: vec!["Restrictive Covenants".into()],
                            rationale_key: "policy.non_compete_forbidden".into(),
                            why_it_matters: "Dealbreaker policy forbids non-compete clauses (which are also void under Section 27, ICA).".into(),
                            questions_to_ask: "Can this non-compete clause be deleted entirely?".into(),
                            action_recommendation: "Delete post-termination non-compete provision.".into(),
                        });
                    }
                }
                _ => {
                    // Raw substring check
                    if let Some(ref target) = rule.value_str {
                        if lower.contains(&target.to_lowercase()) {
                            violations.push(Finding {
                                id: format!("POLICY-VIOLATION-{}", i + 1),
                                rule_id: format!("VE-POLICY-CUSTOM-{}", i + 1),
                                category: "POLICY".into(),
                                severity: rule.severity,
                                confidence_pct: 90,
                                affected_party: "Signing Party".into(),
                                evidence_quote: format!("Matched custom playbook dealbreaker: '{}'", target),
                                related_clauses: vec!["Custom Policy Check".into()],
                                rationale_key: "policy.custom_keyword_matched".into(),
                                why_it_matters: format!("Matched user dealbreaker constraint: '{}'", rule.raw_input),
                                questions_to_ask: "Does counterparty agree to waive this restriction?".into(),
                                action_recommendation: "Negotiate removal or amendment of this clause.".into(),
                            });
                        }
                    }
                }
            }
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_notice_policy() {
        let compiler = PolicyCompiler::new();
        let rule = compiler.compile("termination notice > 30 days");
        assert_eq!(rule.field, "termination.notice_days");
        assert_eq!(rule.operator, PolicyOperator::Gt);
        assert_eq!(rule.value_num, Some(30));
    }

    #[test]
    fn test_evaluate_policy_violation() {
        let compiler = PolicyCompiler::new();
        let rule = compiler.compile("notice longer than 30 days");

        let mut ledger = MoneyDateLedger::default();
        ledger.termination_notice_days = Some(90);

        let violations = compiler.evaluate(&[rule], &ledger, "Notice of 90 days required.");
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "VE-POLICY-NOTICE-001");
    }
}
