use crate::document_importer::DocumentImporter;
use crate::models::Language;
use dioxus::prelude::*;

/// Preloaded high-risk sample contract for instant testing
pub const SAMPLE_HIGH_RISK_CONTRACT: &str = r#"MASTER CONSULTING SERVICES AGREEMENT
Between TechCorp India Pvt Ltd ("Client") and Consultant ("Vendor").

1. FEES & PAYMENT:
The total consulting fee shall be INR 1,50,000 payable within 45 days of receipt of invoice. Late payments shall incur a compounded penalty fee of 3% per month.

2. TERM & TERMINATION:
This Agreement shall commence on the Effective Date and shall automatically renew for successive 1-year terms unless cancelled. Client may terminate this Agreement at any time at its sole discretion upon 7 days prior written notice.

3. RESTRAINT OF TRADE (POST-EMPLOYMENT NON-COMPETE):
Consultant agrees that during the term and for a period of twenty-four (24) months following termination, Consultant shall not engage, directly or indirectly, in any business, trade, or employment that competes with Client.

4. INDEMNIFICATION & LIABILITIES:
Consultant shall indemnify, defend, and hold harmless Client from and against any and all claims, damages, liabilities, costs, and legal fees. Consultant's liability under this Agreement shall be unlimited.

5. INTELLECTUAL PROPERTY:
Consultant hereby assigns to Client all right, title, and interest in and to all intellectual property and inventions conceived from time to time, as deemed fit by Client.

6. GOVERNING LAW & JURISDICTION:
This Agreement shall be governed by exclusive foreign arbitration, and Consultant waives all rights to approach courts in India."#;

#[derive(Props, Clone, PartialEq)]
pub struct InputScreenProps {
    pub contract_text: Signal<String>,
    pub custom_dealbreakers: Signal<Vec<String>>,
    pub selected_language: Signal<Language>,
    pub on_analyze: EventHandler<()>,
    pub on_open_history: EventHandler<()>,
}

