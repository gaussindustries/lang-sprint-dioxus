// src/views/reading.rs
//
// Reading / type-the-meaning drill. Paste an L2 sentence; the drill finds the
// words it knows — citation forms, plus inflected forms of any noun that carries
// a declension table — and quizzes their meaning. Evidence emitted:
// VocabRecognition per word, Reading for the finished sentence (which floors
// Script/Vocab/Grammar through the model). A "just read" toggle logs a lighter
// Reading signal instead of quizzing.
//
// The paste box is the stand-in for the PDF ingest pipeline, which will later
// hand this same component sentences automatically.

use std::collections::HashMap;

use dioxus::prelude::*;

use crate::assets::freq_json_for;
use crate::components::drills::typing_engine::{accepted_answers, grade_answer, tokenize};
use crate::learner::{now_ms, use_learner};
use crate::learning::{Evidence, Skill, Source};
use crate::models::lexicon::{GeoCase, LexEntry, Lexicon};

#[derive(Clone, PartialEq)]
struct QuizTok {
    surface: String,
    gloss: String,
    accepted: Vec<String>,
}

fn sentence_id(lang: &str, s: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.trim().hash(&mut h);
    format!("{lang}:sentence:{:x}", h.finish())
}

#[component]
pub fn ReadingPage() -> Element {
    let lang = use_context::<Signal<String>>();
    let learner = use_learner();

    // lexicon for the active language
    let lex_res = use_resource(move || {
        let l = lang.read().clone();
        async move { Lexicon::from_json(freq_json_for(&l)).unwrap_or_default() }
    });

    // surface form -> entry, incl. every declined form of nouns that have a table
    let by_surface = use_memo(move || {
        let mut map: HashMap<String, LexEntry> = HashMap::new();
        let lexicon = lex_res.read().clone().unwrap_or_default();
        for e in lexicon.all() {
            map.entry(e.head().to_string()).or_insert_with(|| e.clone());
            if let Some(d) = &e.declension {
                for c in GeoCase::ALL {
                    let f = d.forms(c);
                    for form in [&f.singular, &f.plural, &f.with_noun] {
                        let form = form.trim();
                        if !form.is_empty() {
                            map.entry(form.to_string()).or_insert_with(|| e.clone());
                        }
                    }
                }
            }
        }
        map
    });

    let mut sentence_input = use_signal(String::new);
    let mut loaded = use_signal(|| false);
    let mut quiz = use_signal(Vec::<QuizTok>::new);
    let mut idx = use_signal(|| 0usize);
    let mut typed = use_signal(String::new);
    let mut feedback = use_signal(|| None::<(bool, String, i32)>);
    let mut scores = use_signal(Vec::<f32>::new);
    let mut meaning_mode = use_signal(|| true);
    let mut started = use_signal(|| 0u64);

    // ── handlers ────────────────────────────────────────────────────────────
    let load = move |_| {
        let s = sentence_input();
        let map = by_surface.read();
        let mut q: Vec<QuizTok> = Vec::new();
        for t in tokenize(&s) {
            if let Some(e) = map.get(&t) {
                let acc = accepted_answers(&e.en);
                if !acc.is_empty() && !q.iter().any(|x| x.surface == t) {
                    q.push(QuizTok {
                        surface: t.clone(),
                        gloss: e.en.clone(),
                        accepted: acc,
                    });
                }
            }
        }
        quiz.set(q);
        idx.set(0);
        typed.set(String::new());
        feedback.set(None);
        scores.set(Vec::new());
        started.set(now_ms());
        loaded.set(true);
    };

    let check = {
        let learner = learner.clone();
        move |_| {
            let q = quiz();
            let i = idx();
            if i >= q.len() || feedback().is_some() {
                return;
            }
            let tok = q[i].clone();
            let score = grade_answer(&typed(), &tok.accepted);
            let now = now_ms();
            let latency = now.saturating_sub(started()) as u32;
            let l = lang.read().clone();
            learner.emit(Evidence::new(
                now,
                l.clone(),
                format!("{l}:word:{}", tok.surface),
                Skill::VocabRecognition,
                score,
                0.0,
                latency,
                Source::Reading,
            ));
            let mut sc = scores();
            sc.push(score);
            scores.set(sc);
            feedback.set(Some((
                score >= 0.6,
                tok.gloss.clone(),
                (score * 100.0).round() as i32,
            )));
        }
    };

    let next = {
        let learner = learner.clone();
        move |_| {
            let q = quiz();
            let i = idx();
            if i + 1 >= q.len() {
                let sc = scores();
                let avg = if sc.is_empty() {
                    0.0
                } else {
                    sc.iter().sum::<f32>() / sc.len() as f32
                };
                let l = lang.read().clone();
                learner.emit(Evidence::new(
                    now_ms(),
                    l.clone(),
                    sentence_id(&l, &sentence_input()),
                    Skill::Reading,
                    avg,
                    0.0,
                    0,
                    Source::Reading,
                ));
            }
            idx.set(i + 1);
            typed.set(String::new());
            feedback.set(None);
            started.set(now_ms());
        }
    };

    let read_done = {
        let learner = learner.clone();
        move |_| {
            let l = lang.read().clone();
            learner.emit(Evidence::new(
                now_ms(),
                l.clone(),
                sentence_id(&l, &sentence_input()),
                Skill::Reading,
                0.5, // "encountered", not tested
                0.0,
                0,
                Source::Reading,
            ));
            quiz.set(Vec::new());
            loaded.set(false);
            sentence_input.set(String::new());
        }
    };

    let new_sentence = move |_| {
        quiz.set(Vec::new());
        idx.set(0);
        typed.set(String::new());
        feedback.set(None);
        scores.set(Vec::new());
        loaded.set(false);
        sentence_input.set(String::new());
    };

    // ── precomputed view values (no method calls inside rsx) ─────────────────
    let q = quiz();
    let total = q.len();
    let i = idx();
    let cur = q.get(i).cloned();
    let done = total > 0 && i >= total;
    let mode_meaning = meaning_mode();
    let loaded_v = loaded();
    let sentence = sentence_input();
    let sc = scores();
    let avg_pct = if sc.is_empty() {
        0
    } else {
        ((sc.iter().sum::<f32>() / sc.len() as f32) * 100.0).round() as i32
    };
    let progress = if total > 0 {
        format!("Word {} of {}", (i + 1).min(total), total)
    } else {
        String::new()
    };
    let fb_view: Option<(&'static str, String, String)> =
        feedback().map(|(correct, gloss, pct)| {
            (
                if correct {
                    "✓ Got it"
                } else {
                    "✗ Not quite"
                },
                gloss,
                format!("{pct}%"),
            )
        });
    let known: Vec<(String, String)> = q
        .iter()
        .map(|t| (t.surface.clone(), t.gloss.clone()))
        .collect();

    let geo = "'Noto Serif Georgian','Noto Serif',Georgia,serif";

    rsx! {
        div { class: "min-h-screen bg-gray-800 text-white p-8",
            div { class: "max-w-2xl mx-auto text-center",
                h1 { class: "text-2xl font-semibold mb-1", "Reading" }
                p { class: "text-sm text-gray-400 mb-5",
                    "Paste a sentence in your target language. Known words are quizzed; the rest is context."
                }

                // mode toggle
                div { class: "flex justify-center gap-2 mb-5",
                    button {
                        class: if mode_meaning { "px-3 py-1.5 rounded text-sm bg-indigo-600" } else { "px-3 py-1.5 rounded text-sm bg-gray-700" },
                        onclick: move |_| meaning_mode.set(true),
                        "Type meanings"
                    }
                    button {
                        class: if mode_meaning { "px-3 py-1.5 rounded text-sm bg-gray-700" } else { "px-3 py-1.5 rounded text-sm bg-indigo-600" },
                        onclick: move |_| meaning_mode.set(false),
                        "Just read"
                    }
                }

                if !loaded_v {
                    // paste + load
                    div { class:"justify-center items-center",
                        textarea {
                            class: "w-full p-3 rounded bg-gray-900 text-white text-lg focus:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500",
                            style: "font-family:{geo};",
                            placeholder: "კაცი ქუჩაში დადიოდა.",
                            value: "{sentence}",
                            oninput: move |e| sentence_input.set(e.value()),
                        }
                        button {
                            class: " mt-3 px-4 py-2 rounded bg-indigo-600 hover:bg-indigo-500 text-sm",
                            onclick: load,
                            "Load sentence"
                        }
                    }
                } else if total == 0 {
                    // loaded but nothing recognized
                    div {
                        p { class: "mb-2", style: "font-family:{geo};", "{sentence}" }
                        p { class: "text-sm text-gray-400 mb-4",
                            "No dictionary words recognized. Inflected forms are matched only for nouns that have a declension table — try another sentence, or add the words to the dictionary."
                        }
                        button { class: "px-4 py-2 rounded bg-gray-700 text-sm", onclick: new_sentence, "New sentence" }
                    }
                } else {
                    // the sentence, for context
                    div { class: "p-3 mb-4 rounded bg-gray-900 text-lg", style: "font-family:{geo};", "{sentence}" }

                    if mode_meaning {
                        if done {
                            div {
                                p { class: "text-lg mb-3", "Sentence done — {avg_pct}% across {total} words." }
                                button { class: "px-4 py-2 rounded bg-indigo-600 hover:bg-indigo-500 text-sm", onclick: new_sentence, "New sentence" }
                            }
                        } else if let Some(tok) = cur.clone() {
                            div {
                                p { class: "text-xs text-gray-500", "{progress}" }
                                p { class: "text-3xl my-3", style: "font-family:{geo};", "{tok.surface}" }
                                input {
                                    class: "w-full p-2 rounded bg-gray-900 text-white focus:outline-none focus-visible:ring-2 focus-visible:ring-indigo-500",
                                    placeholder: "meaning in English",
                                    value: "{typed}",
                                    oninput: move |e| typed.set(e.value()),
                                }
                                if let Some((mark, gloss, pct)) = fb_view.clone() {
                                    div { class: "mt-3",
                                        p { class: "text-sm", "{mark} — {gloss} ({pct})" }
                                        button { class: "mt-2 px-4 py-2 rounded bg-indigo-600 hover:bg-indigo-500 text-sm", onclick: next, "Next →" }
                                    }
                                } else {
                                    button { class: "mt-3 px-4 py-2 rounded bg-indigo-600 hover:bg-indigo-500 text-sm", onclick: check, "Check" }
                                }
                            }
                        }
                    } else {
                        // just read: reveal glosses, log a light Reading signal
                        div {
                            p { class: "text-sm text-gray-400 mb-2", "Words in this sentence you've seen before:" }
                            div { class: "space-y-1 mb-4",
                                for (surface, gloss) in known.iter().cloned() {
                                    div { class: "flex justify-between text-sm",
                                        span { style: "font-family:{geo};", "{surface}" }
                                        span { class: "text-gray-400", "{gloss}" }
                                    }
                                }
                            }
                            button { class: "px-4 py-2 rounded bg-indigo-600 hover:bg-indigo-500 text-sm", onclick: read_done, "Done reading" }
                        }
                    }
                }
            }
        }
    }
}
