// src/views/home.rs
use dioxus::prelude::*;
use std::fs;
use crate::{components::{TypingTest, Keyboard}, models::letter::Letter, views::Alphabet};

pub fn Home() -> Element {
    let lang = use_signal(|| "georgian".to_string());

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

    let letters_vec = letters.read().clone().unwrap_or_default();

    rsx! {
        div { class: "flex flex-col min-h-screen bg-gray-800",

            header { class: "bg-indigo-600 text-white p-4 text-center",
                h1 { class: "text-3xl font-bold", "LangSprint – ქართული" }
            }

            Alphabet { letters: letters_vec.clone(), lang: lang.clone() }

            section { class: "p-6 flex justify-center",
                div {
                    h2 { class: "text-2xl font-semibold mb-4 text-center", "Typing Test" }
                }
            }

            // ⬇️ Keyboard wraps TypingTest so it captures key events
            div { class: "mt-auto p-4 bg-gray-900 border-t",
                Keyboard { letters: letters_vec.clone(),
                    TypingTest { lang: lang.clone(), letters_vec: letters_vec.clone() }
                }
            }
        }
    }
}