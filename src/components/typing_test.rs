/**
 * TODO:
 * implementing user data{
 * 	calibrations for each key (utilize right click menu)
 * }
 * make generalized functions where:
 *  read json file, manipulate it in memory, save it once done, repeat
 * 
 * 	
 * 
 */

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
use crate::components::tabs::{Tabs, TabList, TabTrigger, TabContent};


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

#[derive(Clone, Copy, PartialEq, Eq)]
enum WpmState {
    Idle,
    Running,
    Finished,
}

#[derive(Clone, Copy)]
struct WpmResult {
    wpm: u32,
    correct_words: u32,
    incorrect_words: u32,
    correct_letters: u32,
    incorrect_letters: u32,
}

fn clean_word(raw: &str) -> Option<String> {
    // Take text before the first slash and trim
    let first = raw.split('/')
        .next()
        .unwrap_or("")
        .trim();

    if first.is_empty() {
        None
    } else {
        Some(first.to_string())
    }
}


fn build_wpm_text(words: &[FrequencyWord], min_chars: usize) -> String {
    if words.is_empty() {
        return String::new();
    }

    let mut rng = rand::rng();
    let mut out = String::new();

    while out.len() < min_chars {
        let idx = rng.random_range(0..words.len());
        let raw = words[idx].word.as_str();

        // sanitize: drop everything after "/", trim spaces
        let Some(clean) = clean_word(raw) else {
            continue; // skip empty / weird entries
        };

        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&clean);
    }

    out
}


fn compute_wpm_result(target: &str, typed: &str, duration_secs: u64) -> WpmResult {
    let duration_secs = duration_secs.max(1);

    let target_words: Vec<&str> = target.split_whitespace().collect();
    let typed_words: Vec<&str> = typed.split_whitespace().collect();

    let mut correct_words = 0u32;
    let mut incorrect_words = 0u32;
    let mut correct_letters = 0u32;
    let mut incorrect_letters = 0u32;

    for (i, tw) in typed_words.iter().enumerate() {
        let typed_len = tw.chars().count() as u32;
        if let Some(target_w) = target_words.get(i) {
            if tw == target_w {
                correct_words += 1;
                correct_letters += typed_len;
            } else {
                incorrect_words += 1;
                incorrect_letters += typed_len;
            }
        } else {
            // extra word that target didn't have
            incorrect_words += 1;
            incorrect_letters += typed_len;
        }
    }

    let total_typed_words = typed_words.len() as f32;
    let wpm = ((total_typed_words * 60.0) / duration_secs as f32).round() as u32;

    WpmResult {
        wpm,
        correct_words,
        incorrect_words,
        correct_letters,
        incorrect_letters,
    }
}



