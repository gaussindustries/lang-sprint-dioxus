use dioxus::prelude::*;

#[component]
pub fn WordCard(
    word: String, 
    def: String, 
    pos: Option<String>, 
    example: Option<String>
) -> Element {

    let pos_text = pos.unwrap_or_else(|| "—".into());
    let example_text = example.unwrap_or_else(|| "".into());

    rsx!{
        div { class: "p-4 rounded-lg bg-gray-800 border border-gray-700 shadow-md w-full",
            // Main word (big & colored)
            div { class: "text-3xl font-semibold text-indigo-400 mb-1", 
                "{word}" 
            }

            // Definition + POS
            div { class: "text-sm text-gray-300 italic",
                "{def} ",
                span { class: "text-gray-500", "({pos_text})" }
            }

            // Example sentence (optional)
            {(example_text != "").then(|| rsx!{
                div { class: "mt-2 text-sm text-gray-400 italic", 
                    "e.g. {example_text}"
                }
            })}
        }
    }
}
