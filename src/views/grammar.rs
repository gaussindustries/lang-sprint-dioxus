use dioxus::prelude::*;

use crate::components::Grammar;

/// Route wrapper for grammar. Reads the active language from context and hands
/// it to the `Grammar` renderer, which resolves the right content doc.
#[component]
pub fn GrammarPage() -> Element {
    let active = use_context::<Signal<String>>();
    let lang = active();

    rsx! {
        div { class: "min-h-screen bg-gray-800 text-white p-8",
            h1 { style: "font-size:1.5rem; font-weight:600; text-align:center; margin-bottom:1.5rem;", "Grammar" }
            Grammar { lang }
        }
    }
}
