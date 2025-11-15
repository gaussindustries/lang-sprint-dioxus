// src/views/home.rs
use dioxus::prelude::*;
use std::fs;
use crate::{views::Alphabet, components::{ Keyboard }, models::letter::Letter};

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
	let letters_vec = letters.read().clone().unwrap_or_default();
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
			Alphabet { letters: letters_vec.clone() , lang: lang.clone()}

            // ── Keyboard ───────────────────────────────────────
            div { class: "mt-auto p-4 bg-gray-900 border-t",
                Keyboard { letters: letters_vec.clone() }
            }
        }
    }
}