#[component]
pub fn TypingTest(lang: Signal<String>, letters_vec: Vec<Letter>) -> Element {
    // Which word index we are on
    let mut current_index = use_signal(|| 0usize);
    // What the user has typed for the current word
    let mut typed = use_signal(|| String::new());

    // Countdown progress for auto-advance: None = idle, Some(f) = remaining fraction
	
	

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
		

    // Clamp index so we don't go out of bounds after filtering
    let word_count = filtered_words.len();
	let has_words = word_count > 0;

	let (current_opt, target_word) = if has_words {
    let idx = current_index().min(word_count - 1);
    let current = filtered_words[idx].clone();
    let target_word = current.word.clone();   // clone the String while you still own `current`
    (Some(current), target_word)
	} else {
		(None, String::new())
	};


	let typed_now = typed();

	let target_chars: Vec<char> = target_word.chars().collect();
	let typed_chars: Vec<char> = typed_now.chars().collect();

	let all_correct = has_words
		&& !typed_chars.is_empty()
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

	// this is for the word drill tab
    let advance_progress = use_signal(|| None::<f32>);
    let mut advance_delay = use_signal(|| Duration::from_millis(1500));

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
		
		
		//this section is respect to real time 
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

    let mut show_settings = use_signal(|| false );

	let typed_chars_for_display = typed_chars.clone();
	let letters_typed_len = typed_chars.len();
	let target_len = target_chars.len();
	
	let mut input_focused = use_signal(|| false);
	let mut active_test_tab = use_signal(|| "drill".to_string());

	// ── WPM-state signals ───────────────────────────────────────────────
	let mut wpm_target_text = use_signal(|| String::new());
	let mut wpm_typed       = use_signal(|| String::new());
	let mut wpm_duration    = use_signal(|| 60u64);      // default 60s
	let mut wpm_remaining   = use_signal(|| 60u64);
	let mut wpm_state       = use_signal(|| WpmState::Idle);
	let mut wpm_result      = use_signal(|| None::<WpmResult>);

	// ── WPM timer effect ─────────────────────────────────────────────────-
	{
		let mut state_sig   = wpm_state.clone();
		let mut rem_sig     = wpm_remaining.clone();
		let duration_sig    = wpm_duration.clone();
		let typed_sig       = wpm_typed.clone();
		let target_sig      = wpm_target_text.clone();
		let mut result_sig  = wpm_result.clone();

		use_resource(move || {
			let state_snapshot = state_sig();

			async move {
				if !matches!(state_snapshot, WpmState::Running) {
					return;
				}

				let mut remaining = rem_sig();
				while remaining > 0 {
					tokio::time::sleep(Duration::from_secs(1)).await;
					remaining -= 1;
					rem_sig.set(remaining);
				}

				let duration   = duration_sig();
				let typed_str  = typed_sig();
				let target_str = target_sig();
				let res        = compute_wpm_result(&target_str, &typed_str, duration);

				result_sig.set(Some(res));
				state_sig.set(WpmState::Finished);
			}
		});
	}


	 // ── WPM rendering convenience ─────────────────────────────────────────
    let wpm_target_now = wpm_target_text();
    let wpm_typed_now = wpm_typed();
    let wpm_target_chars: Vec<char> = wpm_target_now.chars().collect();
    let wpm_typed_chars: Vec<char> = wpm_typed_now.chars().collect();

	// For word-based rendering in the WPM tab
	let wpm_target_words: Vec<String> = wpm_target_now
		.split_whitespace()
		.map(|s| s.to_string())
		.collect();

	let wpm_typed_words: Vec<String> = wpm_typed_now
		.split_whitespace()
		.map(|s| s.to_string())
		.collect();

    let current_wpm_state = wpm_state();
    let current_wpm_remaining = wpm_remaining();
    let current_wpm_duration = wpm_duration();
    let wpm_result_now = wpm_result();
	{
		// clone what we need into the effect
		let lang              = lang.clone();

		let mut current_index = current_index.clone();
		let mut typed_sig     = typed.clone();
		let mut active_pos_sig = active_pos.clone();
		let mut test_mode_sig  = test_mode.clone();
		let mut min_rank_sig   = min_rank.clone();
		let mut max_rank_sig   = max_rank.clone();
		let mut bounded_order_sig = bounded_order.clone();

		let mut wpm_target_sig = wpm_target_text.clone();
		let mut wpm_typed_sig  = wpm_typed.clone();
		let mut wpm_state_sig  = wpm_state.clone();
		let mut wpm_rem_sig    = wpm_remaining.clone();
		let mut wpm_result_sig = wpm_result.clone();
		let wpm_duration_sig   = wpm_duration.clone();

		use_effect(move || {
			// read lang so this effect re-runs whenever the language changes
			let _ = lang();

			// ── reset drill state ─────────────────────────────
			current_index.set(0);
			typed_sig.set(String::new());
			active_pos_sig.set(Vec::new());          // “all POS” again
			test_mode_sig.set(TestMode::Random);     // or whatever default you want
			min_rank_sig.set(1);
			max_rank_sig.set(250);
			bounded_order_sig.set(BoundedOrder::Random);

			// ── reset WPM state ───────────────────────────────
			wpm_target_sig.set(String::new());       // dump old language text
			wpm_typed_sig.set(String::new());
			wpm_state_sig.set(WpmState::Idle);

			let dur = wpm_duration_sig();
			wpm_rem_sig.set(dur);
			wpm_result_sig.set(None);
		});
	}

    rsx! {
        section { class: "flex justify-center",
            div { class: "w-full max-w-4xl flex gap-8",

                div { class: "flex-1 flex flex-col gap-4",
					Tabs {
						class: "w-full".to_string(),
						value: active_test_tab(),
						default_value: "drill".to_string(),
						on_value_change: move |v: String| {
							active_test_tab.set(v);

							// reset drill state
							typed.set(String::new());
							current_index.set(0);

							// reset wpm state
							wpm_typed.set(String::new());
							wpm_state.set(WpmState::Idle);
							wpm_remaining.set(wpm_duration());
							wpm_result.set(None);
						},


						TabList {
							TabTrigger {
								index: 0usize,
								value: "drill".to_string(),
								"Word drill"
							}
							TabTrigger {
								index: 1usize,
								value: "wpm".to_string(),
								"WPM"
							}
							// you can add more TabTrigger here for other tests
						}

						TabContent {
							index: 0usize,
							value: "drill".to_string(),

							// ⬇️ NO extra { } here – just children
							div { class: "relative flex justify-center min-h-[4rem]",

								input {
									r#type: "text",
									value: "{typed_now}",
									oninput: move |evt: FormEvent| {
										typed.set(evt.value());
									},
									onfocus: move |_| {
										input_focused.set(true);
									},
									onblur: move |_| {
										input_focused.set(false);
									},

									class: "absolute inset-0 w-full h-full opacity-0 cursor-text",
									style: "caret-color: transparent; color: transparent; border: none; outline: none; box-shadow: none; height: 55px; width: 33%;",
									autocomplete: "off",
									autocorrect: "off",
									spellcheck: "false",
									autofocus: "true",
								}

								{
									let is_focused = input_focused();
									let focus_class = if is_focused {
										"bg-slate-800/40 ring-1 ring-indigo-500/60 rounded px-3"
									} else {
										""
									};

									rsx! {
										div {
											class: "flex justify-center gap-4 text-3xl min-h-[4rem] \
													rounded-md transition-all duration-150 {focus_class}",

											if has_words {
												{
													let hint_map = hint_map.clone();
													let typed_chars_for_display = typed_chars_for_display.clone();

													target_chars.iter().enumerate().map(move |(i, ch)| {
														let base = if i < typed_chars_for_display.len() {
															if typed_chars_for_display[i] == *ch {
																"text-white"
															} else {
																"text-red-400"
															}
														} else {
															"text-gray-500"
														};

														let class = if is_focused {
															format!("{base} drop-shadow-[0_0_6px_rgba(129,140,248,0.6)]")
														} else {
															base.to_string()
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
											} else {
												div { class: "text-sm text-gray-400 text-center",
													"No words match the current filters / rank range."
												}
											}
										}
									}
								}
							}
							// Status + countdown indicator
							div { class: "flex justify-center items-center gap-5 text-sm mt-6",
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
							div { class: "text-xs text-gray-300 mt-1 flex flex-col items-center gap-1",
								if let Some(current) = &current_opt {
									div { class: "flex flex-col items-center gap-2",
										span { "Rank: #" b{"{current.rank}"} }
										span {
											"Translation: {current.en} \u{00A0}"
											if let Some(pos) = current.pos.clone() {
												span {
													class: "px-2 py-0.5 rounded-full bg-indigo-900 text-indigo-200 \
															text-[0.65rem] uppercase tracking-wide",
													"[{pos}]"
												}
											}
										}
									}

									if let Some(ex) = current.example.clone() {
										div {
											class: "text-[0.7rem] text-gray-400 italic text-center max-w-md",
											"{ex}"
										}
									}
								} else {
									div {
										class: "text-[0.7rem] text-gray-500 italic text-center max-w-md",
										"Adjust POS filters or rank range in Settings to see words here."
									}
								}
							}
						}
						TabContent{
							index: 1usize,
							value: "wpm".to_string(),
								div { class: "space-y-4 p-4",

									// Top bar: duration + status + start button
									div { class: "flex justify-between items-center text-xs text-gray-400",

										// duration selector
										div { class: "flex items-center gap-2",
											span { "Length:" }
											{
												let mut wpm_state_sig  = wpm_state.clone();
												let mut wpm_rem_sig    = wpm_remaining.clone();
												let mut wpm_typed_sig  = wpm_typed.clone();
												let mut wpm_target_sig = wpm_target_text.clone();
												let duration_sig       = wpm_duration.clone();
												let words_for_wpm      = filtered_words.clone();

												let label = if matches!(current_wpm_state, WpmState::Running) {
													"Restart"
												} else {
													"Start"
												};

												rsx! {
													button {
														class: "px-3 py-1 rounded bg-indigo-600 hover:bg-indigo-500 text-white text-[0.75rem]",
														onclick: move |_| {
															// build new target text, sized to duration *and* grid
															let dur = duration_sig();

															// assume up to ~200 WPM, ~5 chars/word
															let max_wpm            = 200usize;
															let avg_chars_per_word = 5usize;

															// duration-based estimate
															let duration_chars =
																max_wpm * avg_chars_per_word * (dur as usize) / 60;

															// grid-based requirement: e.g. 6 cols x 6 rows = 36 words
															let cols              = 6usize;
															let rows              = 6usize;
															let min_words_visible = cols * rows; // 36
															let grid_chars        = min_words_visible * avg_chars_per_word;

															// final char budget: enough for the grid AND the time AND not tiny
															let min_chars = duration_chars
																.max(grid_chars) // always enough to fill 6x6
																.max(500);       // general safety floor

															let text = build_wpm_text(&words_for_wpm, min_chars);
															wpm_target_sig.set(text);

															wpm_state_sig.set(WpmState::Running);
														},
														"{label}"
													}
												}
											}
										}

										// status + start / restart
										div { class: "flex items-center gap-3",
											span {
												match current_wpm_state {
													WpmState::Idle => "Status: idle",
													WpmState::Running => "Status: running",
													WpmState::Finished => "Status: finished",
												}
											}
											span { " Time left: {current_wpm_remaining}s" }

											{
												let mut wpm_state_sig = wpm_state.clone();
												let mut wpm_rem_sig = wpm_remaining.clone();
												let mut wpm_typed_sig = wpm_typed.clone();
												let mut wpm_target_sig = wpm_target_text.clone();
												let duration_sig = wpm_duration.clone();
												let words_for_wpm = filtered_words.clone();

												rsx!{
													button {
														class: "px-3 py-1 rounded bg-indigo-600 hover:bg-indigo-500 text-white text-[0.75rem]",
														onclick: move |_| {
															let dur = duration_sig();
															wpm_rem_sig.set(dur);
															wpm_typed_sig.set(String::new());
															wpm_state_sig.set(WpmState::Idle);

															// build new target text
															// build new target text, sized to duration
															let dur = duration_sig();

															// assume up to ~200 WPM, ~5 chars/word
															let max_wpm          = 200usize;
															let avg_chars_per_word = 5usize;

															// how many characters we want total
															let min_chars =
																max_wpm * avg_chars_per_word * (dur as usize) / 60;

															// safety floor so it's never tiny
															let min_chars = min_chars.max(500);

															// now build that much text
															let text = build_wpm_text(&words_for_wpm, min_chars);
															wpm_target_sig.set(text);

															
															wpm_state_sig.set(WpmState::Running);
														},
														if matches!(current_wpm_state, WpmState::Running) {
															"Restart"
														} else {
															"Start"
														}
													}
												}
											}
										}
									}

									// Invisible textarea + visible WPM text
									div { class: "flex justify-center ",

										textarea {
											value: "{wpm_typed_now}",
											oninput: move |evt: FormEvent| {
												wpm_typed.set(evt.value());
											},
											class: "absolute inset-0 w-full h-full opacity-0 cursor-text",
											style: "caret-color: transparent; color: transparent; border: none; outline: none; box-shadow: none;",
											autocomplete: "off",
											autocorrect: "off",
											spellcheck: "false",
										}

										div {
											class: "w-full max-w-2xl mx-auto text-xl leading-relaxed pointer-events-none",
											// Force a 4-column CSS grid regardless of Tailwind setup
											style: "display: grid; grid-template-columns: repeat(6, minmax(0, 1fr)); gap: 0.75rem 1rem; justify-items: center;",

											{
												let target_words = wpm_target_words.clone();
												let typed_words  = wpm_typed_words.clone();

												target_words.into_iter().enumerate().map(move |(wi, word)| {
													let typed_word = typed_words.get(wi).cloned().unwrap_or_default();

													let target_chars: Vec<char> = word.chars().collect();
													let typed_chars: Vec<char>  = typed_word.chars().collect();

													rsx! {
														// one grid cell per word
														div {
															class: "inline-flex gap-0.5",

															{
																let typed_chars = typed_chars.clone();

																target_chars.into_iter().enumerate().map(move |(ci, ch)| {
																	let class = if ci < typed_chars.len() {
																		if typed_chars[ci] == ch {
																			"text-green-300"
																		} else {
																			"text-red-400"
																		}
																	} else {
																		"text-gray-500"
																	};

																	rsx! {
																		span { class: "{class}", "{ch}" }
																	}
																})
															}
														}
													}
												})
											}
										}
									}

									// WPM results when finished
									if let (WpmState::Finished, Some(res)) = (current_wpm_state, wpm_result_now) {
										div { class: "mt-2 text-xs text-gray-300 flex flex-col items-center gap-1",
											span { class: "text-sm font-semibold text-indigo-300",
												"WPM: {res.wpm}"
											}
											span {
												"Words – correct: {res.correct_words}, incorrect: {res.incorrect_words}"
											}
											span {
												"Letters – correct: {res.correct_letters}, incorrect: {res.incorrect_letters}"
											}
										}
									}
								}
							}
						}
						//settings
                    if show_settings() {
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
									show_settings.set(false);
								},
								"Hide"
							}
						}
					} else {
                        div {class:"flex justify-center mb-3",
                            Tooltip { 
                                TooltipTrigger { class:"flex justify-center",
                                    button{ class:"text-center opacity-50 hover:opacity-100 transition-all duration-300 hover:scale-105 hover:cursor-pointer", onclick: move |_| {
                                            show_settings.set(true);
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
    

