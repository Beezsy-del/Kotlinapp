use crate::models::{AnalysisResult, Language};
use crate::voice_engine::VoiceEngine;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ResultsScreenProps {
    pub result: AnalysisResult,
    pub selected_language: Signal<Language>,
    pub voice_engine: VoiceEngine,
    pub on_back: EventHandler<()>,
}

#[component]
pub fn ResultsScreen(props: ResultsScreenProps) -> Element {
    let mut font_size = use_signal(|| 14); // Dynamic font scaler (12px to 22px)
    let mut is_speaking = use_signal(|| false);
    let mut checklist_state = use_signal(|| props.result.pre_signing_checklist.clone());
    let mut copy_notification = use_signal(|| Option::<String>::None);

    let lang = *props.selected_language.read();
    let result = props.result.clone();
    let risk_color = result.risk_level.hex_color();
    let risk_bg = result.risk_level.bg_color();

    let voice_audio = props.voice_engine.clone();
    let voice_nav = props.voice_engine.clone();
    let result_audio = props.result.clone();
    let result_export = props.result.clone();
    let selected_lang_signal = props.selected_language;

    let toggle_audio = move |_| {
        if *is_speaking.read() {
            voice_audio.stop();
            is_speaking.set(false);
        } else {
            let summary = voice_audio.build_audio_summary(&result_audio, *selected_lang_signal.read());
            voice_audio.speak_async(&summary, *selected_lang_signal.read());
            is_speaking.set(true);
        }
    };

    let increase_font = move |_| {
        let current = *font_size.read();
        if current < 22 {
            font_size.set(current + 1);
        }
    };

    let decrease_font = move |_| {
        let current = *font_size.read();
        if current > 12 {
            font_size.set(current - 1);
        }
    };

    let handle_export_report = move |_| {
        let file = rfd::FileDialog::new()
            .set_file_name("VerdictEdge_Analysis_Report.md")
            .add_filter("Markdown Report", &["md"])
            .add_filter("Text Document", &["txt"])
            .save_file();

        if let Some(path) = file {
            let mut report = String::new();
            report.push_str("# VerdictEdge Statutory & Risk Analysis Report\n\n");
            report.push_str(&format!("**Risk Assessment:** {}\n", result_export.risk_level.label(Language::English)));
            report.push_str(&format!("**Summary:** {}\n\n", result_export.summary(Language::English)));

            if !result_export.statutory_voidabilities.is_empty() {
                report.push_str("## ⚖️ Statutory Voidability (Indian Contract Act 1872)\n");
                for sv in &result_export.statutory_voidabilities {
                    report.push_str(&format!("### [{}] {}\n", sv.act_section, sv.title_en));
                    report.push_str(&format!("* **Legal Basis:** {}\n", sv.legal_reason_en));
                    report.push_str(&format!("* **Snippet:** `\"{}\"`\n\n", sv.quote_snippet));
                }
            }

            if !result_export.clause_breakdowns.is_empty() {
                report.push_str("## 🛡️ Counter-Offers & Mitigation\n");
                for cb in &result_export.clause_breakdowns {
                    report.push_str(&format!("* **Vulnerability:** {}\n", cb.problem_en));
                    report.push_str(&format!("* **Solution:** {}\n", cb.solution_en));
                    report.push_str(&format!("* **Counter-Clause:** {}\n\n", cb.counter_offer_draft));
                }
            }

            if let Some(insight) = &result_export.phi35_insight {
                report.push_str("## 🤖 Microsoft Phi-3.5 Mini Legal Advisory\n");
                report.push_str(insight);
                report.push_str("\n\n");
            }

            let _ = std::fs::write(path, report);
            copy_notification.set(Some("Report exported successfully to file!".into()));
        }
    };

    rsx! {
        div {
            class: "results-container",
            style: "font-size: {font_size.read()}px;",

            // Navigation Bar
            div { class: "results-nav-bar",
                button {
                    class: "back-btn",
                    onclick: move |_| {
                        voice_nav.stop();
                        props.on_back.call(());
                    },
                    "← New Scan"
                }

                div { class: "utility-controls",
                    // Dynamic Font Sizing Scaler
                    div { class: "font-scaler",
                        button { class: "scaler-btn", onclick: decrease_font, "A-" }
                        span { class: "scaler-display", "{font_size.read()}px" }
                        button { class: "scaler-btn", onclick: increase_font, "A+" }
                    }

                    // Multilingual Audio Narrator Control
                    button {
                        class: if *is_speaking.read() { "audio-btn playing" } else { "audio-btn" },
                        onclick: toggle_audio,
                        if *is_speaking.read() { "⏹️ Stop Audio" } else { "🔊 Play Audio Briefing" }
                    }

                    // Export Report Button
                    button {
                        class: "export-btn",
                        onclick: handle_export_report,
                        "📥 Export Report"
                    }
                }
            }

            // Notification message
            if let Some(msg) = copy_notification.read().as_ref() {
                div { class: "status-banner",
                    span { "{msg}" }
                    button {
                        class: "banner-close",
                        onclick: move |_| copy_notification.set(None),
                        "✕"
                    }
                }
            }

            // Overall Risk Banner
            div {
                class: "risk-banner",
                style: "background: {risk_bg}; border-color: {risk_color};",
                div { class: "risk-badge-row",
                    span {
                        class: "risk-badge",
                        style: "background: {risk_color}; color: #FFFFFF;",
                        "{result.risk_level.label(lang)}"
                    }
                    span {
                        style: "color: {risk_color}; font-weight: 600; font-size: 14px;",
                        "{result.risk_level.description(lang)}"
                    }
                }
                p { class: "risk-summary-text", "{result.summary(lang)}" }
            }

            // Dealbreaker Matches (if any triggered)
            if !result.dealbreaker_matches.is_empty() {
                div { class: "analysis-card",
                    div { class: "card-title", style: "color: #FCD34D;", "⚡ Custom Dealbreakers Triggered" }
                    for db in &result.dealbreaker_matches {
                        div { class: "ambiguity-item",
                            span { class: "ambiguity-phrase", "Rule: \"{db.rule_keyword}\"" }
                            if !db.matched_context.is_empty() {
                                div { class: "quote-box", "\"{db.matched_context}\"" }
                            }
                        }
                    }
                }
            }

            // Statutory Voidability (Indian Contract Act 1872)
            if !result.statutory_voidabilities.is_empty() {
                div { class: "analysis-card",
                    div { class: "card-title", style: "color: #EF4444;", "⚖️ Statutory Voidability Warnings (Indian Contract Act)" }
                    for sv in &result.statutory_voidabilities {
                        div { class: "statutory-card",
                            div { class: "statutory-badge", "{sv.status}" }
                            div { class: "statutory-title", "[{sv.act_section}] {sv.title(lang)}" }
                            div { class: "statutory-reason", "{sv.legal_reason(lang)}" }
                            if !sv.quote_snippet.is_empty() {
                                div { class: "quote-box", "\"{sv.quote_snippet}\"" }
                            }
                        }
                    }
                }
            }

            // Microsoft Phi-3.5 Mini On-Device Analysis
            if let Some(insight) = &result.phi35_insight {
                div { class: "analysis-card phi35-card",
                    div { class: "card-title phi35-title", "🤖 Microsoft Phi-3.5 Mini On-Device Legal Advisory" }
                    div { class: "phi35-content", "{insight}" }
                }
            }

            // Key Vulnerabilities & Counter-Clause Proposals
            if !result.clause_breakdowns.is_empty() {
                div { class: "analysis-card",
                    div { class: "card-title", style: "color: #34D399;", "🛡️ Proposed Counter-Clauses & Negotiation Leverage" }
                    for (i, cb) in result.clause_breakdowns.iter().enumerate() {
                        div { class: "breakdown-item", key: "{i}",
                            div { class: "breakdown-problem", "⚠️ Risk: {cb.problem(lang)}" }
                            div { class: "breakdown-solution", "💡 Recommended Fix: {cb.solution(lang)}" }
                            if !cb.original_snippet.is_empty() {
                                div { class: "quote-box", "Original: \"{cb.original_snippet}\"" }
                            }
                            div { class: "counter-clause-box",
                                div { class: "counter-clause-header",
                                    span { class: "counter-clause-label", "Balanced Counter-Offer Draft" }
                                }
                                p { class: "counter-clause-text", "{cb.counter_offer_draft}" }
                            }
                        }
                    }
                }
            }

            // Interactive Pre-Signing Checklist
            if !checklist_state.read().is_empty() {
                div { class: "analysis-card",
                    div { class: "card-title", style: "color: #60A5FA;", "✅ Interactive Pre-Signing Due Diligence Checklist" }
                    for (idx, item) in checklist_state.read().iter().enumerate() {
                        div {
                            class: "checklist-item",
                            key: "{item.id}",
                            onclick: move |_| {
                                let mut list = checklist_state.write();
                                list[idx].is_resolved = !list[idx].is_resolved;
                            },
                            input {
                                r#type: "checkbox",
                                class: "checklist-checkbox",
                                checked: item.is_resolved,
                            }
                            span {
                                class: if item.is_resolved { "checklist-task resolved" } else { "checklist-task" },
                                "{item.task(lang)}"
                            }
                        }
                    }
                }
            }

            // Discretion & Ambiguity Detector
            if !result.ambiguities.is_empty() {
                div { class: "analysis-card",
                    div { class: "card-title", style: "color: #FBBF24;", "🔍 Discretionary & Ambiguous Terms" }
                    for amb in &result.ambiguities {
                        div { class: "ambiguity-item",
                            span { class: "ambiguity-phrase", "{amb.phrase}" }
                            span { class: "ambiguity-exp", "{amb.explanation(lang)}" }
                            if !amb.quote_snippet.is_empty() {
                                div { class: "quote-box", "\"{amb.quote_snippet}\"" }
                            }
                        }
                    }
                }
            }

            // Deadlines & Financial Exposures Grid
            if !result.deadlines.is_empty() || !result.financial_exposures.is_empty() {
                div { class: "analysis-card",
                    div { class: "card-title", "⏱️ Extracted Deadlines & Financial Obligations" }
                    div { class: "metrics-grid",
                        for dl in &result.deadlines {
                            div { class: "metric-pill",
                                span { class: "metric-value", "{dl.timeframe}" }
                                span { class: "metric-label", "{dl.obligation(lang)}" }
                            }
                        }
                        for fin in &result.financial_exposures {
                            div { class: "metric-pill",
                                span { class: "metric-value", "{fin.amount_or_cost}" }
                                span { class: "metric-label", "{fin.title(lang)}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
