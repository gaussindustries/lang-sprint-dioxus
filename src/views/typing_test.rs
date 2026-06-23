use dioxus::prelude::*;

use crate::{
    assets::alphabet_json_for,
    components::{Keyboard, TypingTest},
    models::letter::Letter,
};

/// Route wrapper for the typing test. Loads the active language's letters (the
/// keyboard + per-key hints need them) and renders the on-screen keyboard
/// wrapping the typing drill.
#[component]
pub fn TypingPage() -> Element {
    let lang = use_context::<Signal<String>>();
    let mut load_error = use_signal(|| None::<String>);

    let letters = use_resource(move || {
        let lang_name = lang.read().clone();
        async move {
            match serde_json::from_str::<Vec<Letter>>(alphabet_json_for(&lang_name)) {
                Ok(v) => v,
                Err(e) => {
                    let msg = format!("Failed to parse alphabet for {lang_name}: {e}");
                    eprintln!("{msg}");
                    load_error.set(Some(msg));
                    Vec::new()
                }
            }
        }
    });
    let letters_vec = letters.read().clone().unwrap_or_default();

    rsx! {
        div { class: "min-h-screen bg-gray-800 text-white",
            if let Some(err) = load_error() {
                div { class: "bg-red-900 text-red-200 px-4 py-2 text-sm text-center", "{err}" }
            }
            section { class: "flex justify-center",
                div { class: "p-4 w-full",
                    h2 { class: "text-2xl font-semibold text-center mb-2", "Typing Test" }
                    Keyboard { letters: letters_vec.clone(),
                        TypingTest { lang, letters_vec: letters_vec.clone() }
                    }
                }
            }
        }
    }
}
