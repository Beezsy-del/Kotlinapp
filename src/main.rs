use verdict_edge_rs::contract_engine::ContractEngine;
use verdict_edge_rs::models::Language;
use verdict_edge_rs::phi35_engine::Phi35Engine;
use verdict_edge_rs::voice_engine::VoiceEngine;

fn main() {
    println!("============================================================");
    println!(" VerdictEdge (Rust + Dioxus + Microsoft Phi-3.5 Mini)");
    println!("============================================================");

    let engine = ContractEngine::new();
    let phi35 = Phi35Engine::new();
    let voice = VoiceEngine::new();

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
    let result = engine.analyze_contract_with_llm(
        sample_contract,
        &dealbreakers,
        &phi35,
        Language::English,
    );

    println!("Risk Level: {}", result.risk_level.label(Language::English));
    println!("Statutory Voidabilities Detected: {}", result.statutory_voidabilities.len());

    println!("\n--- 🎙️ Multilingual Spoken Briefings (Voice Normalization) ---");
    println!("[EN Briefing]:\n{}\n", voice.build_audio_summary(&result, Language::English));
    println!("[HI Briefing]:\n{}\n", voice.build_audio_summary(&result, Language::Hindi));
    println!("[KN Briefing]:\n{}\n", voice.build_audio_summary(&result, Language::Kannada));
}
