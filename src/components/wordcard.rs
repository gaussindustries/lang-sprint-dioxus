use dioxus::prelude::*;

#[component]
pub fn WordCard(word: String, def: String, pos: String, example: String) -> Element {
	
	rsx!{
        div { class: "p-4 rounded-lg bg-gray-800 border",
            div { class: "text-2xl font-bold text-indigo-800", "{word}" }
            div { class: "text-sm text-gray-600", "{def} ({pos})" }
            div { class: "mt-2 text-sm italic", "e.g. {example}" }
        }
    }
}