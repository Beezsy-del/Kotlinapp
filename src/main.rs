use verdict_edge_rs::app::run_desktop_app;
use verdict_edge_rs::contract_engine::ContractEngine;
use verdict_edge_rs::history_store::HistoryStore;
use verdict_edge_rs::models::Language;
use verdict_edge_rs::phi35_engine::Phi35Engine;
use verdict_edge_rs::ui_input::SAMPLE_HIGH_RISK_CONTRACT;
use verdict_edge_rs::voice_engine::VoiceEngine;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // If passed `--cli`, run the terminal demo pipeline
    if args.iter().any(|arg| arg == "--cli") {
        run_cli_demo();
    } else {
        // Default: Launch native Dioxus Desktop GUI
        println!("🚀 Launching VerdictEdge (Rust + Dioxus Desktop + Microsoft Phi-3.5 Mini)...");
        run_desktop_app();
    }
}

fn run_cli_demo() {
    println!("============================================================");
    println!(" VerdictEdge (Rust + Dioxus + Microsoft Phi-3.5 Mini)");
    println!("============================================================");

    let engine = ContractEngine::new();
    let phi35 = Phi35Engine::new();
    let voice = VoiceEngine::new();
    let history = HistoryStore::new();

    let dealbreakers = vec![
        "non-compete".to_string(),
        "sole discretion".to_string(),
        "unlimited liability".to_string(),
    ];

    println!("Analyzing High-Risk Sample Agreement...");
    let result = engine.analyze_contract_with_llm(
        SAMPLE_HIGH_RISK_CONTRACT,
        &dealbreakers,
        &phi35,
        Language::English,
    );

    println!("Risk Level: {}", result.risk_level.label(Language::English));
    println!("Summary: {}", result.summary(Language::English));
    println!("Statutory Voidabilities Detected: {}", result.statutory_voidabilities.len());
    for sv in &result.statutory_voidabilities {
        println!(" - [{}] {}", sv.act_section, sv.title(Language::English));
    }

    println!("\nCounter-Clause Proposals: {}", result.clause_breakdowns.len());
    for cb in &result.clause_breakdowns {
        println!(" * {}", cb.counter_offer_draft);
    }

    println!("\nPre-signing Checklist Items: {}", result.pre_signing_checklist.len());

    let record = HistoryStore::record_from_analysis(SAMPLE_HIGH_RISK_CONTRACT, &result);
    let _ = history.save_record(record);
    println!("Persisted scan record into local history. Total scans: {}", history.load_all().len());

    println!("\nSpoken Audio Briefing (Normalized):");
    println!("{}", voice.build_audio_summary(&result, Language::English));

    if let Some(ref insight) = result.phi35_insight {
        println!("\nOn-Device Neural Assessment:\n{}", insight);
    }
}
