use dioxus::prelude::*;
use crate::models::letter::Letter;

#[component]
pub fn Alphabet(letters: Vec<Letter>, lang: Signal<String>) -> Element {
    // Read language once per render
    let current_lang = lang.read().clone();
    let is_empty = letters.is_empty();

    rsx! {
        section { class: "p-6",
            h2 { class: "text-2xl font-semibold mb-4 text-center", "Alphabet - ანბანი" }

            div { class: "grid grid-cols-6 gap-4 max-w-3xl mx-auto",
                // If letters_vec is empty, show a loading message
                if is_empty {
                    p { class: "col-span-6 text-center text-gray-500", "Loading…" }
                } else {
                    {letters.iter().cloned().map(|letter| {
                        let lang_for_click = current_lang.clone();

                        rsx! {
                            button {
                                key: "{letter.letter}",
                                class: "group p-4 rounded-lg border-2 hover:border-indigo-500 \
                                        hover:cursor-help active:cursor-wait transition-all text-center",

                                onclick: move |_| {
                                    if let Some(file) = &letter.audio {
                                        let path = format!(
                                            "langs/{}/pronunciation/alphabet/{}",
                                            lang_for_click,
                                            file
                                        );
                                        // TODO: hook up audio playback here
                                        // play_audio(&path);
                                    }
                                },

                                div { class: "text-3xl font-bold", "{letter.letter}" }
                                div { class: "text-sm text-gray-300 italic", "{letter.name}" }
                                div { class: "text-xs text-gray-400", "{letter.pron}" }
                            }
                        }
                    })}
                }
            }
        }
    }
}
