use dioxus::events::FormEvent;
use dioxus::prelude::*;
use rand::Rng;
use std::time::Duration;
use dioxus_primitives::slider::SliderValue;
use crate::components::radio_group::{RadioGroup, RadioItem};
use crate::components::slider::{Slider,SliderRange, SliderThumb, SliderTrack};
use dioxus_primitives::{ContentSide, ContentAlign};
use crate::components::tooltip::{Tooltip,TooltipTrigger,TooltipContent};
use crate::components::toggle::Toggle;
use crate::components::input::Input;
use crate::models::{freq_word::FrequencyWord, letter::Letter};
use crate::assets::freq_json_for;
use std::collections::{BTreeSet, HashMap};
use crate::components::keyboard::code_to_qwerty_label;


#[derive(Clone, Copy, PartialEq, Eq)]
enum TestMode {
    Sequential,
    Random,
    Bounded,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BoundedOrder {
    Sequential,
    Random,
}

impl BoundedOrder {
    fn as_str(self) -> &'static str {
        match self {
            BoundedOrder::Sequential => "bounded_seq",
            BoundedOrder::Random     => "bounded_rand",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "bounded_seq"  => BoundedOrder::Sequential,
            "bounded_rand" => BoundedOrder::Random,
            _              => BoundedOrder::Random,
        }
    }
}


impl TestMode {
    fn as_str(self) -> &'static str {
        match self {
            TestMode::Sequential => "sequential",
            TestMode::Random     => "random",
            TestMode::Bounded    => "bounded",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "sequential" => TestMode::Sequential,
            "bounded"    => TestMode::Bounded,
            _            => TestMode::Random,
        }
    }
}


