use verdict_edge_rs::contract_engine::ContractEngine;
use verdict_edge_rs::models::Language;

fn main() {
    println!("============================================================");
    println!(" VerdictEdge (Rust + Dioxus + Microsoft Phi-3.5 Mini)");
    println!("============================================================");

    let engine = ContractEngine::new();
    let sample_contract = r#"
        CONSULTING SERVICES AGREEMENT
        Section 3. Payment: Total fee shall be INR 75,000 payable within 30 days of invoice.
        Late payments shall incur a penalty fee of 2% per month.
        Section 5. Termination: Client may terminate this agreement at its sole discretion upon 14 days notice.
        Section 8. Restraint of Trade: The Consultant agrees to a post-employment non-compete restriction
        prohibiting any competitive software work for a period of 12 months.
        Section 10. Indemnification: Consultant shall indemnify and hold harmless the Client against any and all claims.
    "#;

    let dealbreakers = vec!["sole discretion".to_string()];
    let result = engine.analyze_contract(sample_contract, &dealbreakers);

    println!("Risk Level: {}", result.risk_level.label(Language::English));
    println!("Summary (EN): {}", result.summary(Language::English));
    println!("Summary (HI): {}", result.summary(Language::Hindi));
    println!("Summary (KN): {}", result.summary(Language::Kannada));
    println!("Statutory Voidabilities Detected: {}", result.statutory_voidabilities.len());
    for sv in &result.statutory_voidabilities {
        println!(" - [{}] {}: {}", sv.act_section, sv.title(Language::English), sv.legal_reason(Language::English));
    }
    println!("Vulnerabilities / Counter-Offers: {}", result.clause_breakdowns.len());
    for cb in &result.clause_breakdowns {
        println!(" - Vulnerability: {}", cb.problem(Language::English));
        println!("   {}", cb.counter_offer_draft);
    }
    println!("Discretion / Ambiguities: {}", result.ambiguities.len());
    println!("Deadlines: {}", result.deadlines.len());
    println!("Financial Exposures: {}", result.financial_exposures.len());
    println!("Pre-signing Checklist Items: {}", result.pre_signing_checklist.len());
}
