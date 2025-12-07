use dioxus::events::FormEvent;
use dioxus::prelude::*;
use rand::Rng;
use std::time::Duration;
use dioxus_primitives::slider::SliderValue;
use crate::components::slider::{Slider,SliderRange, SliderThumb, SliderTrack};
use dioxus_primitives::{ContentSide, ContentAlign};
use crate::components::tooltip::{Tooltip,TooltipTrigger,TooltipContent};
use crate::components::toggle::Toggle;
use crate::models::{freq_word::FrequencyWord, letter::Letter};
use crate::assets::freq_json_for;
use std::collections::BTreeSet;

#[component]
pub fn TypingTest(lang: Signal<String>, letters_vec: Vec<Letter>) -> Element {
    // Which word index we are on
    let current_index = use_signal(|| 0usize);
    // What the user has typed for the current word
    let mut typed = use_signal(|| String::new());

    // Countdown progress for auto-advance: None = idle, Some(f) = remaining fraction
    let advance_progress = use_signal(|| None::<f32>);
    let mut advance_delay = use_signal(|| Duration::from_millis(1500));

    // POS filter: which parts of speech are enabled
    // Empty = treat as "all enabled"
    let mut active_pos = use_signal(|| Vec::<String>::new());

    // Load the 1000-word frequency list for the current language
    let words_res = use_resource(move || {
        let lang_name = lang.read().clone();

        async move {
            let json = freq_json_for(&lang_name);

            serde_json::from_str::<Vec<FrequencyWord>>(json)
                .unwrap_or_else(|e| {
                    eprintln!("Failed to parse 1000.json for {lang_name}: {e}");
                    Vec::new()
                })
        }
    });

    let words = words_res.read().clone().unwrap_or_default();

    // ── Derive distinct POS tags from words ────────────────────────────────
    let mut pos_set = BTreeSet::new();
    for w in &words {
        if let Some(pos) = &w.pos {
            pos_set.insert(pos.clone());
        }
    }
    let all_pos: Vec<String> = pos_set.into_iter().collect();

	// If no filters selected, treat as "no filter" → all words
	let active_now = active_pos();

	// How many distinct POS values exist in this language
	let all_pos_count = all_pos.len();

	let filtered_words: Vec<FrequencyWord> = if active_now.is_empty() {
		// No filter → everything
		words.clone()
	} else {
		words
			.iter()
			.cloned()
			.filter(|w| match &w.pos {
				// Word has a POS → keep it if that POS is enabled
				Some(p) => active_now.contains(p),

				// Word has NO pos:
				// - include it only if *all* POS are enabled (active_now == all_pos)
				//   so "everything on" behaves like no filter.
				None => active_now.len() == all_pos_count,
			})
			.collect()
	};


    if filtered_words.is_empty() {
        return rsx! {
            section { class: "p-6 flex justify-center",
                div { class: "text-gray-300 text-center space-y-2",
                    div { "No words match the current POS filters." }
                    if !all_pos.is_empty() {
                        div { class: "text-xs text-gray-500",
                            "Try enabling more parts of speech."
                        }
                    }
                }
            }
        };
    }

    // Clamp index so we don't go out of bounds after filtering
    let word_count = filtered_words.len();
    let idx = current_index().min(word_count - 1);
    let current = filtered_words[idx].clone();

    let target_word = current.word.clone();
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

    // === AUTO-ADVANCE EFFECT + COUNTDOWN ==================================
    {
        let mut idx_sig      = current_index.clone();
        let mut typed_sig    = typed.clone();
        let mut progress_sig = advance_progress.clone();
        let delay_sig        = advance_delay.clone();
        let target_effect    = target_word.clone(); // captured immutably
        let len_snapshot     = word_count;

        use_resource(move || {
            let typed_snapshot   = typed_sig.read().clone();
            let cur_idx_snapshot = idx_sig();          // current index
            let target_snapshot  = target_effect.clone();
            let delay_snapshot   = delay_sig();        // Duration

            async move {
                if typed_snapshot.is_empty()
                    || typed_snapshot != target_snapshot
                    || len_snapshot == 0
                {
                    progress_sig.set(None);
                    return;
                }

                let steps: u64 = 20;
                let total_ms: u64 = delay_snapshot.as_millis() as u64;
                let step_ms: u64 = (total_ms / steps).max(1);

                for step in 0..=steps {
                    let fraction = 1.0 - (step as f32 / steps as f32);
                    progress_sig.set(Some(fraction.clamp(0.0, 1.0)));

                    if step < steps {
                        tokio::time::sleep(Duration::from_millis(step_ms)).await;
                    }
                }

                // After the countdown, pick a new word FROM THE FILTERED SET
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

    // === CIRCLE PROGRESS VISUAL ===========================================
    let progress_opt = advance_progress();
    let (is_counting, circumference_str, offset_str) = if let Some(p) = progress_opt {
        let radius: f32 = 16.0;
        let circumference: f32 = 2.0 * std::f32::consts::PI * radius;
        let clamped = p.clamp(0.0, 1.0);
        let offset: f32 = circumference * (1.0 - clamped);
        (
            true,
            format!("{:.3}", circumference),
            format!("{:.3}", offset),
        )
    } else {
        (false, String::new(), String::new())
    };

    let delay_ms = advance_delay().as_millis() as f32;

    let duration_display = if delay_ms >= 1000.0 {
        format!("{:.1} s", delay_ms / 1000.0)
    } else {
        format!("{} ms", delay_ms.round() as i32)
    };

    let mut show_set_delay = use_signal(|| false );

    rsx! {
        section { class: "flex justify-center",
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
                        input {
                            class: "w-full px-3 py-2 rounded bg-gray-900 border border-gray-700 \
                                    text-white focus:border-indigo-500 focus:outline-none text-center",
                            value: "{typed_now}",
                            oninput: move |evt: FormEvent| {
                                typed.set(evt.value());
                            },
                        }
                    }

                    // Status + countdown indicator
                    div { class: "flex items-center justify-between text-sm",
                        if all_correct {
                            span { class: "text-green-400 font-semibold", "Correct! 🎉" }
                        } else {
                            span { class: "text-gray-400",
                                "Letters: {typed_chars.len()} / {target_chars.len()}"
                            }
                        }

                        if is_counting {
                            div { class: "w-10 h-10",
                                svg {
                                    width: "40",
                                    height: "40",
                                    view_box: "0 0 40 40",

                                    circle {
                                        cx: "20",
                                        cy: "20",
                                        r: "16",
                                        stroke: "rgba(255,255,255,0.2)",
                                        "stroke-width": "4",
                                        fill: "none",
                                    }

                                    circle {
                                        cx: "20",
                                        cy: "20",
                                        r: "16",
                                        stroke: "rgb(129, 140, 248)",
                                        "stroke-width": "4",
                                        fill: "none",
                                        "stroke-linecap": "round",
                                        "stroke-dasharray": "{circumference_str}",
                                        "stroke-dashoffset": "{offset_str}",
                                        transform: "rotate(-90 20 20)",
                                        class: "transition-[stroke-dashoffset] duration-50 linear",
                                    }
                                }
                            }
                        }
                    }

                    // Tiny meta info (rank + EN + POS + example)
                    div { class: "text-xs text-gray-500 mt-1 flex flex-col items-center gap-1",
                        div { class: "flex items-center gap-2",
                            span { "Rank #{current.rank} — {current.en}" }

                            if let Some(pos) = current.pos.clone() {
                                span {
                                    class: "px-2 py-0.5 rounded-full bg-indigo-900 text-indigo-200 \
                                            text-[0.65rem] uppercase tracking-wide",
                                    "{pos}"
                                }
                            }
                        }

                        if let Some(ex) = current.example.clone() {
                            div {
                                class: "text-[0.7rem] text-gray-400 italic text-center max-w-md",
                                "{ex}"
                            }
                        }
                    }

                    if show_set_delay() {
						div { class:"flex flex-col items-center gap-3 mb-3",

							// ── Delay slider + label ─────────────────────────────
							div { class: "flex items-center gap-5",
								Slider {
									default_value: SliderValue::Single(advance_delay().as_millis() as f64),
									min: 100.0,
									max: 1500.0,
									step: 50.0,
									horizontal: true,
									on_value_change: move |value: SliderValue| {
										let SliderValue::Single(v) = value;
										advance_delay.set(Duration::from_millis(v as u64));
									},

									SliderTrack {
										SliderRange {}
										SliderThumb {}
									}
								}

								div { style: "font-size: 16px; font-weight: bold;",
									"{duration_display}"
								}
							}

							// ── POS FILTER TOGGLES (separate row) ─────────────────
							if !all_pos.is_empty() {
								// Outer: limited width, scrolls horizontally
								div { class: "w-full max-w-md mx-auto overflow-x-auto",
									// Inner: single line, horizontally scrollable
									div { class: "inline-flex gap-2 text-xs whitespace-nowrap",
										{all_pos.iter().map(|pos_label| {
											let label_text        = pos_label.clone();          // display
											let label_for_closure = label_text.clone();         // move into closure

											let mut active_pos_sig = active_pos.clone();
											let mut idx_sig        = current_index.clone();
											let mut typed_sig      = typed.clone();
											let all_pos_clone      = all_pos.clone();

											let pressed_now = {
												let act = active_pos_sig.read();
												act.is_empty() || act.contains(&label_text)
											};

											rsx! {
												Toggle {
													pressed: pressed_now,
													on_pressed_change: move |pressed: bool| {
														active_pos_sig.with_mut(|vec| {
															if vec.is_empty() {
																*vec = all_pos_clone.clone();
															}

															let idx = vec.iter().position(|p| p == &label_for_closure);

															match (pressed, idx) {
																(true,  None)    => vec.push(label_for_closure.clone()),
																(false, Some(i)) => { vec.remove(i); }
																_ => {}
															}
														});

														idx_sig.set(0);
														typed_sig.set(String::new());
													},
													{label_text}
												}
											}
										})}
									}
								}
							}

							// ── Hide button ──────────────────────────────────────
							button {
								class:"text-center opacity-50 hover:opacity-100 transition-all duration-300 \
									hover:scale-105 hover:cursor-pointer",
								onclick: move |_| {
									show_set_delay.set(false);
								},
								"Hide"
							}
						}
					} else {
                        div {class:"flex justify-center mb-3",
                            Tooltip { 
                                TooltipTrigger { class:"flex justify-center",
                                    button{ class:"text-center opacity-50 hover:opacity-100 transition-all duration-300 hover:scale-105 hover:cursor-pointer", onclick: move |_| {
                                            show_set_delay.set(true);
                                        },
                                        "Settings"
                                    }
                                }
                                TooltipContent {
                                    side: ContentSide::Top,
                                    align: ContentAlign::Center,
                                    div{class:"w-[150px] text-center","Current Delay: {duration_display}"}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
