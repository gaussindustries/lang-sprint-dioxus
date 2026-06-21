// src/views/dashboard.rs
//
// Reads the folded mastery vector and draws it. A stub for now (bars, not the
// radar) — the point is to prove the loop end to end: drill → evidence → log →
// model → here. The radar replaces the bars later without touching the brain.

use dioxus::prelude::*;
use strum::IntoEnumIterator;

use crate::learner::{now_ms, use_learner};
use crate::learning::Skill;

#[component]
pub fn DashboardPage() -> Element {
    let learner = use_learner();
    let now = now_ms();
    let model = learner.model();
    let mastery = model.mastery(now);
    let events = learner.log_len();
    let items = model.item_count();
    let due = model.due(now, 0.9).len();

    // precompute rows (no method calls left in the template)
    let rows: Vec<(String, Option<f32>)> = Skill::iter()
        .map(|s| (s.label().to_string(), mastery.get(&s).copied().flatten()))
        .collect();

    rsx! {
        div { class: "min-h-screen bg-gray-800 text-white p-8",
            div { class: "max-w-2xl mx-auto",
                h1 { class: "text-2xl font-semibold mb-1", "Progress" }
                p { class: "text-sm text-gray-400 mb-6",
                    "{events} observations · {items} items tracked · {due} due for review"
                }
                div { class: "space-y-3",
                    for (label, val) in rows.iter().cloned() {
                        DashRow { label, val }
                    }
                }
                p { class: "text-xs text-gray-500 mt-8",
                    "Mastery is a conservative estimate (mean − k·σ). Axes you haven't touched read \"untested\"; confidence contracts as time passes without a probe, and a passed probe snaps it back."
                }
            }
        }
    }
}

#[component]
fn DashRow(label: String, val: Option<f32>) -> Element {
    let (text, bar) = match val {
        Some(v) => {
            let p = (v * 100.0).round().clamp(0.0, 100.0) as i32;
            (format!("{p}%"), p.max(2))
        }
        None => ("untested".to_string(), 0),
    };
    rsx! {
        div {
            div { class: "flex justify-between text-sm mb-1",
                span { "{label}" }
                span { class: "text-gray-400", "{text}" }
            }
            div { class: "h-2 rounded bg-gray-700 overflow-hidden",
                div { class: "h-full bg-indigo-500", style: "width: {bar}%;" }
            }
        }
    }
}
