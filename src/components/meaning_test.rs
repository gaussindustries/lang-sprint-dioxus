// src/components/meaning_test.rs
//
// "WPM, but type the meaning." A word streams in; you type its meaning and press
// Enter. The direction toggle picks the axis:
//   - answer in English  → recognition (see L2 word, produce the meaning)
//   - answer in target    → production  (see the meaning, produce the L2 word)
// Graded by the shared typing_engine core (exact + fuzzy), scored as correct/min,
// and every answer is logged to the learner at full difficulty (real recall).

use dioxus::events::{FormEvent, KeyboardEvent};
use dioxus::prelude::*;
use rand::Rng;

use crate::components::drills::typing_engine::{accepted_answers, grade_answer};
use crate::models::lexicon::LexEntry;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Running,
    Done,
}

/// Title-case a language code for display ("georgian" -> "Georgian").
fn nice(lang: &str) -> String {
    let mut c = lang.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => "the target language".to_string(),
    }
}

#[component]
pub fn MeaningTest(words: Vec<LexEntry>, lang: Signal<String>) -> Element {
    let mut phase = use_signal(|| Phase::Idle);
    let mut to_english = use_signal(|| true); // true: L2->EN (recognition); false: EN->L2 (production)
    let mut duration = use_signal(|| 60u64);
    let mut remaining = use_signal(|| 60u64);
    let mut idx = use_signal(|| 0usize);
    let mut typed = use_signal(|| String::new());
    let mut correct = use_signal(|| 0u32);
    let mut attempts = use_signal(|| 0u32);
    let mut prompt_start = use_signal(|| crate::learner::now_ms());
    let mut last = use_signal(|| None::<(bool, String, String)>);
    let mut focus_target = use_signal(|| None::<std::rc::Rc<MountedData>>);

    let learner = crate::learner::use_learner();

    let pool_len = words.len();
    let has_words = pool_len > 0;

    // reset to setup screen when the language changes
    use_effect(move || {
        let _ = lang();
        phase.set(Phase::Idle);
        typed.set(String::new());
        idx.set(0);
        correct.set(0);
        attempts.set(0);
        last.set(None);
    });

    // countdown while Running
    use_effect(move || {
        let p = phase();
        if p != Phase::Running {
            return;
        }
        spawn(async move {
            let mut r = remaining();
            while r > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                if phase() != Phase::Running {
                    return;
                }
                r -= 1;
                remaining.set(r);
            }
            phase.set(Phase::Done);
        });
    });

    // current card
    let cur = if has_words {
        Some(words[idx().min(pool_len - 1)].clone())
    } else {
        None
    };
    let dir_en = to_english();
    let target_name = nice(&lang());

    let (prompt_label, prompt_text, answer_hint) = match &cur {
        Some(e) if dir_en => (
            target_name.clone(),
            e.word.clone(),
            "type the meaning in English".to_string(),
        ),
        Some(e) => (
            "English".to_string(),
            e.en.clone(),
            format!("type the word in {target_name}"),
        ),
        None => (String::new(), String::new(), String::new()),
    };
    let cur_for_submit = cur.clone();

    // snapshots (no method calls allowed inside rsx interpolation)
    let typed_now = typed();
    let rem_now = remaining();
    let cor = correct();
    let att = attempts();
    let acc_pct = if att > 0 {
        (cor as f32 / att as f32 * 100.0).round() as i32
    } else {
        0
    };
    let per_min = ((cor as f32 * 60.0 / duration().max(1) as f32) * 10.0).round() / 10.0;
    let dir_explain = if dir_en {
        format!("{target_name} word → English meaning (recognition)")
    } else {
        format!("English meaning → {target_name} word (production)")
    };

    rsx! {
        div { class: "w-full max-w-xl mx-auto flex flex-col items-center gap-4 mt-2",

            if !has_words {
                div { class: "text-sm text-gray-400 text-center",
                    "No words match the current filters / rank range."
                }
            } else if phase() == Phase::Idle {
                div { class: "text-center text-gray-300 text-sm",
                    "See a word, type its meaning. Press Enter to submit."
                }

                div { class: "flex items-center gap-2",
                    span { class: "text-xs text-gray-400", "Answer in:" }
                    button {
                        class: format!("px-3 py-1 rounded text-sm {}", if dir_en { "bg-indigo-600 text-white" } else { "bg-gray-700 text-gray-300" }),
                        onclick: move |_| to_english.set(true),
                        "English"
                    }
                    button {
                        class: format!("px-3 py-1 rounded text-sm {}", if !dir_en { "bg-indigo-600 text-white" } else { "bg-gray-700 text-gray-300" }),
                        onclick: move |_| to_english.set(false),
                        "{target_name}"
                    }
                }
                div { class: "text-xs text-gray-500", "{dir_explain}" }

                div { class: "flex items-center gap-2",
                    span { class: "text-xs text-gray-400", "Time:" }
                    for secs in [30u64, 60, 120] {
                        button {
                            key: "{secs}",
                            class: format!("px-3 py-1 rounded text-sm {}", if duration() == secs { "bg-indigo-600 text-white" } else { "bg-gray-700 text-gray-300" }),
                            onclick: move |_| {
                                duration.set(secs);
                                remaining.set(secs);
                            },
                            "{secs}s"
                        }
                    }
                }

                button {
                    class: "mt-2 px-5 py-2 rounded bg-green-600 hover:bg-green-500 text-white font-semibold",
                    onclick: move |_| {
                        if pool_len == 0 {
                            return;
                        }
                        correct.set(0);
                        attempts.set(0);
                        last.set(None);
                        remaining.set(duration());
                        typed.set(String::new());
                        let mut rng = rand::rng();
                        idx.set(rng.random_range(0..pool_len));
                        prompt_start.set(crate::learner::now_ms());
                        phase.set(Phase::Running);
                    },
                    "Start"
                }
            } else if phase() == Phase::Running {
                div { class: "flex justify-between w-full text-sm",
                    span { class: "text-indigo-300 font-semibold", "{rem_now}s" }
                    span { class: "text-gray-400", "{cor} / {att}" }
                }

                div { class: "w-full rounded-lg bg-gray-900/60 border border-gray-700 p-6 flex flex-col items-center gap-2",
                    span { class: "text-[0.65rem] uppercase tracking-wide text-gray-500", "{prompt_label}" }
                    span { class: "text-3xl", "{prompt_text}" }
                    span { class: "text-xs text-gray-500", "{answer_hint}" }
                }

                input {
                    r#type: "text",
                    value: "{typed_now}",
                    autocomplete: "off",
                    autocorrect: "off",
                    spellcheck: "false",
                    class: "w-full text-center text-2xl bg-transparent border-b-2 border-gray-600 focus:border-indigo-400 outline-none py-2",
                    oninput: move |e: FormEvent| typed.set(e.value()),
                    onmounted: move |e| {
                        let el = e.data();
                        focus_target.set(Some(el.clone()));
                        spawn(async move { let _ = el.set_focus(true).await; });
                    },
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key().to_string() == "Enter" && !typed().trim().is_empty() {
                            if let Some(entry) = cur_for_submit.clone() {
                                let ans = typed();
                                let accepted = if dir_en {
                                    accepted_answers(&entry.en)
                                } else {
                                    accepted_answers(&entry.word)
                                };
                                let g = grade_answer(&ans, &accepted);
                                let was_correct = g >= 0.85;
                                let latency = crate::learner::now_ms().saturating_sub(prompt_start()) as u32;
                                let headword = entry.word.clone();
                                learner.emit_all(crate::learner::meaning_evidence(&lang(), dir_en, &headword, g, latency));
                                attempts.with_mut(|a| *a += 1);
                                if was_correct {
                                    correct.with_mut(|c| *c += 1);
                                }
                                last.set(Some((was_correct, ans.clone(), accepted.join(", "))));
                                if pool_len > 0 {
                                    let mut rng = rand::rng();
                                    let cur_i = idx();
                                    let mut n = rng.random_range(0..pool_len);
                                    if pool_len > 1 && n == cur_i {
                                        n = (n + 1) % pool_len;
                                    }
                                    idx.set(n);
                                }
                                typed.set(String::new());
                                prompt_start.set(crate::learner::now_ms());
                            }
                        }
                    },
                }

                if let Some((ok, your, acc)) = last() {
                    if ok {
                        div { class: "text-sm text-center text-green-400", "✓ correct" }
                    } else {
                        div { class: "text-sm text-center text-red-400",
                            "✗ you typed '{your}' — answer: {acc}"
                        }
                    }
                }

                button {
                    class: "text-xs text-gray-500 hover:text-gray-300 mt-1",
                    onclick: move |_| phase.set(Phase::Done),
                    "End now"
                }
            } else {
                div { class: "w-full rounded-lg bg-gray-900/60 border border-gray-700 p-6 flex flex-col items-center gap-2",
                    div { class: "text-lg font-semibold", "Time!" }
                    div { class: "text-3xl text-indigo-300", "{cor} / {att}" }
                    div { class: "text-sm text-gray-400", "{acc_pct}% correct · {per_min} per minute" }
                }
                div { class: "flex gap-3 mt-2",
                    button {
                        class: "px-4 py-2 rounded bg-green-600 hover:bg-green-500 text-white",
                        onclick: move |_| {
                            if pool_len == 0 {
                                return;
                            }
                            correct.set(0);
                            attempts.set(0);
                            last.set(None);
                            remaining.set(duration());
                            typed.set(String::new());
                            let mut rng = rand::rng();
                            idx.set(rng.random_range(0..pool_len));
                            prompt_start.set(crate::learner::now_ms());
                            phase.set(Phase::Running);
                        },
                        "Again"
                    }
                    button {
                        class: "px-4 py-2 rounded bg-gray-700 hover:bg-gray-600 text-gray-200",
                        onclick: move |_| phase.set(Phase::Idle),
                        "Change settings"
                    }
                }
            }
        }
    }
}
