use dioxus::prelude::*;

use crate::components::{Grammar, GrammarQuiz};

/// Route wrapper for grammar. Reads the active language from context and offers
/// two modes over the SAME content doc: read the notes, or drill the authored
/// questions. Both resolve via `crate::grammar::doc_for`.
#[component]
pub fn GrammarPage() -> Element {
    let active = use_context::<Signal<String>>();
    let lang = active();
    let mut practice = use_signal(|| false);
    let practicing = practice();

    let tab = |on: bool| -> &'static str {
        if on {
            "px-3 py-1.5 rounded text-sm bg-indigo-600 text-white"
        } else {
            "px-3 py-1.5 rounded text-sm bg-gray-700 text-gray-300"
        }
    };

    rsx! {
        div { class: "min-h-screen bg-gray-800 text-white p-8",
            h1 { style: "font-size:1.5rem; font-weight:600; text-align:center; margin-bottom:1rem;", "Grammar" }

            div { class: "flex justify-center gap-2 mb-6",
                button {
                    class: tab(!practicing),
                    onclick: move |_| practice.set(false),
                    "Learn"
                }
                button {
                    class: tab(practicing),
                    onclick: move |_| practice.set(true),
                    "Practice"
                }
            }

            if practicing {
                // keyed on language so switching resets the quiz state
                GrammarQuiz { key: "{lang}", lang: lang.clone() }
            } else {
                Grammar { lang: lang.clone() }
            }
        }
    }
}
