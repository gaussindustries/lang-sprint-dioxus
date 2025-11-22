use dioxus::events::FormEvent;
use dioxus::prelude::*;
use rand::Rng;
use std::fs;

use crate::components::WordCard;
use crate::models::freq_word::FrequencyWord;

#[component]
pub fn TypingTest(lang: Signal<String>) -> Element {
    // Which word index we are on
    let mut current_index = use_signal(|| 0usize);
    // What the user has typed for the current word
    let mut typed = use_signal(|| String::new());
    // Whether to show the English definition on the card
    let mut show_english = use_signal(|| true);

    // Load the 1000-word frequency list for the current language
    let words_res = use_resource(move || {
        let lang_name = lang.read().clone();
        async move {
            let path = format!("langs/{}/1000.json", lang_name);
            let raw = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str::<Vec<FrequencyWord>>(&raw).unwrap_or_default()
        }
    });

    let words = words_res.read().clone().unwrap_or_default();

    if words.is_empty() {
        return rsx! {
            section { class: "p-6 flex justify-center",
                div { class: "text-gray-300", "No frequency list loaded (1000.json)." }
            }
        };
    }

    // Clamp index so we don't go out of bounds if file changes
    let idx = current_index().min(words.len() - 1);
    let current = words[idx].clone();

    let target_word = current.ge.clone();
    let typed_now = typed();

    let target_chars: Vec<char> = target_word.chars().collect();
    let typed_chars: Vec<char> = typed_now.chars().collect();

    // Check if fully correct
    let all_correct = !typed_chars.is_empty()
        && typed_chars.len() == target_chars.len()
        && typed_chars
            .iter()
            .zip(target_chars.iter())
            .all(|(a, b)| a == b);

    rsx! {
        section { class: "p-6 flex justify-center",
            div { class: "w-full max-w-4xl flex gap-8",

                // Left side: WordCard
                div { class: "flex-1",
                    WordCard {
                        word: current.ge.clone(),
                        def: if show_english() { current.en.clone() } else { String::new() },
                        pos: current.pos.clone().unwrap_or_else(|| "".into()),
                        example: current.example.clone().unwrap_or_else(|| "".into()),
                    }

                    // Small toggle to hide/show English def (future: slider/shortcut)
                    button {
                        class: "mt-3 px-3 py-1 text-xs rounded bg-gray-700 hover:bg-gray-600",
                        onclick: move |_| {
                            show_english.with_mut(|v| *v = !*v);
                        },
                        if show_english() {
                            "Hide English"
                        } else {
                            "Show English"
                        }
                    }
                }

                // Right side: typing practice
                div { class: "flex-1 flex flex-col gap-4",

                    // Per-letter display
                    div { class: "flex justify-center gap-2 text-3xl",
                        {
                            target_chars.iter().enumerate().map(|(i, ch)| {
                                let class = if i < typed_chars.len() {
                                    if typed_chars[i] == *ch {
                                        // correct letter
                                        "text-white"
                                    } else {
                                        // typed but wrong at this position
                                        "text-red-400"
                                    }
                                } else {
                                    // not reached yet
                                    "text-gray-500"
                                };

                                rsx! {
                                    span { class: "{class} font-bold", "{ch}" }
                                }
                            })
                        }
                    }

                    // Input field
                    div {
                        label { class: "block text-sm text-gray-300 mb-1",
                            "Type the word in Georgian:"
                        }
                        input {
                            class: "w-full px-3 py-2 rounded bg-gray-900 border border-gray-700 \
                                    text-white focus:border-indigo-500 focus:outline-none",
                            value: "{typed_now}",
                            oninput: move |evt: FormEvent| {
                                typed.set(evt.value());
                            },
                        }
                    }

                    // Status + controls
                    div { class: "flex items-center justify-between text-sm",
                        if all_correct {
                            span { class: "text-green-400 font-semibold", "Correct! 🎉" }
                        } else {
                            span { class: "text-gray-400",
                                "Letters: {typed_chars.len()} / {target_chars.len()}"
                            }
                        }

                        button {
                            class: format!(
                                "px-4 py-2 rounded text-sm font-semibold {}",
                                if all_correct {
                                    "bg-indigo-600 hover:bg-indigo-500 text-white"
                                } else {
                                    "bg-gray-700 text-gray-400 cursor-not-allowed"
                                }
                            ),
                            disabled: !all_correct,
                            onclick: move |_| {
                                if words.is_empty() {
                                    return;
                                }
                                // Simple RNG: pick a new index
                                let len = words.len();
                                let mut rng = rand::thread_rng();
                                let mut next = rng.gen_range(0..len);

                                // avoid repeating same word when possible
                                let cur_idx = current_index();
                                if len > 1 && next == cur_idx {
                                    next = (next + 1) % len;
                                }

                                current_index.set(next);
                                typed.set(String::new());
                            },
                            "Next word"
                        }
                    }

                    // Tiny meta info
                    div { class: "text-xs text-gray-500 mt-1",
                        "Rank #{current.rank} — {current.en}"
                    }
                }
            }
        }
    }
}
