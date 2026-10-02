use verdict_edge_rs::models::*;

fn main() {
    println!("=== VerdictEdge Rust (Dioxus + Phi-3.5 Mini) ===");
    println!("Version 0.1.0 Initialized");
    println!("Supported Languages: English, Hindi (हिंदी), Kannada (ಕನ್ನಡ)");
    println!("Risk Levels: {:?}, {:?}, {:?}, {:?}",
        RiskLevel::Low.label(Language::English),
        RiskLevel::Medium.label(Language::English),
        RiskLevel::High.label(Language::English),
        RiskLevel::Invalid.label(Language::English),
    );
}
