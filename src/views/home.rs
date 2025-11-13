// src/views/home.rs
use dioxus::prelude::*;
use std::fs;
use crate::components::{Keyboard, WordCard};

#[derive(serde::Deserialize, Clone, Debug)]
struct Letter {
    letter: String,
    name:   String,
    pron:   String,
    audio:  Option<String>,
    finger: String,
}

#[component]
pub fn Home() -> Element {
    // -------------------------------------------------------------
    // 1. Language (signal) – will be used later for switching
    // -------------------------------------------------------------
    let lang = use_signal(|| "georgian".to_string());

    // -------------------------------------------------------------
    // 2. Load alphabet.json with use_future
    // -------------------------------------------------------------
    let letters = use_resource(move || {
        let cur_lang = lang.read().clone();
        async move {
            let path = format!("langs/{}/alphabet.json", cur_lang);
            let raw = fs::read_to_string(&path).unwrap_or_else(|_| {
                fs::read_to_string("langs/georgian/alphabet.json").unwrap_or_default()
            });
            serde_json::from_str::<Vec<Letter>>(&raw).unwrap_or_default()
        }
    });

    // -------------------------------------------------------------
    // 3. Render
    // -------------------------------------------------------------
    rsx! {
        div { class: "flex flex-col min-h-screen bg-gray-800",

            // ── Header ────────────────────────────────────────
            header { class: "bg-indigo-600 text-white p-4 text-center",
                h1 { class: "text-3xl font-bold", "LangSprint – ქართული" }
            }

            // ── Alphabet Grid ─────────────────────────────────
            section { class: "p-6",
                h2 { class: "text-2xl font-semibold mb-4 text-center", "Alphabet - ანბანი" }
                div { class: "grid grid-cols-6 gap-4 max-w-3xl mx-auto",
                    {
                        // Clone the data *once* for the entire render
                        let data = letters.read().clone();  // <-- CRITICAL
                        match data {
                            Some(letters_vec) => {
                                letters_vec.into_iter().map(|letter| {
                                    let current_lang = lang.read().clone();
                                    rsx! {
                                        button {
                                            key: "{letter.letter}",
                                            class: "group p-4 rounded-lg border-2 hover:border-indigo-500 hover:cursor-help active:cursor-wait transition-all text-center",
                                            onclick: move |_| {
                                                if let Some(file) = &letter.audio {
                                                    let path = format!("langs/{}/pronunciation/alphabet/{}", current_lang, file);
                                                    // play_audio(&path);
                                                }
                                            },
                                            div { class: "text-3xl font-bold", "{letter.letter}" }
                                            div { class: "text-sm text-gray-600 italic", "{letter.name}" }
                                            div { class: "text-xs text-gray-500", "{letter.pron}" }
                                            div { class: "mt-1 text-xs font-medium text-indigo-600", "{letter.finger}" }
                                        }
                                    }
                                }).collect::<Vec<_>>().into_iter()
                            },
                            None => {
                                vec![rsx! {
                                    p { class: "col-span-6 text-center text-gray-500", "Loading…" }
                                }].into_iter()
                            }
                        }
                    }
                }
            }

            // ── Test preview ───────────────────────────────────
            section { class: "p-6 bg-black flex justify-center",
                div { 
					h2 { class: "text-2xl font-semibold mb-4", "Typing Test (preview)" }
					WordCard {
						word: "და",
						def: "and",
						pos: "conjunction",
						example: "მე და შენ"
					}
				}
            }

            // ── Keyboard ───────────────────────────────────────
            div { class: "mt-auto p-4 bg-gray-900 border-t",
                Keyboard { calibration: "None" }
            }
        }
    }
}