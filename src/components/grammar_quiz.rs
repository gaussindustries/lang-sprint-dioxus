// src/components/grammar_quiz.rs
//
// The grammar "Practice" mode: walks a language's authored Drill bank one
// question at a time, grades with the shared typing core (type-in) or exact
// match (choice), and logs Skill::Grammar evidence per language. Keyed on the
// language by the caller, so switching language resets the quiz cleanly.

use dioxus::events::KeyboardEvent;
use dioxus::prelude::*;

use crate::components::drills::typing_engine::grade_answer;
use crate::grammar::{doc_for, Answer};
use crate::learner::{now_ms, use_learner};
use crate::learning::{Evidence, Skill, Source};

fn drill_id(lang: &str, prompt: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    prompt.hash(&mut h);
    format!("{lang}:grammar:{:x}", h.finish())
}

#[component]
pub fn GrammarQuiz(lang: String) -> Element {
    let learner = use_learner();
    let drills = doc_for(&lang).drills;
    let total = drills.len();

    let mut idx = use_signal(|| 0usize);
    let mut typed = use_signal(String::new);
    // feedback: (was_correct, correct-answer display, optional note)
    let mut feedback = use_signal(|| None::<(bool, String, Option<String>)>);
    let mut scores = use_signal(Vec::<f32>::new);
    let mut started = use_signal(now_ms);

    let i = idx();
    let cur = drills.get(i).cloned();
    let done = total > 0 && i >= total;
    let fb = feedback();
    let typed_now = typed();
    let sc = scores();
    let avg_pct = if sc.is_empty() {
        0
    } else {
        ((sc.iter().sum::<f32>() / sc.len() as f32) * 100.0).round() as i32
    };
    let progress = if total > 0 {
        format!("Question {} of {}", (i + 1).min(total), total)
    } else {
        String::new()
    };
    let fb_view: Option<(String, Option<String>)> =
        fb.clone().map(|(ok, correct_disp, note_opt)| {
            let mark = if ok {
                "✓ Correct".to_string()
            } else {
                format!("✗ Not quite — {correct_disp}")
            };
            (mark, note_opt)
        });

    let next = move |_| {
        idx.with_mut(|i| *i += 1);
        typed.set(String::new());
        feedback.set(None);
        started.set(now_ms());
    };
    let restart = move |_| {
        idx.set(0);
        typed.set(String::new());
        feedback.set(None);
        scores.set(Vec::new());
        started.set(now_ms());
    };

    rsx! {
        div { style: "max-width:42rem; margin:0 auto;",

            if total == 0 {
                div { style: "text-align:center; color:#9ca3af; padding:2.5rem 0; font-size:0.95rem;",
                    "No grammar drills written for this language yet."
                }
            } else if done {
                div { style: "text-align:center; padding:2rem 0;",
                    div { style: "font-size:1.1rem; margin-bottom:0.5rem;", "Done." }
                    div { style: "font-size:2rem; color:#a5b4fc; margin-bottom:0.25rem;", "{avg_pct}%" }
                    div { style: "font-size:0.85rem; color:#9ca3af; margin-bottom:1.25rem;", "across {total} questions" }
                    button {
                        style: "padding:0.5rem 1.1rem; border-radius:0.5rem; background:#4f46e5; color:#fff; cursor:pointer; border:none;",
                        onclick: restart,
                        "Again"
                    }
                }
            } else if let Some(d) = cur.clone() {
                // wrapped in a block-expression node so the per-question `let`s are
                // legal (rsx if/for bodies take nodes, not statements)
                {
                    let prompt = d.prompt.clone();
                    let note = d.note.clone();
                    rsx! {
                        div {
                            div { style: "font-size:0.75rem; color:#6b7280; margin-bottom:0.4rem;", "{progress}" }
                            p { style: "font-size:1.15rem; color:#f3f4f6; line-height:1.5; margin-bottom:1rem;", "{prompt}" }

                            {
                                match d.answer {
                                    Answer::TypeIn(answers) => rsx! {
                                        input {
                                            style: "width:100%; padding:0.55rem 0.7rem; border-radius:0.5rem; background:#111827; color:#fff; border:1px solid #374151; font-size:1.05rem;",
                                            placeholder: "type your answer",
                                            value: "{typed_now}",
                                            disabled: fb.is_some(),
                                            autocomplete: "off",
                                            autocorrect: "off",
                                            spellcheck: "false",
                                            oninput: move |e| typed.set(e.value()),
                                            onkeydown: {
                                                let answers = answers.clone();
                                                let prompt = prompt.clone();
                                                let note = note.clone();
                                                let lang = lang.clone();
                                                let learner = learner.clone();
                                                move |e: KeyboardEvent| {
                                                    if e.key().to_string() == "Enter"
                                                        && feedback().is_none()
                                                        && !typed().trim().is_empty()
                                                    {
                                                        let score = grade_answer(&typed(), &answers);
                                                        let latency = now_ms().saturating_sub(started()) as u32;
                                                        learner.emit(Evidence::new(
                                                            now_ms(), lang.clone(), drill_id(&lang, &prompt),
                                                            Skill::Grammar, score, 0.0, latency, Source::Grammar,
                                                        ));
                                                        scores.with_mut(|v| v.push(score));
                                                        feedback.set(Some((score >= 0.85, answers.join(" / "), note.clone())));
                                                    }
                                                }
                                            },
                                        }
                                        if fb.is_none() {
                                            button {
                                                style: "margin-top:0.75rem; padding:0.5rem 1.1rem; border-radius:0.5rem; background:#4f46e5; color:#fff; cursor:pointer; border:none;",
                                                onclick: {
                                                    let answers = answers.clone();
                                                    let prompt = prompt.clone();
                                                    let note = note.clone();
                                                    let lang = lang.clone();
                                                    let learner = learner.clone();
                                                    move |_| {
                                                        if feedback().is_none() && !typed().trim().is_empty() {
                                                            let score = grade_answer(&typed(), &answers);
                                                            let latency = now_ms().saturating_sub(started()) as u32;
                                                            learner.emit(Evidence::new(
                                                                now_ms(), lang.clone(), drill_id(&lang, &prompt),
                                                                Skill::Grammar, score, 0.0, latency, Source::Grammar,
                                                            ));
                                                            scores.with_mut(|v| v.push(score));
                                                            feedback.set(Some((score >= 0.85, answers.join(" / "), note.clone())));
                                                        }
                                                    }
                                                },
                                                "Check"
                                            }
                                        }
                                    },
                                    Answer::Choice { options, correct } => {
                                        let correct_text = options.get(correct).cloned().unwrap_or_default();
                                        rsx! {
                                            div { style: "display:flex; flex-direction:column; gap:0.5rem;",
                                                for (oi, opt) in options.iter().cloned().enumerate() {
                                                    button {
                                                        key: "{oi}",
                                                        disabled: fb.is_some(),
                                                        style: "text-align:left; padding:0.55rem 0.8rem; border-radius:0.5rem; border:1px solid #374151; background:#111827; color:#e5e7eb; cursor:pointer; font-size:1rem;",
                                                        onclick: {
                                                            let prompt = prompt.clone();
                                                            let note = note.clone();
                                                            let lang = lang.clone();
                                                            let learner = learner.clone();
                                                            let correct_text = correct_text.clone();
                                                            move |_| {
                                                                if feedback().is_none() {
                                                                    let score = if oi == correct { 1.0 } else { 0.0 };
                                                                    let latency = now_ms().saturating_sub(started()) as u32;
                                                                    learner.emit(Evidence::new(
                                                                        now_ms(), lang.clone(), drill_id(&lang, &prompt),
                                                                        Skill::Grammar, score, 0.0, latency, Source::Grammar,
                                                                    ));
                                                                    scores.with_mut(|v| v.push(score));
                                                                    feedback.set(Some((oi == correct, correct_text.clone(), note.clone())));
                                                                }
                                                            }
                                                        },
                                                        "{opt}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if let Some((mark, note_opt)) = fb_view.clone() {
                                div { style: "margin-top:1rem;",
                                    p { style: "font-size:0.95rem; color:#e5e7eb;", "{mark}" }
                                    if let Some(nt) = note_opt {
                                        p { style: "font-size:0.85rem; color:#9ca3af; margin-top:0.35rem; line-height:1.5;", "{nt}" }
                                    }
                                    button {
                                        style: "margin-top:0.75rem; padding:0.5rem 1.1rem; border-radius:0.5rem; background:#4f46e5; color:#fff; cursor:pointer; border:none;",
                                        onclick: next,
                                        "Next →"
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
