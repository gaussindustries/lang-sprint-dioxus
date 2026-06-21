use dioxus::prelude::*;

use crate::{
    assets::alphabet_json_for,
    components::{separator::Separator, Keyboard, TypingTest},
    models::letter::Letter,
    views::Alphabet,
};

#[component]
pub fn Home() -> Element {
    // Language is provided app-wide (and switched from the navbar).
    let lang = use_context::<Signal<String>>();
    let mut load_error = use_signal(|| None::<String>);

    // Re-parse the alphabet whenever the language changes.
    let letters = use_resource(move || {
        let lang_name = lang.read().clone();
        async move {
            match serde_json::from_str::<Vec<Letter>>(alphabet_json_for(&lang_name)) {
                Ok(vec) => vec,
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

    // NOTE: the fade-in now lives in the Navbar layout, so it replays on every
    // route change AND language switch — no per-page wrapper needed here.
    rsx! {
        div { class: "flex flex-col min-h-screen bg-gray-800 text-white",
            header { class: "bg-indigo-600 text-center p-3 space-y-2 flex justify-center gap-6 items-center",
                h1 { class: "text-3xl font-bold", "LangSprint" }
            }
            if let Some(err) = load_error() {
                div { class: "bg-red-900 text-red-200 px-4 py-2 text-sm text-center", "{err}" }
            }
            div { class: "shadow-inner",
                Alphabet { letters: letters_vec.clone(), lang }
            }
            div { class: "flex justify-center",
                div { class: "w-11/12",
                    Separator { horizontal: true }
                }
            }
            section { class: "flex justify-center",
                div { class: "mt-auto p-4 w-full shadow-xs",
                    h2 { class: "text-2xl font-semibold text-center", "Typing Test" }
                    Keyboard { letters: letters_vec.clone(),
                        TypingTest { lang, letters_vec: letters_vec.clone() }
                    }
                }
            }
        }
    }
}
