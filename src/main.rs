use verdict_edge_rs::contract_engine::ContractEngine;
use verdict_edge_rs::models::Language;
use verdict_edge_rs::phi35_engine::Phi35Engine;

fn main() {
    println!("============================================================");
    println!(" VerdictEdge (Rust + Dioxus + Microsoft Phi-3.5 Mini)");
    println!("============================================================");

    let engine = ContractEngine::new();
    let phi35 = Phi35Engine::new();

    println!("Phi-3.5 Status: {:?}", phi35.status());

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
    
    // Analyze with both Rule Engine and Microsoft Phi-3.5-mini
    let result = engine.analyze_contract_with_llm(
        sample_contract,
        &dealbreakers,
        &phi35,
        Language::English,
    );

    println!("Risk Level: {}", result.risk_level.label(Language::English));
    println!("Summary (EN): {}", result.summary(Language::English));
    println!("Statutory Voidabilities Detected: {}", result.statutory_voidabilities.len());
    for sv in &result.statutory_voidabilities {
        println!(" - [{}] {}: {}", sv.act_section, sv.title(Language::English), sv.legal_reason(Language::English));
    }

    if let Some(insight) = &result.phi35_insight {
        println!("\n{}", insight);
    }
}
