// src/views/alphabet.rs
use dioxus::prelude::*;
use crate::{audio, models::letter::Letter};
use std::time::Duration;

const FLASH_DURATION: Duration = Duration::from_millis(700);

#[component]
pub fn Alphabet(letters: Vec<Letter>, lang: Signal<String>) -> Element {
    // Which button should show the ring?
    let mut flashing = use_signal(|| None::<String>);

    // Whenever `flashing` changes, start a 700ms timer to clear it
    use_resource(move || {
        let mut flashing_signal = flashing.clone();

        async move {
            // Take a snapshot in a *separate scope* so the read borrow ends
            let maybe_path: Option<String> = {
                let read_guard = flashing_signal.read();
                read_guard.clone()
            };

            if let Some(path) = maybe_path {
                tokio::time::sleep(FLASH_DURATION).await;

                // Now we can borrow mutably safely
                flashing_signal.with_mut(|current| {
                    if current.as_ref() == Some(&path) {
                        *current = None;
                    }
                });
            }
        }
    });

    let cards = letters.into_iter().map(move |letter| {
        let lang_signal = lang.clone();
        let letter_for_click = letter.clone();
        let mut flashing_signal = flashing.clone();

        let lang_name = lang_signal.read().clone();
        let path = format!(
            "langs/{}/pronunciation/alphabet/{}",
            lang_name,
            letter_for_click.audio.as_deref().unwrap_or("<missing>.wav")
        );

        // Does this letter match the current flashing path?
        let is_flashing = flashing_signal
            .read()
            .as_ref()
            .map(|p| p == &path)
            .unwrap_or(false);

        let base_classes = "group p-4 rounded-lg border-2 transition-all text-center \
                            hover:border-indigo-500 hover:cursor-pointer";

        let ring_classes = if is_flashing {
            " border-indigo-400 ring-4 ring-indigo-400"
        } else {
            " border-gray-600"
        };

        rsx! {
            button {
                key: "{letter.letter}",
                class: "{base_classes}{ring_classes}",

                onclick: move |_| {
                    if let Some(file) = &letter_for_click.audio {
                        let lang_name = lang_signal.read().clone();
                        let play_path = format!(
                            "langs/{}/pronunciation/alphabet/{}",
                            lang_name,
                            file
                        );

                        // Play audio
                        #[cfg(not(target_arch = "wasm32"))]
                        audio::play_audio(&play_path);

                        // Start 700ms flash
                        flashing_signal.set(Some(play_path));
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
        }
    }
}
