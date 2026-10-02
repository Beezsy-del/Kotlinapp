use crate::contract_engine::ContractEngine;
use crate::history_store::HistoryStore;
use crate::models::{AnalysisResult, HistoryRecord, Language};
use crate::phi35_engine::Phi35Engine;
use crate::ui_input::InputScreen;
use crate::ui_results::ResultsScreen;
use crate::voice_engine::VoiceEngine;
use dioxus::prelude::*;

const EMBEDDED_CSS: &str = include_str!("style.css");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Input,
    Results,
}

#[component]
pub fn App() -> Element {
    let mut app_state = use_signal(|| AppState::Input);
    let mut contract_text = use_signal(String::new);
    let custom_dealbreakers = use_signal(|| vec![
        "non-compete".to_string(),
        "sole discretion".to_string(),
        "unlimited liability".to_string(),
    ]);
    let selected_language = use_signal(|| Language::English);
    let mut current_result = use_signal(|| Option::<AnalysisResult>::None);
    let mut show_history = use_signal(|| false);
    let mut history_records = use_signal(|| Vec::<HistoryRecord>::new());

    let voice_engine = use_signal(|| VoiceEngine::new());
    let contract_engine = use_signal(|| ContractEngine::new());
    let phi35_engine = use_signal(|| Phi35Engine::new());
    let history_store = use_signal(|| HistoryStore::new());

    let mut handle_analyze = move |_| {
        let text = contract_text.read().clone();
        if text.trim().is_empty() {
            return;
        }

        let rules = custom_dealbreakers.read().clone();
        let lang = *selected_language.read();

        // Run full analysis with Rule Engine + Microsoft Phi-3.5 Mini
        let res = contract_engine.read().analyze_contract_with_llm(
            &text,
            &rules,
            &phi35_engine.read(),
            lang,
        );

        // Save into local persistent history
        if !res.is_invalid {
            let record = HistoryStore::record_from_analysis(&text, &res);
            let _ = history_store.read().save_record(record);
        }

        current_result.set(Some(res));
        app_state.set(AppState::Results);
    };

    let handle_open_history = move |_| {
        let loaded = history_store.read().load_all();
        history_records.set(loaded);
        show_history.set(true);
    };

    rsx! {
        // Embed theme stylesheet
        style { "{EMBEDDED_CSS}" }

        div { class: "app-root",
            match *app_state.read() {
                AppState::Input => rsx! {
                    InputScreen {
                        contract_text,
                        custom_dealbreakers,
                        selected_language,
                        on_analyze: handle_analyze,
                        on_open_history: handle_open_history,
                    }
                },
                AppState::Results => {
                    if let Some(res) = current_result.read().as_ref() {
                        rsx! {
                            ResultsScreen {
                                result: res.clone(),
                                selected_language,
                                voice_engine: voice_engine.read().clone(),
                                on_back: move |_| app_state.set(AppState::Input),
                            }
                        }
                    } else {
                        rsx! { div { "No analysis results." } }
                    }
                }
            }

            // History Modal Drawer
            if *show_history.read() {
                div {
                    class: "history-modal-overlay",
                    onclick: move |_| show_history.set(false),
                    div {
                        class: "history-drawer",
                        onclick: move |e| e.stop_propagation(),
                        div { class: "drawer-header",
                            span { class: "drawer-title", "📂 Offline Scan History" }
                            button {
                                class: "banner-close",
                                onclick: move |_| show_history.set(false),
                                "✕"
                            }
                        }

                        div { class: "history-list",
                            if history_records.read().is_empty() {
                                p { style: "color: var(--text-muted); font-size: 13px;", "No contracts analyzed yet." }
                            } else {
                                for rec in history_records.read().iter() {
                                    div {
                                        class: "history-card-item",
                                        key: "{rec.id}",
                                        onclick: {
                                            let full = rec.full_text.clone();
                                            move |_| {
                                                contract_text.set(full.clone());
                                                show_history.set(false);
                                                handle_analyze(());
                                            }
                                        },
                                        div { class: "history-item-top",
                                            span {
                                                class: "risk-badge",
                                                style: "background: {rec.risk_level.hex_color()}; color: #FFFFFF; font-size: 10px; padding: 2px 6px;",
                                                "{rec.risk_level.label(*selected_language.read())}"
                                            }
                                            span { class: "history-item-date", "{rec.formatted_date}" }
                                        }
                                        div { class: "history-item-title", "{rec.title}" }
                                        div { class: "history-item-snippet", "{rec.snippet}" }
                                        div { class: "history-item-actions",
                                            span { style: "color: var(--primary); font-size: 12px;", "Click to re-analyze →" }
                                            button {
                                                class: "history-delete-btn",
                                                onclick: {
                                                    let id = rec.id.clone();
                                                    move |e| {
                                                        e.stop_propagation();
                                                        let _ = history_store.read().delete_record(&id);
                                                        let reloaded = history_store.read().load_all();
                                                        history_records.set(reloaded);
                                                    }
                                                },
                                                "Delete"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Entrypoint to run VerdictEdge as a native desktop application
pub fn run_desktop_app() {
    let window = dioxus::desktop::WindowBuilder::new()
        .with_title("VerdictEdge — Air-Gapped Legal Risk & Statutory Intelligence");

    let _config = dioxus::desktop::Config::default()
        .with_window(window);

    dioxus::desktop::launch::launch(App, vec![], vec![]);
}