#[component]
pub fn TypingTest(lang: Signal<String>, letters_vec: Vec<Letter>) -> Element {
    // Which word index we are on
    let mut current_index = use_signal(|| 0usize);
    // What the user has typed for the current word
    let mut typed = use_signal(|| String::new());

    // Countdown progress for auto-advance: None = idle, Some(f) = remaining fraction
    let advance_progress = use_signal(|| None::<f32>);
    let mut advance_delay = use_signal(|| Duration::from_millis(1500));

    // POS filter: which parts of speech are enabled
    // Empty = treat as "all enabled"
    let mut active_pos = use_signal(|| Vec::<String>::new());
	let mut test_mode = use_signal(|| TestMode::Random);
	let mut min_rank  = use_signal(|| 1u32);
	let mut max_rank  = use_signal(|| 250u32);
	let mut bounded_order = use_signal(|| BoundedOrder::Random);

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
	if words.is_empty() {
    return rsx! {
        section { class: "p-6 flex justify-center",
            div { class: "text-gray-300 text-center space-y-2",
                div { "No frequency list loaded (1000.json)." }
                div { class: "text-xs text-gray-500",
                    "Check that your 1000.json exists and is valid."
                }
            }
        }
    };
}

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
		// All distinct POS in this language
	let all_pos_count = all_pos.len();

	// Current POS filters
	let active_now = active_pos();

	// First: POS filtering
	let words_pos_filtered: Vec<FrequencyWord> = if active_now.is_empty() {
		// No POS filter → keep all
		words.clone()
	} else {
		words
			.iter()
			.cloned()
			.filter(|w| match &w.pos {
				Some(p) => active_now.contains(p),
				// entries with no POS only survive if “all POS” are enabled
				None => active_now.len() == all_pos_count,
			})
			.collect()
	};

	// Then: apply bounded mode (by rank), if selected
	let mode_now = test_mode();          // TestMode enum
	let lo = min_rank();
	let hi = max_rank().max(lo);         // keep hi >= lo

	let filtered_words: Vec<FrequencyWord> = match mode_now {
			TestMode::Bounded => {
				words_pos_filtered
					.into_iter()
					.filter(|w| w.rank >= lo && w.rank <= hi)
					.collect()
			}
			_ => words_pos_filtered,
		};
		if filtered_words.is_empty() {
		return rsx! {
			section { class: "p-6 flex justify-center",
				div { class: "text-gray-300 text-center space-y-2",
					div { "No words match the current filters/rank range." }
					div { class: "text-xs text-gray-500",
						"Open Settings and adjust POS or rank bounds."
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
	
	// Map Georgian char -> "J" or "⇧J" depending on whether it's shifted
	let mut hint_map: HashMap<char, String> = HashMap::new();

	for letter in &letters_vec {
		if let Some(ch) = letter.letter.chars().next() {
			let base = code_to_qwerty_label(&letter.key_code);
			if !base.is_empty() {
				let rendered = if letter.shifted {
					// whatever style you like:
					// format!("Shift+{}", base)
					format!("⇧{}", base)
				} else {
					base.to_string()
				};

				// if multiple entries share same char, last one wins; that's fine here
				hint_map.insert(ch, rendered);
			}
		}
	}


    // === AUTO-ADVANCE EFFECT + COUNTDOWN ==================================
	{
		let mut idx_sig      = current_index.clone();
		let mut typed_sig    = typed.clone();
		let mut progress_sig = advance_progress.clone();
		let delay_sig        = advance_delay.clone();
		let target_effect    = target_word.clone(); // captured immutably
		let len_snapshot     = word_count;
		let mode_sig         = test_mode.clone();   // <── NEW
		let bounded_order_sig = bounded_order.clone();

		use_resource(move || {
			let typed_snapshot   = typed_sig.read().clone();
			let cur_idx_snapshot = idx_sig();
			let target_snapshot  = target_effect.clone();
			let delay_snapshot   = delay_sig();
			let mode_snapshot    = mode_sig();
			let order_snapshot   = bounded_order_sig(); // <── new

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

				let next = match mode_snapshot {
					TestMode::Sequential => {
						if len_snapshot == 0 {
							0
						} else {
							(cur_idx_snapshot + 1) % len_snapshot
						}
					}

					TestMode::Random => {
						let mut rng = rand::rng();
						let mut n = rng.random_range(0..len_snapshot);

						if len_snapshot > 1 && n == cur_idx_snapshot {
							n = (n + 1) % len_snapshot;
						}
						n
					}

					TestMode::Bounded => {
						match order_snapshot {
							BoundedOrder::Sequential => {
								if len_snapshot == 0 {
									0
								} else {
									(cur_idx_snapshot + 1) % len_snapshot
								}
							}
							BoundedOrder::Random => {
								let mut rng = rand::rng();
								let mut n = rng.random_range(0..len_snapshot);

								if len_snapshot > 1 && n == cur_idx_snapshot {
									n = (n + 1) % len_snapshot;
								}
								n
							}
						}
					}
				};

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

	let typed_chars_for_display = typed_chars.clone();
	let letters_typed_len = typed_chars.len();
	let target_len = target_chars.len();

    rsx! {
        section { class: "flex justify-center",
            div { class: "w-full max-w-4xl flex gap-8",

                div { class: "flex-1 flex flex-col gap-4",

					// Per-letter display + subtle QWERTY hint
               div { class: "flex justify-center gap-4 text-3xl",
					{
						let hint_map = hint_map.clone();
						let typed_chars_for_display = typed_chars_for_display.clone();

						target_chars.iter().enumerate().map(move |(i, ch)| {
							let class = if i < typed_chars_for_display.len() {
								if typed_chars_for_display[i] == *ch {
									"text-white"
								} else {
									"text-red-400"
								}
							} else {
								"text-gray-500"
							};

							let hint = hint_map
								.get(ch)
								.map(|s| s.as_str())
								.unwrap_or("");

							rsx! {
								div { class: "flex flex-col items-center leading-tight",
									span { class: "{class} font-bold", "{ch}" }
									span { class: "text-xs text-gray-500 opacity-60 mt-1", "{hint}" }
								}
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
						div { class:"flex flex-col items-center gap-3 mb-3 border-b-1 rounded",

							div { class:"grid gap-5",
								// ── Delay slider + label ─────────────────────────────
							h4 { class: "text-xs text-gray-400 text-center", "Auto Advance Delay" }
							
							div { class: "flex justify-center gap-5",
							
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
								h4 { class: "text-xs text-gray-400 text-center", "Point of Speech Filtration [all by default]" }

								div { class: "w-full max-w-md mx-auto overflow-x-auto",
									// Inner: single line, horizontally scrollable
									div { class: "grid grid-cols-6 gap-1 text-xs whitespace-nowrap",
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
												div{class:"flex justify-center",
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
											}
										})}
									}
								}
							}
							div{
								// ── TEST MODE RADIO GROUP ─────────────────────────────────
								div { class: "w-full max-w-md mx-auto mb-2",
									h4 { class: "text-xs text-gray-400 mb-1 text-center", "Test mode" }
									div{ class:"flex justify-center",

										RadioGroup {
											// current selection
											value: test_mode().as_str().to_string(),
											horizontal: true,
											on_value_change: move |value: String| {
												test_mode.set(TestMode::from_str(&value));
												// reset index & input when mode changes
												current_index.set(0);
												typed.set(String::new());
											},

												RadioItem {
													value: "sequential".to_string(),
													index: 0usize,
													"Sequential"
												}
												RadioItem {
													value: "random".to_string(),
													index: 1usize,
													"Random"
												}
												RadioItem {
													value: "bounded".to_string(),
													index: 2usize,
													"Bounded (by rank)"
												}
											}
										}
										if matches!(test_mode(), TestMode::Bounded) {
											{
												let mut min_rank_sig = min_rank.clone();
												let mut max_rank_sig = max_rank.clone();
												let mut idx_sig      = current_index.clone();
												let mut typed_sig    = typed.clone();
												let max_rank_available = words.iter().map(|w| w.rank).max().unwrap_or(1);
												let mut bounded_order_sig = bounded_order.clone();

												rsx! {
													div {
														class: "flex flex-col items-center gap-2 text-xs text-gray-300 mt-2",

														// rank range row (your existing inputs) ...
														div {
															class: "flex items-center justify-center gap-2",
															span { "Rank range:" }

															// MIN
															Input {
																r#type: "number",
																min: "1",
																max: max_rank_available.to_string(),
																step: "1",
																value: min_rank().to_string(),
																oninput: move |evt: FormEvent| {
																	let raw = evt.value();
																	if let Ok(mut v) = raw.parse::<u32>() {
																		if v < 1 { v = 1; }
																		if v > max_rank_available { v = max_rank_available; }

																		let current_max = max_rank_sig();
																		if v > current_max {
																			max_rank_sig.set(v);
																		}

																		min_rank_sig.set(v);
																		idx_sig.set(0);
																		typed_sig.set(String::new());
																	}
																}
															}

															span { "–" }

															// MAX
															Input {
																r#type: "number",
																min: "1",
																max: max_rank_available.to_string(),
																step: "1",
																value: max_rank().to_string(),
																oninput: move |evt: FormEvent| {
																	let raw = evt.value();
																	if let Ok(mut v) = raw.parse::<u32>() {
																		if v < 1 { v = 1; }
																		if v > max_rank_available { v = max_rank_available; }

																		let current_min = min_rank_sig();
																		if v < current_min {
																			v = current_min;
																		}

																		max_rank_sig.set(v);
																		idx_sig.set(0);
																		typed_sig.set(String::new());
																	}
																}
															}
														}

														// order within bounded range
														div {
															class: "flex items-center gap-2 mt-1",
															span { "Order in range:" }

															RadioGroup {
																value: bounded_order_sig().as_str().to_string(),
																horizontal: true,
																on_value_change: move |value: String| {
																	bounded_order_sig.set(BoundedOrder::from_str(&value));
																	idx_sig.set(0);
																	typed_sig.set(String::new());
																},

																RadioItem {
																	value: "bounded_seq".to_string(),
																	index: 0usize,
																	"Sequential"
																}
																RadioItem {
																	value: "bounded_rand".to_string(),
																	index: 1usize,
																	"Random"
																}
															}
														}
													}
												}
											}
										}
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
                                    div{class:"w-[150px] text-center border-b","Current"}
                                    div{class:"w-[150px] text-center","Advance Delay: {duration_display}"}
                                    div{class:"w-[150px] text-center","Expand for POS filters"}
                                    div{class:"w-[150px] text-center","Test Mode: {test_mode().as_str()}"}

                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
