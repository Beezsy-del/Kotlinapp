use crate::models::{ContractFamily, MissingClause};

/// Missing-Clause Engine
/// Inspects contracts against baseline checklists tailored to specific contract families
#[derive(Debug, Clone, Default)]
pub struct MissingClauseEngine;

impl MissingClauseEngine {
    pub fn new() -> Self {
        Self
    }

    /// Detect contract family based on contract terminology
    pub fn detect_family(&self, text: &str) -> ContractFamily {
        let lower = text.to_lowercase();

        if lower.contains("non-disclosure") || lower.contains("confidentiality agreement") || lower.contains("proprietary information agreement") {
            ContractFamily::Nda
        } else if lower.contains("employment") || lower.contains("employee") || lower.contains("consultant") || lower.contains("probation") || lower.contains("salary") {
            ContractFamily::Employment
        } else if lower.contains("saas") || lower.contains("software as a service") || lower.contains("service level agreement") || lower.contains("cloud service") {
            ContractFamily::SaaS
        } else if lower.contains("services") || lower.contains("vendor") || lower.contains("contractor") || lower.contains("deliverables") || lower.contains("scope of work") {
            ContractFamily::ServicesVendor
        } else {
            ContractFamily::General
        }
    }

    /// Check for missing critical clauses for the detected contract family
    pub fn check_missing_clauses(&self, text: &str, family: ContractFamily) -> Vec<MissingClause> {
        let lower = text.to_lowercase();
        let mut missing = Vec::new();

        match family {
            ContractFamily::Nda => {
                if !lower.contains("exclusion") && !lower.contains("public domain") && !lower.contains("prior knowledge") {
                    missing.push(MissingClause {
                        name: "Standard Carve-outs / Exclusions to Confidential Information".into(),
                        importance: "High".into(),
                        rationale: "Without explicit exclusions (public domain, independently developed, compelled by law), information you already know could be claimed as confidential.".into(),
                        suggested_clause_snippet: "Exclusions: Confidential Information shall not include information that: (a) is or becomes publicly known through no breach; (b) was already in recipient's possession prior to disclosure; (c) is independently developed without reference to disclosed materials; or (d) is required to be disclosed by applicable law or court order.".into(),
                    });
                }
                if !lower.contains("return") && !lower.contains("destroy") && !lower.contains("destruction") {
                    missing.push(MissingClause {
                        name: "Return or Destruction of Confidential Information".into(),
                        importance: "Medium".into(),
                        rationale: "Lacks a clear obligation for the recipient to delete or return sensitive proprietary assets upon expiration.".into(),
                        suggested_clause_snippet: "Return of Materials: Upon written request or termination of this Agreement, Recipient shall promptly return or certify destruction of all documents and records containing Confidential Information.".into(),
                    });
                }
                if !lower.contains("governing law") && !lower.contains("jurisdiction") && !lower.contains("dispute") {
                    missing.push(MissingClause {
                        name: "Governing Law & Jurisdiction Clause".into(),
                        importance: "High".into(),
                        rationale: "Absence of a choice-of-law clause leaves forum and substantive law vulnerable to unpredictable multi-jurisdictional conflicts.".into(),
                        suggested_clause_snippet: "Governing Law: This Agreement shall be governed by and construed in accordance with the laws of India, and the courts at Bengaluru shall have exclusive jurisdiction.".into(),
                    });
                }
            }
            ContractFamily::Employment => {
                if !lower.contains("notice period") && !lower.contains("termination notice") {
                    missing.push(MissingClause {
                        name: "Mutual Termination Notice Period".into(),
                        importance: "High".into(),
                        rationale: "Lacks explicit notice duration for termination, leaving either party susceptible to immediate or arbitrary dismissal without transition pay.".into(),
                        suggested_clause_snippet: "Notice Period: Either party may terminate this employment upon thirty (30) days prior written notice or payment of basic salary in lieu thereof.".into(),
                    });
                }
                if !lower.contains("intellectual property") && !lower.contains("ip assignment") && !lower.contains("work for hire") {
                    missing.push(MissingClause {
                        name: "Clear Intellectual Property (IP) Ownership Boundary".into(),
                        importance: "High".into(),
                        rationale: "Lacks clear demarcation between employer work product and employee prior inventions/personal projects.".into(),
                        suggested_clause_snippet: "IP Rights: All intellectual property developed solely in connection with duties during working hours shall belong to Employer. Prior inventions and personal projects developed on personal time without company resources remain the exclusive property of Employee.".into(),
                    });
                }
            }
            ContractFamily::ServicesVendor => {
                if !lower.contains("acceptance") && !lower.contains("sign-off") && !lower.contains("review period") {
                    missing.push(MissingClause {
                        name: "Deliverable Acceptance Criteria & Review Window".into(),
                        importance: "High".into(),
                        rationale: "Without an objective acceptance procedure, the client can indefinitely delay payment by claiming dissatisfaction.".into(),
                        suggested_clause_snippet: "Acceptance: Client shall review deliverables within ten (10) business days. In the absence of written rejection detailing non-conformities within said period, deliverables shall be deemed accepted.".into(),
                    });
                }
                if !lower.contains("cure period") && !lower.contains("remedy period") && !lower.contains("opportunity to cure") {
                    missing.push(MissingClause {
                        name: "Breach Cure Period (Right to Remedy)".into(),
                        importance: "High".into(),
                        rationale: "No right to cure default, exposing you to immediate contract termination and damages for minor or inadvertent oversights.".into(),
                        suggested_clause_snippet: "Cure Period: In the event of an alleged material breach, the non-breaching party shall provide written notice specifying the default, and the breaching party shall have fifteen (15) days to cure such breach.".into(),
                    });
                }
                if !lower.contains("limitation of liability") && !lower.contains("liability shall not exceed") {
                    missing.push(MissingClause {
                        name: "Mutual Limitation of Liability Cap".into(),
                        importance: "High".into(),
                        rationale: "Entire agreement is silent on a liability cap, leaving commercial damages exposure boundless.".into(),
                        suggested_clause_snippet: "Limitation of Liability: In no event shall either party's aggregate liability under this Agreement exceed the total amounts paid or payable hereunder in the preceding twelve (12) months.".into(),
                    });
                }
            }
            ContractFamily::SaaS => {
                if !lower.contains("uptime") && !lower.contains("sla") && !lower.contains("service level") {
                    missing.push(MissingClause {
                        name: "Service Level Agreement (SLA) & Uptime Guarantee".into(),
                        importance: "High".into(),
                        rationale: "Lacks service availability commitment, meaning downtime occurs without any service credits or contractual remedy.".into(),
                        suggested_clause_snippet: "SLA: Provider guarantees 99.9% monthly platform uptime, excluding scheduled maintenance. Outages exceeding this threshold shall entitle Customer to pro-rata service fee credits.".into(),
                    });
                }
                if !lower.contains("data breach") && !lower.contains("security incident") {
                    missing.push(MissingClause {
                        name: "Data Breach Notification & Security Protocol".into(),
                        importance: "High".into(),
                        rationale: "No contractual deadline for provider to notify customer upon security compromises involving customer data.".into(),
                        suggested_clause_snippet: "Security Incident: Provider shall report any confirmed unauthorized access to Customer data within forty-eight (48) hours of discovery.".into(),
                    });
                }
            }
            ContractFamily::General => {
                if !lower.contains("severability") {
                    missing.push(MissingClause {
                        name: "Severability Clause".into(),
                        importance: "Medium".into(),
                        rationale: "If any clause (e.g. non-compete) is declared void by a court, absence of severability risks invalidating the entire agreement.".into(),
                        suggested_clause_snippet: "Severability: If any provision of this Agreement is held to be invalid or unenforceable, the remaining provisions shall continue in full force and effect.".into(),
                    });
                }
                if !lower.contains("force majeure") {
                    missing.push(MissingClause {
                        name: "Force Majeure Clause".into(),
                        importance: "Medium".into(),
                        rationale: "No relief mechanism if performance is rendered impossible due to acts of God, natural disasters, or government restrictions.".into(),
                        suggested_clause_snippet: "Force Majeure: Neither party shall be liable for delays resulting from causes beyond reasonable control, including acts of God, natural catastrophes, or war.".into(),
                    });
                }
            }
        }

        missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_contract_family() {
        let engine = MissingClauseEngine::new();
        assert_eq!(engine.detect_family("This Employment Agreement is entered into..."), ContractFamily::Employment);
        assert_eq!(engine.detect_family("MUTUAL NON-DISCLOSURE AND CONFIDENTIALITY AGREEMENT"), ContractFamily::Nda);
        assert_eq!(engine.detect_family("MASTER SERVICES AGREEMENT FOR DELIVERABLES"), ContractFamily::ServicesVendor);
    }

    #[test]
    fn test_detect_missing_nda_clauses() {
        let engine = MissingClauseEngine::new();
        let text = "Agreement between Party A and Party B to discuss secret projects.";
        let missing = engine.check_missing_clauses(text, ContractFamily::Nda);
        assert!(!missing.is_empty());
        assert!(missing.iter().any(|m| m.name.contains("Exclusions")));
    }
}
