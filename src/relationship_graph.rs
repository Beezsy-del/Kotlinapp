use crate::models::{Finding, Relation, RelationType, RiskLevel};
use regex::Regex;

/// Cross-Clause Relationship Graph Analyzer
/// Discovers inter-clause dependencies, carve-outs, exception overrides, and punctured liability caps
#[derive(Debug, Clone, Default)]
pub struct RelationshipGraph;

impl RelationshipGraph {
    pub fn new() -> Self {
        Self
    }

    /// Build relationship graph from contract text and discover stealth liabilities
    pub fn analyze(&self, text: &str) -> (Vec<Relation>, Vec<Finding>) {
        let mut relations = Vec::new();
        let mut stealth_findings = Vec::new();

        let lower = text.to_lowercase();

        // 1. Detect Punctured Liability Cap (Cap EXCLUDES Indemnity/IP => UNCAPS)
        self.detect_liability_cap_exceptions(text, &lower, &mut relations, &mut stealth_findings);

        // 2. Detect Survival Overrides (Confidentiality / Restraints survive termination indefinitely)
        self.detect_survival_overrides(text, &lower, &mut relations, &mut stealth_findings);

        // 3. Detect Immediate Termination Overrides (Termination for Cause OVERRIDES Cure Period)
        self.detect_termination_overrides(text, &lower, &mut relations, &mut stealth_findings);

        // 4. Detect General Cross-References ("Subject to Section X", "Notwithstanding Section Y")
        self.detect_cross_references(text, &mut relations);

        (relations, stealth_findings)
    }

    fn detect_liability_cap_exceptions(
        &self,
        text: &str,
        lower: &str,
        relations: &mut Vec<Relation>,
        findings: &mut Vec<Finding>,
    ) {
        // Pattern: Liability cap clause containing carve-outs for indemnity or confidentiality
        let has_cap = lower.contains("liability") && (lower.contains("cap") || lower.contains("shall not exceed") || lower.contains("aggregate liability"));
        let carveout_re = Regex::new(r"(?i)(?:except|excluding|other than|notwithstanding (?:anything to the contrary|the foregoing)|shall not apply to)\s*[^.;\n]{0,80}(?:indemn(?:ity|ification)|intellectual property|ip infringement|confidentiality|breach of section \d+)").ok();

        if has_cap {
            if let Some(re) = carveout_re {
                if let Some(m) = re.find(text) {
                    let quote = m.as_str().trim().to_string();

                    relations.push(Relation {
                        from_clause: "Limitation of Liability".into(),
                        relation_type: RelationType::Excludes,
                        to_clause: "Indemnification & Third-Party Claims".into(),
                        evidence_quote: quote.clone(),
                        reason: "Liability cap explicitly excludes indemnity obligations from protection.".into(),
                    });

                    relations.push(Relation {
                        from_clause: "Indemnification Clause".into(),
                        relation_type: RelationType::Uncaps,
                        to_clause: "Limitation of Liability".into(),
                        evidence_quote: quote.clone(),
                        reason: "Indemnity carves through the liability ceiling, creating uncapped dollar exposure.".into(),
                    });

                    findings.push(Finding {
                        id: format!("FINDING-GRAPH-{}", findings.len() + 1),
                        rule_id: "VE-GRAPH-PUNCTURED-CAP-001".into(),
                        category: "FINANCIAL".into(),
                        severity: RiskLevel::High,
                        confidence_pct: 94,
                        affected_party: "Signing Party".into(),
                        evidence_quote: quote,
                        related_clauses: vec!["Limitation of Liability".into(), "Indemnification".into()],
                        rationale_key: "liability.cap_carveout".into(),
                        why_it_matters: "A liability cap is illusory if indemnity claims are excluded. You remain exposed to unlimited financial liability.".into(),
                        questions_to_ask: "Can the indemnity obligations be brought inside the mutual liability cap?".into(),
                        action_recommendation: "Strike out the indemnity carve-out so the liability cap applies universally to all claims under the agreement.".into(),
                    });
                }
            }
        }
    }

