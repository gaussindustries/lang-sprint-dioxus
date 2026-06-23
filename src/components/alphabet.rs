use dioxus::prelude::*;
use std::time::Duration;

use crate::{
    assets::letter_audio_bytes,
    audio,
    models::letter::{Letter, LetterKind},
    settings::use_settings,
};

const FLASH_DURATION: Duration = Duration::from_millis(700);

#[component]
pub fn Alphabet(letters: Vec<Letter>, lang: Signal<String>) -> Element {
    // Logical id of the card currently flashing its ring (e.g. "georgian/a.wav").
    let mut flashing = use_signal(|| None::<String>);
    // Volume now lives in the shared settings store (set from the navbar gear).
    let settings = use_settings();

    let title = match lang.read().as_str() {
        "georgian" => "ანბანი",
        "russian" => "Алфавит",
        _ => "🟪⬛",
    };

    // Clear the flash 700ms after it changes (only if it's still the same id).
    use_resource(move || async move {
        let current = flashing.read().clone(); // guard dropped at this semicolon
        if let Some(id) = current {
            tokio::time::sleep(FLASH_DURATION).await;
            flashing.with_mut(|cur| {
                if cur.as_ref() == Some(&id) {
                    *cur = None;
                }
            });
        }
    });

    let cards = letters.into_iter().map(move |letter| {
        let lang_name = lang.read().clone();
        let audio_file = letter.audio.clone();

        // Must match the id we set on click, below.
        let id = format!(
            "{lang_name}/{}",
            audio_file.as_deref().unwrap_or("<missing>.wav")
        );
        let is_flashing = flashing.read().as_ref() == Some(&id);

        // Resting border encodes vowel/consonant; flashing overrides it.
        let border = if is_flashing {
            "border-indigo-400 ring-4 ring-indigo-400 !text-indigo-500"
        } else {
            match letter.kind {
                LetterKind::Vowel => "border-amber-400",
                LetterKind::Consonant => "border-sky-500",
                LetterKind::Other => "border-gray-600",
            }
        };

        let base = "group p-4 rounded-lg border-2 transition-all text-center \
                    hover:border-indigo-500 hover:cursor-pointer select-none text-white";

        rsx! {
            button {
                key: "{letter.letter}",
                class: "{base} {border}",

                onclick: move |_| {
                    if let Some(file) = audio_file.as_deref() {
                        let id = format!("{lang_name}/{file}");
                        if let Some(bytes) = letter_audio_bytes(&lang_name, file) {
                            #[cfg(not(target_arch = "wasm32"))]
                            audio::play_audio_bytes(&id, bytes, settings.read().volume);
                            flashing.set(Some(id));
                        } else {
                            eprintln!("No embedded audio for {lang_name}/{file}");
                        }
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
            h2 { class: "text-2xl font-semibold mb-4 text-center", "Alphabet – {title}" }

            // legend (volume moved to the settings dialog in the navbar)
            div { class: "flex justify-center gap-4 my-3 text-xs text-gray-400",
                div { class: "flex items-center gap-1.5",
                    span { class: "inline-block w-3 h-3 rounded-sm border-2 border-amber-400" }
                    "vowel"
                }
                div { class: "flex items-center gap-1.5",
                    span { class: "inline-block w-3 h-3 rounded-sm border-2 border-sky-500" }
                    "consonant"
                }
            }

            div { class: "grid grid-cols-6 gap-4 max-w-3xl mx-auto",
                {cards}
            }
        }
    }
}
