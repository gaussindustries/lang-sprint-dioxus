// src/views/dashboard.rs
//
// Mastery dashboard: a six-axis radar of the conservative per-skill estimate,
// fed by the folded evidence log. The brain is untouched — this only reads
// model.mastery(). Untested axes sit at the center; the shape contracts as
// confidence decays and snaps back when a probe is passed. Reading evidence
// floors its prerequisites, so those axes can rise without being drilled.

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

#[component]
pub fn DashboardPage() -> Element {
    let learner = use_learner();
    let now = now_ms();
    let model = learner.model();
    let mastery = model.mastery(now);
    let events = learner.log_len();
    let items = model.item_count();
    let due = model.due(now, 0.9).len();

    let skills: Vec<Skill> = Skill::iter().collect();
    let n = skills.len();

    // ── geometry (all computed here; rsx only interpolates finished values) ──
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

    let readout: Vec<(String, String)> = skills
        .iter()
        .map(|s| {
            let txt = match mastery.get(s).copied().flatten() {
                Some(x) => format!("{}%", (x * 100.0).round() as i32),
                None => "untested".to_string(),
            };
            (s.label().to_string(), txt)
        })
        .collect();

    rsx! {
        div { class: "min-h-screen bg-gray-800 text-white p-8",
            div { class: "max-w-2xl mx-auto",
                h1 { class: "text-2xl font-semibold mb-1", "Progress" }
                p { class: "text-sm text-gray-400 mb-6",
                    "{events} observations · {items} items tracked · {due} due for review"
                }

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

                div { class: "grid grid-cols-2 gap-x-8 gap-y-2 mt-6 text-sm",
                    for (label, txt) in readout.iter().cloned() {
                        div { class: "flex justify-between",
                            span { "{label}" }
                            span { class: "text-gray-400", "{txt}" }
                        }
                    }
                }

                p { class: "text-xs text-gray-500 mt-8",
                    "A conservative estimate (mean − k·σ): untested axes sit at center, the shape contracts as time passes without a probe, and a passed probe snaps it back. Reading floors its prerequisites, so those axes can rise without being drilled directly."
                }
            }
        }
    }
}