#[component]
pub fn InputScreen(props: InputScreenProps) -> Element {
    let mut new_dealbreaker_input = use_signal(String::new);
    let mut status_banner = use_signal(|| String::new());

    let mut contract_text = props.contract_text;
    let mut custom_dealbreakers = props.custom_dealbreakers;
    let mut selected_language = props.selected_language;

    let mut handle_add_dealbreaker = move || {
        let val = new_dealbreaker_input.read().trim().to_string();
        if !val.is_empty() && !custom_dealbreakers.read().contains(&val) {
            custom_dealbreakers.write().push(val);
            new_dealbreaker_input.set(String::new());
        }
    };

    let handle_load_sample = move |_| {
        contract_text.set(SAMPLE_HIGH_RISK_CONTRACT.to_string());
        status_banner.set("Loaded sample high-risk consulting agreement.".to_string());
    };

    let handle_clear_text = move |_| {
        contract_text.set(String::new());
        status_banner.set("Editor cleared.".to_string());
    };

    let handle_file_upload = move |_| {
        let file = rfd::FileDialog::new()
            .add_filter("Legal Documents", &["pdf", "txt", "md", "png", "jpg", "jpeg"])
            .pick_file();

        if let Some(path) = file {
            let importer = DocumentImporter::new();
            match importer.extract_text(&path) {
                Ok(content) => {
                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("document");
                    contract_text.set(content);
                    status_banner.set(format!("Successfully imported: {}", file_name));
                }
                Err(err) => {
                    status_banner.set(format!("Import error: {}", err));
                }
            }
        }
    };

    rsx! {
        div { class: "input-container",
            // Header / App Branding
            header { class: "app-header",
                div { class: "brand-group",
                    div { class: "app-logo", "⚖️" }
                    div {
                        h1 { class: "app-title", "VerdictEdge" }
                        p { class: "app-tagline", "Air-Gapped, On-Device Legal Risk & Statutory Intelligence" }
                    }
                }

                div { class: "header-actions",
                    // Language Switcher
                    div { class: "lang-switcher",
                        button {
                            class: if *selected_language.read() == Language::English { "lang-btn active" } else { "lang-btn" },
                            onclick: move |_| selected_language.set(Language::English),
                            "English"
                        }
                        button {
                            class: if *selected_language.read() == Language::Hindi { "lang-btn active" } else { "lang-btn" },
                            onclick: move |_| selected_language.set(Language::Hindi),
                            "हिंदी"
                        }
                        button {
                            class: if *selected_language.read() == Language::Kannada { "lang-btn active" } else { "lang-btn" },
                            onclick: move |_| selected_language.set(Language::Kannada),
                            "ಕನ್ನಡ"
                        }
                    }

                    // History Drawer Trigger
                    button {
                        class: "history-btn",
                        onclick: move |_| props.on_open_history.call(()),
                        "📂 Scan History"
                    }
                }
            }

            // Notification / Status Banner
            if !status_banner.read().is_empty() {
                div { class: "status-banner",
                    span { "{status_banner.read()}" }
                    button {
                        class: "banner-close",
                        onclick: move |_| status_banner.set(String::new()),
                        "✕"
                    }
                }
            }

            // Main Editor Section
            main { class: "editor-section",
                div { class: "editor-header-bar",
                    span { class: "editor-label", "Contract Document Text" }
                    div { class: "editor-quick-actions",
                        button {
                            class: "action-pill",
                            onclick: handle_load_sample,
                            "📋 Load High-Risk Sample"
                        }
                        button {
                            class: "action-pill",
                            onclick: handle_file_upload,
                            "📁 Upload PDF / Doc"
                        }
                        button {
                            class: "action-pill text-muted",
                            onclick: handle_clear_text,
                            "🗑️ Clear"
                        }
                    }
                }

                textarea {
                    class: "contract-textarea",
                    placeholder: "Paste employment contract, NDA, freelance agreement, master service agreement (MSA), or lease terms here...",
                    value: "{contract_text.read()}",
                    oninput: move |evt| contract_text.set(evt.value().clone()),
                }

                // Custom Dealbreakers Section
                div { class: "dealbreaker-box",
                    div { class: "dealbreaker-header",
                        span { class: "dealbreaker-title", "⚡ Custom Dealbreaker Keywords" }
                        span { class: "dealbreaker-desc", "Flags clauses containing user-specific dealbreakers (e.g. 60 days notice, non-compete)" }
                    }

                    // Dealbreaker Chips
                    div { class: "dealbreaker-chips",
                        for (idx, keyword) in custom_dealbreakers.read().iter().enumerate() {
                            div { class: "chip", key: "{idx}",
                                span { "{keyword}" }
                                button {
                                    class: "chip-remove",
                                    onclick: move |_| {
                                        custom_dealbreakers.write().remove(idx);
                                    },
                                    "✕"
                                }
                            }
                        }
                    }

                    // Add Dealbreaker Input
                    div { class: "dealbreaker-input-row",
                        input {
                            class: "dealbreaker-input",
                            placeholder: "Add dealbreaker keyword (e.g., 'unilateral', 'arbitration outside india')...",
                            value: "{new_dealbreaker_input.read()}",
                            oninput: move |evt| new_dealbreaker_input.set(evt.value().clone()),
                            onkeydown: move |evt| {
                                if evt.key() == Key::Enter {
                                    handle_add_dealbreaker();
                                }
                            }
                        }
                        button {
                            class: "add-rule-btn",
                            onclick: move |_| handle_add_dealbreaker(),
                            "+ Add Rule"
                        }
                    }
                }

                // Primary Analyze CTA
                div { class: "analyze-cta-row",
                    button {
                        class: "primary-analyze-btn",
                        disabled: contract_text.read().trim().is_empty(),
                        onclick: move |_| props.on_analyze.call(()),
                        "⚡ Analyze Contract with VerdictEdge & Microsoft Phi-3.5 Mini"
                    }
                }
            }
        }
    }
}
