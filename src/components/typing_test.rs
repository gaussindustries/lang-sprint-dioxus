use dioxus::events::FormEvent;
use dioxus::prelude::*;
use rand::Rng;
use std::fs;
use std::time::Duration;

use crate::components::WordCard;
use crate::models::freq_word::FrequencyWord;

const ADVANCE_DELAY: Duration = Duration::from_millis(500);

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

    // Check if fully correct (for UI only)
    let all_correct = !typed_chars.is_empty()
        && typed_chars.len() == target_chars.len()
        && typed_chars
            .iter()
            .zip(target_chars.iter())
            .all(|(a, b)| a == b);

    // === AUTO-ADVANCE EFFECT =========================================
    //
    // Whenever `typed` or `target_word` changes, this resource re-runs,
    // snapshots them, and if they match, waits 500ms then advances.
    let word_count = words.len();
    {
        let mut idx_sig   = current_index.clone();
        let mut typed_sig = typed.clone();
        let target_for_effect = target_word.clone(); // captured immutably

        use_resource(move || {
            // snapshot current typed, index, and target
            let typed_snapshot = typed_sig.read().clone();
            let cur_idx_snapshot = idx_sig(); // registers dependency on index
            let target_snapshot = target_for_effect.clone();

            async move {
                // Don't advance on empty or wrong input
                if typed_snapshot.is_empty() || typed_snapshot != target_snapshot {
                    return;
                }

                if word_count == 0 {
                    return;
                }

                // Wait so the user can see "Correct! 🎉"
                tokio::time::sleep(ADVANCE_DELAY).await;

                let len = word_count;
                let mut rng = rand::rng();
                let mut next = rng.random_range(0..len);

                // avoid repeating the same word when possible
                if len > 1 && next == cur_idx_snapshot {
                    next = (next + 1) % len;
                }

                idx_sig.set(next);
                typed_sig.set(String::new());
            }
        });
    }
 // Left side: WordCard (re-enable when you want)
                // div { class: "flex-1",
                //     WordCard {
                //         word: current.ge.clone(),
                //         def: if show_english() { current.en.clone() } else { String::new() },
                //         pos: current.pos.clone().unwrap_or_else(|| "".into()),
                //         example: current.example.clone().unwrap_or_else(|| "".into()),
                //     }
                //
                //     button {
                //         class: "mt-3 px-3 py-1 text-xs rounded bg-gray-700 hover:bg-gray-600",
                //         onclick: move |_| {
                //             show_english.with_mut(|v| *v = !*v);
                //         },
                //         if show_english() { "Hide English" } else { "Show English" }
                //     }
                // }

    rsx! {
        section { class: "p-6 flex justify-center",
            div { class: "w-full max-w-4xl flex gap-8",

                // Right side: typing practice
                div { class: "flex-1 flex flex-col gap-4",

                    // Per-letter display
                    div { class: "flex justify-center gap-2 text-3xl",
                        {
                            target_chars.iter().enumerate().map(|(i, ch)| {
                                let class = if i < typed_chars.len() {
                                    if typed_chars[i] == *ch {
                                        "text-white"       // correct letter
                                    } else {
                                        "text-red-400"     // wrong at this position
                                    }
                                } else {
                                    "text-gray-500"       // not reached yet
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

                    // Status
                    div { class: "flex items-center justify-between text-sm",
                        if all_correct {
                            span { class: "text-green-400 font-semibold", "Correct! 🎉" }
                        } else {
                            span { class: "text-gray-400",
                                "Letters: {typed_chars.len()} / {target_chars.len()}"
                            }
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
