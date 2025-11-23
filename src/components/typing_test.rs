use dioxus::events::FormEvent;
use dioxus::prelude::*;
use rand::Rng;
use std::fs;
use std::time::Duration;

use crate::components::WordCard;
use crate::models::freq_word::FrequencyWord;

const ADVANCE_DELAY: Duration = Duration::from_millis(1500);

#[component]
pub fn TypingTest(lang: Signal<String>) -> Element {
    // Which word index we are on
    let mut current_index = use_signal(|| 0usize);
    // What the user has typed for the current word
    let mut typed = use_signal(|| String::new());
    // Whether to show the English definition on the card
    let mut show_english = use_signal(|| true);

    // Countdown progress for auto-advance: None = idle, Some(f) = remaining fraction
    let mut advance_progress = use_signal(|| None::<f32>);

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

    // === AUTO-ADVANCE EFFECT + COUNTDOWN ================================
    let word_count = words.len();
    {
        let mut idx_sig       = current_index.clone();
        let mut typed_sig     = typed.clone();
        let mut progress_sig  = advance_progress.clone();
        let target_effect = target_word.clone();

        use_resource(move || {
            // Register dependencies so resource restarts when these change
            let typed_snapshot = typed_sig.read().clone();
            let cur_idx_snapshot = idx_sig();
            let target_snapshot = target_effect.clone();
            let len_snapshot = word_count;

            async move {
                // If not exactly correct, ensure progress is cleared and bail
                if typed_snapshot.is_empty() || typed_snapshot != target_snapshot || len_snapshot == 0 {
                    progress_sig.set(None);
                    return;
                }

                // We are correct → run countdown
                let steps = 20;
                let step_ms = (ADVANCE_DELAY.as_millis() / steps) as u64;

                for step in 0..=steps {
                    let fraction = 1.0 - (step as f32 / steps as f32);
                    progress_sig.set(Some(fraction.max(0.0).min(1.0)));

                    // Last step: no need to sleep again
                    if step < steps {
                        tokio::time::sleep(Duration::from_millis(step_ms)).await;
                    }
                }

                // After the countdown, pick a new word
                let mut rng = rand::rng();
                let mut next = rng.random_range(0..len_snapshot);

                if len_snapshot > 1 && next == cur_idx_snapshot {
                    next = (next + 1) % len_snapshot;
                }

                idx_sig.set(next);
                typed_sig.set(String::new());
                progress_sig.set(None);
            }
        });
    }

    // === CIRCLE PROGRESS VISUAL ========================================
    let progress_opt = advance_progress();
    let (is_counting, circumference_str, offset_str) = if let Some(p) = progress_opt {
        let radius: f32 = 16.0;
        let circumference: f32 = 2.0 * std::f32::consts::PI * radius;
        let clamped = p.max(0.0).min(1.0);
        let offset: f32 = circumference * (1.0 - clamped);
        (
            true,
            format!("{:.3}", circumference),
            format!("{:.3}", offset),
        )
    } else {
        (false, String::new(), String::new())
    };

    rsx! {
        section { class: "p-6 flex justify-center",
            div { class: "w-full max-w-4xl flex gap-8",

                // Left side: WordCard (enable whenever you want)
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

                    // Status + countdown indicator
                    div { class: "flex items-center justify-between text-sm",

                        // Left: status text
                        if all_correct {
                            span { class: "text-green-400 font-semibold", "Correct! 🎉" }
                        } else {
                            span { class: "text-gray-400",
                                "Letters: {typed_chars.len()} / {target_chars.len()}"
                            }
                        }

                        // Right: circular countdown (only visible while counting)
                        if is_counting {
                            div { class: "w-10 h-10",
                                svg {
                                    width: "40",
                                    height: "40",
                                    view_box: "0 0 40 40",

                                    // background circle
                                    circle {
                                        cx: "20",
                                        cy: "20",
                                        r: "16",
                                        stroke: "rgba(255,255,255,0.2)",
                                        "stroke-width": "4",
                                        fill: "none",
                                    }

                                    // foreground arc
                                    circle {
                                        cx: "20",
                                        cy: "20",
                                        r: "16",
                                        stroke: "rgb(129, 140, 248)", // indigo-400-ish
                                        "stroke-width": "4",
                                        fill: "none",
                                        "stroke-linecap": "round",
                                        "stroke-dasharray": "{circumference_str}",
                                        "stroke-dashoffset": "{offset_str}",
                                        transform: "rotate(-90 20 20)",
                                        // small transition so it feels smooth
                                        class: "transition-[stroke-dashoffset] duration-50 linear",
                                    }
                                }
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
