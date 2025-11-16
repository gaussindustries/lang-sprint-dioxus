// src/views/alphabet.rs
use dioxus::prelude::*;
use crate::{audio, models::letter::Letter};

#[component]
pub fn Alphabet(letters: Vec<Letter>, lang: Signal<String>) -> Element {
    // Status string to show success / error messages from audio.rs
    let mut audio_status = use_signal(|| None::<String>);

    // Cards iterator
    let cards = letters.into_iter().map(move |letter| {
        let lang = lang.clone();
        let letter_for_click = letter.clone();
        let mut status = audio_status.clone();

        rsx! {
            button {
                key: "{letter.letter}",
                class: "group p-4 rounded-lg border-2 hover:border-indigo-500 \
                        hover:cursor-pointer active:cursor-wait transition-all text-center",
                onclick: move |_| {
                    if let Some(file) = &letter_for_click.audio {
                        let lang_name = lang.read().clone();
                        let path = format!(
                            "langs/{}/pronunciation/alphabet/{}",
                            lang_name,
                            file
                        );

                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            match audio::play_audio(&path) {
                                Ok(()) => {
                                    status.set(Some(format!("Playing: {}", file)));
                                }
                                Err(err) => {
                                    status.set(Some(err));
                                }
                            }
                        }

                        #[cfg(target_arch = "wasm32")]
                        {
                            status.set(Some(
                                "Audio not supported in web build yet".to_string()
                            ));
                        }
                    } else {
                        status.set(Some("No audio file for this letter".to_string()));
                    }
                },

                div { class: "text-3xl font-bold", "{letter.letter}" }
                div { class: "text-sm text-gray-400 italic", "{letter.name}" }
                div { class: "text-xs text-gray-500", "{letter.pron}" }
            }
        }
    });

    rsx! {
        section { class: "p-6",
            h2 { class: "text-2xl font-semibold mb-4 text-center", "Alphabet - ანბანი" }

            div { class: "grid grid-cols-6 gap-4 max-w-3xl mx-auto",
                {cards}
            }

            // Status line for audio
            // {
            //     audio_status()
            //         .as_ref()
            //         .map(|msg| {
            //             rsx! {
            //                 div {
            //                     class: "mt-4 text-center text-sm text-gray-300",
            //                     "{msg}"
            //                 }
            //             }
            //         })
            // }
        }
    }
}
