// src/views/dashboard.rs
//
// Per-language mastery dashboard. The active language comes from the same
// app-wide context the navbar drives; the radar + bars show ONLY that language's
// folded estimate (the brain now buckets evidence by language, so no bleed).
// Language chips switch the active language (and the rest of the app follows).

use dioxus::prelude::*;
use strum::IntoEnumIterator;

use crate::learner::{now_ms, use_learner};
use crate::learning::Skill;

fn short_label(s: Skill) -> &'static str {
    match s {
        Skill::ScriptSound => "Script",
        Skill::Listening => "Listening",
        Skill::VocabRecognition => "Vocab·rec",
        Skill::VocabProduction => "Vocab·prod",
        Skill::Grammar => "Grammar",
        Skill::Reading => "Reading",
    }
}

fn nice(lang: &str) -> String {
    let mut c = lang.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => "—".to_string(),
    }
}

#[component]
pub fn DashboardPage() -> Element {
    let active = use_context::<Signal<String>>();
    let learner = use_learner();
    let now = now_ms();
    let model = learner.model();

    let lang = active();
    let lang_display = nice(&lang);

    let mastery = model.mastery(&lang, now);
    let events = model.event_count(&lang);
    let items = model.item_count(&lang);
    let due = model.due(&lang, now, 0.9).len();

    // language chips: everything with data, plus the active one (so it always shows)
    let mut lang_codes = model.languages();
    if !lang_codes.iter().any(|l| l == &lang) {
        lang_codes.push(lang.clone());
    }
    lang_codes.sort();
    lang_codes.dedup();
    let chips: Vec<(String, String, String, bool)> = lang_codes
        .iter()
        .map(|code| {
            let mean_txt = match model.mean_mastery(code, now) {
                Some(v) => format!("{}%", (v * 100.0).round() as i32),
                None => "—".to_string(),
            };
            (code.clone(), nice(code), mean_txt, code == &lang)
        })
        .collect();

    // ── radar geometry (computed here; rsx only interpolates finished values) ──
    let skills: Vec<Skill> = Skill::iter().collect();
    let n = skills.len();
    let size = 360i32;
    let cx = 180.0f32;
    let cy = 180.0f32;
    let r = 120.0f32;
    let angle = |i: usize| -> f32 { (-90.0 + 360.0 / n as f32 * i as f32).to_radians() };
    let point = |i: usize, v: f32| -> (i32, i32) {
        let a = angle(i);
        (
            (cx + r * v * a.cos()).round() as i32,
            (cy + r * v * a.sin()).round() as i32,
        )
    };

    let vals: Vec<f32> = skills
        .iter()
        .map(|s| {
            mastery
                .get(s)
                .copied()
                .flatten()
                .unwrap_or(0.0)
                .clamp(0.0, 1.0)
        })
        .collect();

    let rings: Vec<String> = [0.25f32, 0.5, 0.75, 1.0]
        .iter()
        .map(|&lvl| {
            (0..n)
                .map(|i| {
                    let (x, y) = point(i, lvl);
                    format!("{x},{y}")
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();

    let spokes: Vec<(i32, i32)> = (0..n).map(|i| point(i, 1.0)).collect();
    let data_pts: String = (0..n)
        .map(|i| {
            let (x, y) = point(i, vals[i]);
            format!("{x},{y}")
        })
        .collect::<Vec<_>>()
        .join(" ");
    let vertices: Vec<(i32, i32)> = (0..n).map(|i| point(i, vals[i])).collect();

    let labels: Vec<(i32, i32, &'static str, &'static str)> = (0..n)
        .map(|i| {
            let a = angle(i);
            let lx = (cx + (r + 22.0) * a.cos()).round() as i32;
            let ly = (cy + (r + 22.0) * a.sin()).round() as i32 + 4;
            let cosv = a.cos();
            let anchor = if cosv > 0.3 {
                "start"
            } else if cosv < -0.3 {
                "end"
            } else {
                "middle"
            };
            (lx, ly, anchor, short_label(skills[i]))
        })
        .collect();

    // per-axis bars below the radar
    let rows: Vec<(String, String, i32)> = skills
        .iter()
        .map(|s| {
            let label = s.label().to_string();
            match mastery.get(s).copied().flatten() {
                Some(v) => {
                    let p = (v * 100.0).round().clamp(0.0, 100.0) as i32;
                    (label, format!("{p}%"), p)
                }
                None => (label, "untested".to_string(), 0),
            }
        })
        .collect();

    let empty = events == 0;

    rsx! {
        div { class: "min-h-screen bg-gray-800 text-white p-8",
            div { class: "max-w-2xl mx-auto",

                // language chip — read-only (switch languages from the navbar select)
                                div { class: "flex flex-wrap gap-2 mb-6",
                                    for (code, name, mean_txt, is_active) in chips.iter().cloned() {
                                        div {
                                            key: "{code}",
                                            class: format!(
                                                "px-3 py-1.5 rounded-lg border text-sm flex items-center gap-2 {}",
                                                if is_active { "bg-indigo-600 border-indigo-400 text-white" }
                                                else { "bg-gray-900/40 border-gray-700 text-gray-400" }
                                            ),
                                            span { "{name}" }
                                            span { class: "text-xs opacity-70", "{mean_txt}" }
                                        }
                                    }
                                }

                // active-language card
                div { class: "rounded-xl bg-gray-900/40 border border-gray-700 p-6 ",
                    div { class: "text-center mb-4",
                        h2 { class: "text-lg font-semibold text-indigo-200", "{lang_display}" }

                        p { class: "text-xs text-gray-400",
                            "{events} observations · {items} items · {due} due"
                        }

                    }

                    if empty {
                        div { class: "text-center text-sm text-gray-400 py-10",
                            "No evidence for {lang_display} yet — run a drill in this language and it'll show up here."
                        }
                    } else {
                        div { class: "flex justify-center",
                            svg {
                                width: "{size}",
                                height: "{size}",
                                view_box: "0 0 {size} {size}",

                                for ring in rings.iter().cloned() {
                                    polygon {
                                        points: "{ring}",
                                        fill: "none",
                                        stroke: "rgba(148,163,184,0.18)",
                                        "stroke-width": "1",
                                    }
                                }
                                for (x, y) in spokes.iter().cloned() {
                                    line {
                                        x1: "{cx}", y1: "{cy}", x2: "{x}", y2: "{y}",
                                        stroke: "rgba(148,163,184,0.22)", "stroke-width": "1",
                                    }
                                }
                                polygon {
                                    points: "{data_pts}",
                                    fill: "rgba(99,102,241,0.35)",
                                    stroke: "rgb(129,140,248)",
                                    "stroke-width": "2",
                                    "stroke-linejoin": "round",
                                }
                                for (x, y) in vertices.iter().cloned() {
                                    circle { cx: "{x}", cy: "{y}", r: "3", fill: "rgb(129,140,248)" }
                                }
                                for (lx, ly, anchor, label) in labels.iter().cloned() {
                                    text {
                                        x: "{lx}", y: "{ly}",
                                        "text-anchor": "{anchor}",
                                        fill: "#cbd5e1", "font-size": "11",
                                        "{label}"
                                    }
                                }
                            }
                        }

                        // per-axis bars — inline-styled so they don't depend on Tailwind
                                                // emitting h-2 / bg-* (that collapse was hiding them)
                                                div { class: "mt-4",
                                                    for (label, value, pct) in rows.iter().cloned() {
                                                        div {
                                                            style: "display:flex; align-items:center; gap:0.75rem; margin-top:0.5rem;",
                                                            span { style: "width:8rem; font-size:0.875rem; color:#d1d5db;", "{label}" }
                                                            div {
                                                                style: "flex:1 1 0%; height:0.5rem; border-radius:9999px; \
                                                                        background:#374151; overflow:hidden;",
                                                                div { style: "height:100%; background:#6366f1; width:{pct}%;" }
                                                            }
                                                            span {
                                                                style: "width:4rem; text-align:right; font-size:0.875rem; color:#9ca3af;",
                                                                "{value}"
                                                            }
                                                        }
                                                    }
                                                }
                    }
                }

                p { class: "text-xs text-gray-500 mt-6",
                    "Each language is scored independently. Mastery is a conservative estimate (mean − k·σ): \
                     untested axes read \"untested,\" confidence contracts as time passes without a probe, and a \
                     passed probe snaps it back. Reading floors its prerequisites within the same language."
                }
            }
        }
    }
}
