// src/components/dict_search.rs
//
// Persistent navbar search. Reads the active language from context, searches
// the lexicon (L2 head form + L1 gloss), shows a results dropdown, and on
// selection opens a focused word card. Drop `DictSearch {}` anywhere beneath
// the lang context provider (the app root provides it).

use dioxus::prelude::*;

use crate::assets::freq_json_for;
use crate::models::lexicon::{LexEntry, Lexicon};

#[component]
pub fn DictSearch() -> Element {
    // Shared, app-wide current language (provided in `App`).
    let lang = use_context::<Signal<String>>();

    let mut query = use_signal(String::new);
    let mut open = use_signal(|| false); // results dropdown visible
    let mut selected = use_signal(|| None::<LexEntry>); // focused word card

    // Lexicon for the active language; reloads on switch.
    let lex_res = use_resource(move || {
        let l = lang.read().clone();
        async move { Lexicon::from_json(freq_json_for(&l)).unwrap_or_default() }
    });
    let lex = lex_res.read().clone().unwrap_or_default();

    // Reset when the language changes.
    use_effect(move || {
        let _ = lang();
        query.set(String::new());
        open.set(false);
        selected.set(None);
    });

    // palette (matches the dictionary's paperback surface)
    let paper = "#efe7d3";
    let ink = "#2a2622";
    let accent = "#6e2b2b";
    let secondary = "#8a7f6b";
    let rule = "#cbbfa3";
    let l2 = "'Noto Serif Georgian','Noto Serif',Georgia,'Times New Roman',serif";

    let q = query();
    let results: Vec<LexEntry> = if q.trim().is_empty() {
        Vec::new()
    } else {
        lex.search(&q).into_iter().take(8).collect()
    };
    let show_results = open() && !results.is_empty();

    // Prepare result rows so the template has no method calls.
    let rows: Vec<(u32, String, String, LexEntry)> = results
        .iter()
        .map(|e| (e.rank, e.head().to_string(), e.en.clone(), e.clone()))
        .collect();

    // Focused word-card values (precomputed for the template).
    let sel = selected();
    let show_card = sel.is_some();
    let card_head = sel
        .as_ref()
        .map(|e| e.head().to_string())
        .unwrap_or_default();
    let card_gloss = sel.as_ref().map(|e| e.en.clone()).unwrap_or_default();
    let card_rank = sel.as_ref().map(|e| e.rank).unwrap_or(0);
    let card_pos = sel.as_ref().and_then(|e| e.pos.clone());
    let card_example = sel.as_ref().and_then(|e| e.example.clone());

    rsx! {
        div { class: "relative",

            input {
                r#type: "text",
                value: "{q}",
                placeholder: "Search…",
                class: "px-3 py-1.5 rounded-md text-sm w-56 focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-700",
                style: "background:{paper}; color:{ink}; border:1px solid {rule}; font-family:{l2};",
                autocomplete: "off",
                spellcheck: "false",
                oninput: move |e| {
                    query.set(e.value());
                    open.set(true);
                },
                onfocus: move |_| {
                    open.set(true);
                },
            }

            if show_results {
                // click-away catcher
                div {
                    class: "fixed inset-0 z-40",
                    onclick: move |_| { open.set(false); },
                }
                // dropdown
                div {
                    class: "absolute left-0 z-50 mt-1 w-72 max-h-80 overflow-y-auto rounded-md shadow-xl",
                    style: "background:{paper}; border:1px solid {rule};",
                    for (rank, head, en, entry) in rows.iter().cloned() {
                        button {
                            key: "r{rank}",
                            class: "w-full text-left px-3 py-2 focus:outline-none",
                            style: "background:transparent; border:none; border-bottom:1px solid {rule}; cursor:pointer;",
                            onclick: move |_| {
                                selected.set(Some(entry.clone()));
                                open.set(false);
                            },
                            span { style: "font-family:{l2}; font-weight:700; color:{ink};", "{head}" }
                            span { style: "color:{secondary}; margin-left:0.5rem; font-size:0.85rem;", "{en}" }
                        }
                    }
                }
            }

            if show_card {
                // modal backdrop (click to dismiss)
                div {
                    class: "fixed inset-0 flex items-center justify-center p-4",
                    style: "background: rgba(0,0,0,0.45); z-index:60;",
                    onclick: move |_| { selected.set(None); },
                    // card (stop propagation so clicking it doesn't dismiss)
                    div {
                        class: "max-w-md w-full rounded-md shadow-2xl",
                        style: "background:{paper}; color:{ink}; font-family:{l2}; padding:1.75rem 1.9rem; border:1px solid {rule};",
                        onclick: move |e| { e.stop_propagation(); },

                        div {
                            class: "flex items-baseline justify-between",
                            style: "border-bottom:1.5px solid {accent}; padding-bottom:0.5rem;",
                            span { style: "font-size:1.7rem; font-weight:700;", "{card_head}" }
                            span { style: "font-size:0.7rem; color:{secondary};", "#{card_rank}" }
                        }
                        if let Some(p) = card_pos {
                            div { style: "font-style:italic; color:{secondary}; margin-top:0.6rem; font-size:0.85rem;", "{p}" }
                        }
                        div { style: "margin-top:0.4rem; font-size:1.05rem;", "{card_gloss}" }
                        if let Some(ex) = card_example {
                            div { style: "font-style:italic; color:{secondary}; margin-top:0.8rem;", "{ex}" }
                        }
                        div { class: "text-right mt-4",
                            button {
                                style: "background:none; border:none; color:{accent}; cursor:pointer; font-size:0.85rem;",
                                onclick: move |_| { selected.set(None); },
                                "Close"
                            }
                        }
                    }
                }
            }
        }
    }
}
