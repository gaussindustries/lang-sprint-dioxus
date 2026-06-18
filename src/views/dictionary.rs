// src/views/dictionary.rs
//
// The /dictionary route. Thin page that reads the shared language and hands it
// to the reusable `Dictionary` component (the paperback browser).

use dioxus::prelude::*;

use crate::components::Dictionary;

#[component]
pub fn DictionaryPage() -> Element {
    let lang = use_context::<Signal<String>>();

    rsx! {
        div { class: "min-h-screen bg-gray-800 text-white",
            Dictionary { lang }
        }
    }
}