    fn detect_survival_overrides(
        &self,
        text: &str,
        lower: &str,
        relations: &mut Vec<Relation>,
        findings: &mut Vec<Finding>,
    ) {
        if lower.contains("survive") && (lower.contains("termination") || lower.contains("expiration")) {
            let survival_re = Regex::new(r"(?i)(?:sections? \d+(?:[,\s]+and \d+)?|confidentiality|non-compete|indemnity)\s*shall survive\s*(?:indefinitely|in perpetuity|for a period of \d+ years|the termination)").ok();
            if let Some(re) = survival_re {
                if let Some(m) = re.find(text) {
                    let quote = m.as_str().trim().to_string();

                    relations.push(Relation {
                        from_clause: "Survival Provisions".into(),
                        relation_type: RelationType::Survives,
                        to_clause: "Agreement Termination".into(),
                        evidence_quote: quote.clone(),
                        reason: "Specific obligations survive post-termination.".into(),
                    });

                    if quote.to_lowercase().contains("indefinitely") || quote.to_lowercase().contains("perpetuity") {
                        findings.push(Finding {
                            id: format!("FINDING-GRAPH-{}", findings.len() + 1),
                            rule_id: "VE-GRAPH-PERPETUAL-SURVIVAL-002".into(),
                            category: "COMMERCIAL".into(),
                            severity: RiskLevel::Medium,
                            confidence_pct: 90,
                            affected_party: "Signing Party".into(),
                            evidence_quote: quote,
                            related_clauses: vec!["Term and Termination".into(), "Survival".into()],
                            rationale_key: "obligations.perpetual_survival".into(),
                            why_it_matters: "Perpetual survival of obligations creates indefinite legal exposure long after commercial relationship ends.".into(),
                            questions_to_ask: "Can we limit survival to a standard period (e.g. 2 or 3 years post-termination)?".into(),
                            action_recommendation: "Cap post-termination survival to 24–36 months maximum.".into(),
                        });
                    }
                }
            }
        }
    }

    fn detect_termination_overrides(
        &self,
        text: &str,
        lower: &str,
        relations: &mut Vec<Relation>,
        _findings: &mut Vec<Finding>,
    ) {
        if lower.contains("immediately without notice") || lower.contains("immediate termination without cure") {
            let imm_re = Regex::new(r"(?i)(?:terminate immediately|immediate termination)\s*(?:without notice|without opportunity to cure|upon written notice)").ok();
            if let Some(re) = imm_re {
                if let Some(m) = re.find(text) {
                    relations.push(Relation {
                        from_clause: "Immediate Termination for Cause".into(),
                        relation_type: RelationType::Overrides,
                        to_clause: "Standard Notice & Cure Period".into(),
                        evidence_quote: m.as_str().trim().to_string(),
                        reason: "Immediate termination right overrides contractual cure window.".into(),
                    });
                }
            }
        }
    }

    fn detect_cross_references(&self, text: &str, relations: &mut Vec<Relation>) {
        let ref_re = Regex::new(r"(?i)(?:subject to|pursuant to|notwithstanding|in accordance with)\s*(?:section|clause)\s*(\d+(?:\.\d+)?)").ok();
        if let Some(re) = ref_re {
            for caps in re.captures_iter(text).take(5) {
                if let Some(m) = caps.get(0) {
                    let sec = caps.get(1).map(|s| s.as_str()).unwrap_or("X");
                    relations.push(Relation {
                        from_clause: "Referencing Provision".into(),
                        relation_type: RelationType::References,
                        to_clause: format!("Section {}", sec),
                        evidence_quote: m.as_str().trim().to_string(),
                        reason: format!("Explicit cross-reference to Section {}", sec),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_punctured_liability_cap() {
        let graph = RelationshipGraph::new();
        let contract = r#"
            Clause 8. Limitation of Liability.
            The total aggregate liability of Vendor shall not exceed INR 2,00,000,
            except that nothing in this clause shall apply to indemnification obligations under Clause 12.
        "#;

        let (relations, findings) = graph.analyze(contract);
        assert!(!relations.is_empty());
        assert!(relations.iter().any(|r| r.relation_type == RelationType::Excludes || r.relation_type == RelationType::Uncaps));
        assert!(!findings.is_empty());
        assert_eq!(findings[0].rule_id, "VE-GRAPH-PUNCTURED-CAP-001");
    }
}
