use dioxus::prelude::*;
use dioxus_primitives::slider::SliderValue;
use crate::{
    audio,
    components::slider::{Slider, SliderRange, SliderThumb, SliderTrack},
    models::letter::Letter,
};
use std::time::Duration;

const FLASH_DURATION: Duration = Duration::from_millis(700);

#[component]
pub fn Alphabet(letters: Vec<Letter>, lang: Signal<String>) -> Element {
    // Which button should show the ring?
    let mut flashing = use_signal(|| None::<String>);
    let mut volume = use_signal(|| 0.4);

    // Derive language-specific title once per render
    let alphabet_target_lang: &str = match lang.read().as_str() {
        "georgian" => "ანბანი",
        "russian"  => "Алфавит",
        _          => "🟪⬛",
    };

    // Whenever `flashing` changes, start a 700ms timer to clear it
    use_resource(move || {
        let mut flashing_signal = flashing.clone();

        async move {
            let maybe_path: Option<String> = {
                let read_guard = flashing_signal.read();
                read_guard.clone()
            };

            if let Some(path) = maybe_path {
                tokio::time::sleep(FLASH_DURATION).await;

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
                            hover:border-indigo-500 hover:cursor-pointer select-none text-white";

        let ring_classes = if is_flashing {
            " border-indigo-400 ring-4 ring-indigo-400 !text-indigo-500"
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

                        #[cfg(not(target_arch = "wasm32"))]
                        audio::play_audio(&play_path, volume());

                        flashing_signal.set(Some(play_path));
                    }
                },

                div { class: "text-3xl font-bold", "{letter.letter}" }
                div { class: "text-sm text-gray-400 italic", "{letter.name}" }
                div { class: "text-xs text-gray-500", "{letter.pron}" }
            }
        }
    });

    let volume_pct = (volume() * 100.0).round() as i32;

    rsx! {
        section { class: "p-6",
            h2 { class: "text-2xl font-semibold mb-4 text-center",
                "Alphabet – {alphabet_target_lang}"
            }

            div { class:"flex justify-center gap-3",
                h4 { class:"text-center flex", "Volume" }

                div {
                    Slider {
                        default_value: SliderValue::Single(volume() as f64),
                        min: 0.0,
                        max: 1.0,
                        step: 0.05,
                        horizontal: true,
                        on_value_change: move |value: SliderValue| {
                            let SliderValue::Single(v) = value;
                            volume.set(v as f32);
                        },

                        SliderTrack {
                            SliderRange {}
                            SliderThumb {}
                        }
                    }
                }

                div {
                    style: "margin-bottom: 15px; font-size: 16px; font-weight: bold;",
                    "{volume_pct}%"
                }
            }

            div { class: "grid grid-cols-6 gap-4 max-w-3xl mx-auto",
                {cards}
            }
        }
    }
}
