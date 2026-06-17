// src/components/wpm_test.rs
//
// Self-contained WPM ("words per minute") typing test, Monkeytype-style.
//
// Row model (no measurement, no scrollIntoView, no page scroll):
//  - Words are packed into explicit rows of a fixed CHAR budget, so every row
//    is a known, fixed-height single-line block (ROW_H px). Because we control
//    the row height, we can translate the text block by whole rows with no
//    DOM measurement at all.
//  - A 3-row clip window shows the text. The CURRENT row is kept in the middle
//    by translating the block up by (current_row - 1) * ROW_H. As you advance,
//    the block slides up one row at a time (CSS transition), the top row scrolls
//    off, and the next row scrolls in.
//  - A linear-gradient mask fades the top and bottom edges so rows dissipate
//    in/out smoothly instead of popping.
//  - The transparent single-line <input> is the full-cover top layer.

use dioxus::events::{FormEvent, KeyboardEvent};
use dioxus::prelude::Key;
use dioxus::prelude::*;
use rand::Rng;
use std::collections::HashMap;
use std::time::Duration;

use crate::components::keyboard::code_to_qwerty_label;
use crate::models::{freq_word::FrequencyWord, letter::Letter};

/// Breathing caret animation. Injected once via a <style> element.
const WPM_CARET_CSS: &str = "\
@keyframes wpmBreathe { \
  0%, 100% { opacity: 1; box-shadow: 0 0 6px 0 rgba(129,140,248,0.75); } \
  50% { opacity: 0.25; box-shadow: 0 0 1px 0 rgba(129,140,248,0.10); } \
} \
.wpm-caret { animation: wpmBreathe 1.5s ease-in-out infinite; }";

#[derive(Clone, Copy, PartialEq, Eq)]
enum WpmState {
    Idle,
    Running,
    Finished,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WordState {
    Upcoming,
    Current,
    Completed,
}

#[derive(Clone, Copy, Default, PartialEq)]
struct WpmStats {
    net_wpm: u32,
    raw_wpm: u32,
    accuracy: f64,
    consistency: f64,
    correct: u32,
    incorrect: u32,
    typed: u32,
    secs: u32,
}

#[derive(Clone, Copy, PartialEq)]
struct Sample {
    t: u64,
    net: u32,
    raw: u32,
    err: u32,
    tc: u32, // cumulative typed chars (for per-interval consistency)
}

// ── pure helpers ──────────────────────────────────────────────────────────

fn clean_word(raw: &str) -> Option<String> {
    let first = raw.split('/').next().unwrap_or("").trim();
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

    while out.chars().count() < min_chars {
        let idx = rng.random_range(0..words.len());
        let Some(clean) = clean_word(words[idx].word.as_str()) else {
            continue;
        };

        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&clean);
    }

    out
}

fn min_chars_for(dur_secs: u64) -> usize {
    let est = 200usize * 5 * (dur_secs as usize) / 60;
    est.max(250)
}

fn compute_stats(target: &str, typed: &str, elapsed_secs: f64) -> WpmStats {
    let t: Vec<char> = target.chars().collect();
    let y: Vec<char> = typed.chars().collect();

    let mut correct = 0u32;
    let mut incorrect = 0u32;
    for (i, c) in y.iter().enumerate() {
        match t.get(i) {
            Some(tc) if tc == c => correct += 1,
            _ => incorrect += 1,
        }
    }

    let typed_n = y.len() as f64;
    let minutes = elapsed_secs.max(1.0) / 60.0;
    let raw = ((typed_n / 5.0) / minutes).round() as u32;
    let net = (((correct as f64) / 5.0) / minutes).round() as u32;
    let accuracy = if typed_n > 0.0 {
        (correct as f64 / typed_n) * 100.0
    } else {
        100.0
    };

    WpmStats {
        net_wpm: net,
        raw_wpm: raw,
        accuracy,
        consistency: 0.0, // filled in at finish from the sample series
        correct,
        incorrect,
        typed: y.len() as u32,
        secs: elapsed_secs.round().max(1.0) as u32,
    }
}

/// Consistency (0–100): how steady the per-second typing speed was. Computed as
/// 100 * (1 - coefficient_of_variation) over per-second raw speed (chars/sec).
fn consistency_pct(samples: &[Sample]) -> f64 {
    // per-second typed-char deltas
    let mut rates: Vec<f64> = Vec::new();
    for w in samples.windows(2) {
        let d = w[1].tc.saturating_sub(w[0].tc) as f64;
        let dt = (w[1].t.saturating_sub(w[0].t)).max(1) as f64;
        rates.push(d / dt);
    }
    if rates.len() < 2 {
        return 100.0;
    }
    let mean = rates.iter().sum::<f64>() / rates.len() as f64;
    if mean <= 0.0 {
        return 0.0;
    }
    let var = rates.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / rates.len() as f64;
    let cv = var.sqrt() / mean;
    ((1.0 - cv) * 100.0).clamp(0.0, 100.0)
}

fn char_class(idx: usize, cursor: usize, ychars: &[char], ch: char) -> &'static str {
    if idx < cursor {
        if ychars[idx] == ch {
            "text-emerald-300"
        } else {
            "text-red-400 bg-red-500/15 rounded-sm"
        }
    } else {
        "text-gray-500"
    }
}

fn caret_node(cursor: usize) -> Element {
    rsx! {
        span {
            key: "caret-{cursor}",
            class: "wpm-caret",
            style: "display:inline-block; width:2px; height:1.1em; margin:0 -1px; \
                    background-color:#818cf8; border-radius:1px; vertical-align:-0.18em;",
        }
    }
}

/// Pack word indices into rows of ~`budget` characters.
fn pack_rows(words: &[&str], budget: usize) -> Vec<Vec<usize>> {
    let mut rows: Vec<Vec<usize>> = Vec::new();
    let mut cur: Vec<usize> = Vec::new();
    let mut acc = 0usize;

    for (wi, w) in words.iter().enumerate() {
        let wl = w.chars().count();
        if !cur.is_empty() && acc + 1 + wl > budget {
            rows.push(std::mem::take(&mut cur));
            acc = 0;
        }
        if !cur.is_empty() {
            acc += 1; // the space
        }
        acc += wl;
        cur.push(wi);
    }
    if !cur.is_empty() {
        rows.push(cur);
    }
    rows
}

// ── results graph (inline SVG) ──────────────────────────────────────────────

fn results_graph(samples: &[Sample]) -> Element {
    if samples.len() < 2 {
        return rsx! {
            div { class: "text-xs text-gray-500 italic text-center py-4",
                "Not enough data to graph."
            }
        };
    }

    const W: f64 = 480.0;
    const H: f64 = 200.0;
    const PAD_L: f64 = 32.0;
    const PAD_R: f64 = 12.0;
    const PAD_T: f64 = 12.0;
    const PAD_B: f64 = 24.0;
    let plot_w = W - PAD_L - PAD_R;
    let plot_h = H - PAD_T - PAD_B;

    let span = samples.last().map(|s| s.t).unwrap_or(1).max(1) as f64;
    let max_v = samples.iter().map(|s| s.raw.max(s.net)).max().unwrap_or(10) as f64;
    let max_y = ((max_v * 1.15 / 10.0).ceil() * 10.0).max(20.0);

    let px = |t: u64| PAD_L + (t as f64 / span) * plot_w;
    let py = |v: u32| PAD_T + plot_h - (v as f64 / max_y) * plot_h;

    let net_pts = samples
        .iter()
        .map(|s| format!("{:.1},{:.1}", px(s.t), py(s.net)))
        .collect::<Vec<_>>()
        .join(" ");
    let raw_pts = samples
        .iter()
        .map(|s| format!("{:.1},{:.1}", px(s.t), py(s.raw)))
        .collect::<Vec<_>>()
        .join(" ");

    let mut err_dots: Vec<(f64, f64)> = Vec::new();
    let mut prev = 0u32;
    for s in samples {
        if s.err > prev {
            err_dots.push((px(s.t), py(s.net)));
        }
        prev = s.err;
    }

    let y0 = py(0);
    let y_mid = py((max_y / 2.0) as u32);
    let y_top = py(max_y as u32);
    let mid_label = (max_y / 2.0) as u32;
    let top_label = max_y as u32;
    let span_label = span as u32;
    let baseline_x2 = W - PAD_R;
    let bottom_y = H - 8.0;

    // filled area under the net-wpm line (subtle indigo wash)
    let area_pts = format!(
        "{net_pts} {:.1},{:.1} {:.1},{:.1}",
        baseline_x2, y0, PAD_L, y0
    );
    rsx! {
        svg {
            view_box: "0 0 480 200",
            preserve_aspect_ratio: "xMidYMid meet",
            style: "display:block; width:100%; height:15rem;",
            line { x1: "{PAD_L}", y1: "{y_top}", x2: "{baseline_x2}", y2: "{y_top}",
                stroke: "rgba(255,255,255,0.07)", "stroke-width": "1" }
            line { x1: "{PAD_L}", y1: "{y_mid}", x2: "{baseline_x2}", y2: "{y_mid}",
                stroke: "rgba(255,255,255,0.07)", "stroke-width": "1" }
            line { x1: "{PAD_L}", y1: "{y0}", x2: "{baseline_x2}", y2: "{y0}",
                stroke: "rgba(255,255,255,0.18)", "stroke-width": "1" }

            text { x: "{PAD_L - 6.0}", y: "{y_top + 3.0}", fill: "rgba(255,255,255,0.4)",
                "font-size": "9", "text-anchor": "end", "{top_label}" }
            text { x: "{PAD_L - 6.0}", y: "{y_mid + 3.0}", fill: "rgba(255,255,255,0.4)",
                "font-size": "9", "text-anchor": "end", "{mid_label}" }
            text { x: "{PAD_L - 6.0}", y: "{y0 + 3.0}", fill: "rgba(255,255,255,0.4)",
                "font-size": "9", "text-anchor": "end", "0" }

            // subtle per-second tick marks along the baseline
            { (0..=span_label).map(|t| {
                let tx = px(t as u64);
                let tall = t % 5 == 0; // every 5s a touch taller
                let y2 = if tall { y0 + 5.0 } else { y0 + 3.0 };
                rsx! {
                    line {
                        key: "tick{t}",
                        x1: "{tx}", y1: "{y0}", x2: "{tx}", y2: "{y2}",
                        stroke: "rgba(255,255,255,0.22)", "stroke-width": "1"
                    }
                }
            }) }

            polyline { points: "{area_pts}", fill: "rgba(129,140,248,0.16)", stroke: "none" }

            polyline { points: "{raw_pts}", fill: "none", stroke: "rgba(148,163,184,0.55)",
                "stroke-width": "1.5", "stroke-linejoin": "round" }
            polyline { points: "{net_pts}", fill: "none", stroke: "rgb(129,140,248)",
                "stroke-width": "2.5", "stroke-linejoin": "round", "stroke-linecap": "round" }

            { err_dots.iter().enumerate().map(|(i, (cx, cy))| rsx! {
                circle { key: "err{i}", cx: "{cx}", cy: "{cy}", r: "2.6", fill: "rgb(248,113,113)" }
            }) }

            text { x: "{PAD_L}", y: "{bottom_y}", fill: "rgba(255,255,255,0.4)",
                "font-size": "9", "text-anchor": "start", "0s" }
            text { x: "{baseline_x2}", y: "{bottom_y}", fill: "rgba(255,255,255,0.4)",
                "font-size": "9", "text-anchor": "end", "{span_label}s" }
        }
    }
}

// ── component ───────────────────────────────────────────────────────────────

#[component]
pub fn WpmTest(
    words: Vec<FrequencyWord>,
    lang: Signal<String>,
    letters_vec: Vec<Letter>,
) -> Element {
    let mut target = use_signal(String::new);
    let mut typed = use_signal(String::new);
    let mut state = use_signal(|| WpmState::Idle);
    let mut duration = use_signal(|| 60u64);
    let mut remaining = use_signal(|| 60u64);
    let mut result = use_signal(|| None::<WpmStats>);
    let mut show_help = use_signal(|| false);
    let mut samples = use_signal(Vec::<Sample>::new);

    let mut run_id = use_signal(|| 0u32);

    // Restart the SAME passage (Tab / Esc / Restart button).
    let restart = use_callback(move |_: ()| {
        run_id.set(run_id() + 1); // cancel any running timer
        typed.set(String::new());
        state.set(WpmState::Idle);
        remaining.set(duration());
        result.set(None);
        samples.set(Vec::new());
    });
    // Brand-new random passage (New text button). Resets state DIRECTLY (like
    // restart) so the results panel clears immediately, then swaps in a fresh
    // passage.
    let new_text = {
        let words = words.clone();
        use_callback(move |_: ()| {
            run_id.set(run_id() + 1);
            target.set(build_wpm_text(&words, min_chars_for(duration())));
            typed.set(String::new());
            state.set(WpmState::Idle);
            remaining.set(duration());
            result.set(None);
            samples.set(Vec::new());
        })
    };

    let mut hint_map: HashMap<char, String> = HashMap::new();
    for letter in &letters_vec {
        if let Some(ch) = letter.letter.chars().next() {
            let base = code_to_qwerty_label(&letter.key_code);
            if !base.is_empty() {
                let rendered = if letter.shifted {
                    format!("⇧{base}")
                } else {
                    base.to_string()
                };
                hint_map.insert(ch, rendered);
            }
        }
    }

    // ── (re)generate text + reset ─────────────────────────────────────────
    {
        let words = words.clone();
        use_effect(move || {
            let _ = lang();
            let dur = duration();

            target.set(build_wpm_text(&words, min_chars_for(dur)));
            typed.set(String::new());
            state.set(WpmState::Idle);
            remaining.set(dur);
            result.set(None);
            samples.set(Vec::new());
        });
    }

    // ── countdown timer + per-second sampling ──────────────────────────────
    use_resource(move || {
        let token = run_id();
        async move {
            if token == 0 {
                return;
            }
            if !matches!(*state.peek(), WpmState::Running) {
                return;
            }

            let total = *duration.peek();
            remaining.set(total);
            samples.set(vec![Sample {
                t: 0,
                net: 0,
                raw: 0,
                err: 0,
                tc: 0,
            }]);

            let mut rem = total;
            while rem > 0 {
                tokio::time::sleep(Duration::from_secs(1)).await;

                if *run_id.peek() != token || !matches!(*state.peek(), WpmState::Running) {
                    return;
                }

                rem -= 1;
                remaining.set(rem);

                let elapsed = total - rem;
                let s = compute_stats(&target.peek(), &typed.peek(), elapsed as f64);
                samples.write().push(Sample {
                    t: elapsed,
                    net: s.net_wpm,
                    raw: s.raw_wpm,
                    err: s.incorrect,
                    tc: s.typed,
                });
            }

            let mut res = compute_stats(&target.peek(), &typed.peek(), total as f64);
            res.consistency = consistency_pct(&samples.peek());
            result.set(Some(res));
            state.set(WpmState::Finished);
        }
    });

    // ── derive render state ─────────────────────────────────────────────────
    let st = state();
    let dur = duration();
    let rem = remaining();
    let help = show_help();
    let target_str = target();
    let typed_str = typed();
    let sample_data = samples();

    let tchars: Vec<char> = target_str.chars().collect();
    let ychars: Vec<char> = typed_str.chars().collect();
    let cursor = ychars.len();
    let total_chars = tchars.len();

    let elapsed = dur.saturating_sub(rem);
    let live = compute_stats(&target_str, &typed_str, elapsed as f64);
    let shown = if matches!(st, WpmState::Finished) {
        result().unwrap_or(live)
    } else {
        live
    };

    let split_words: Vec<&str> = if target_str.is_empty() {
        Vec::new()
    } else {
        target_str.split(' ').collect()
    };

    // global char start of each word (word + trailing space)
    let mut word_char_start: Vec<usize> = Vec::with_capacity(split_words.len());
    {
        let mut g = 0usize;
        for w in &split_words {
            word_char_start.push(g);
            g += w.chars().count() + 1;
        }
    }
    let cur_word = word_char_start
        .iter()
        .rposition(|&s| s <= cursor)
        .unwrap_or(0);

    // pack into rows and locate the active row. Hinted glyphs are ~3x wider
    // (fixed-pitch cells), so far fewer characters fit per row. Budgets are set
    // a bit under what fits, so full rows spread to fill the width cleanly.
    let row_budget = if help { 32 } else { 50 };
    let rows = pack_rows(&split_words, row_budget);
    let current_row = rows.iter().position(|r| r.contains(&cur_word)).unwrap_or(0);

    // row geometry (fixed; no measurement)
    let row_h: usize = if help { 68 } else { 52 };
    let clip_h = row_h * 3;
    let gap = if help { "0.4rem" } else { "0.85rem" };
    let win_start = current_row.saturating_sub(1); // active row sits in the middle
    let translate = win_start * row_h;

    let cell_style =
        "position:relative; display:inline-block; min-width:1.15em; text-align:center; \
         line-height:1; padding-bottom:0.95em;";
    let hint_style = "position:absolute; left:50%; bottom:0; transform:translateX(-50%); \
         line-height:1; white-space:nowrap; font-size:0.62rem;";

    // build rows
    let mut row_els: Vec<Element> = Vec::new();
    for (ri, row) in rows.iter().enumerate() {
        let mut word_els: Vec<Element> = Vec::new();

        for &wi in row {
            let w = split_words[wi];
            let wlen = w.chars().count();
            let start = word_char_start[wi];
            let end = start + wlen; // trailing-space index

            let word_state = if cursor > end {
                WordState::Completed
            } else if cursor >= start {
                WordState::Current
            } else {
                WordState::Upcoming
            };

            let pill_bg = match word_state {
                WordState::Upcoming => "bg-slate-700/40",
                WordState::Current => "bg-slate-800/70 ring-1 ring-indigo-400/70",
                WordState::Completed => "bg-slate-900/80",
            };

            let mut char_nodes: Vec<Element> = Vec::new();
            for k in 0..wlen {
                let idx = start + k;
                if idx == cursor {
                    char_nodes.push(caret_node(cursor));
                }
                let ch = tchars[idx];
                let class = char_class(idx, cursor, &ychars, ch);

                if help {
                    let hint = hint_map.get(&ch).cloned().unwrap_or_default();
                    let hint_class = if idx == cursor {
                        "text-sky-400 font-bold"
                    } else {
                        "text-blue-800"
                    };
                    char_nodes.push(rsx! {
                        span { key: "c{idx}", style: "{cell_style}",
                            span { class: "{class}", "{ch}" }
                            span { class: "{hint_class}", style: "{hint_style}", "{hint}" }
                        }
                    });
                } else {
                    char_nodes.push(rsx! { span { key: "c{idx}", class: "{class}", "{ch}" } });
                }
            }
            // caret sitting on the space between words (rendered in the gap)
            if cursor == end && wi + 1 < split_words.len() {
                char_nodes.push(caret_node(cursor));
            }

            word_els.push(rsx! {
                span {
                    key: "w{wi}",
                    class: "shrink-0 whitespace-nowrap rounded px-1.5 py-0.5 transition-colors duration-150 {pill_bg}",
                    { char_nodes.into_iter() }
                }
            });
        }

        // full rows spread edge-to-edge to fill the width (even gaps that
        // adapt to the container); the final partial row stays left-aligned.
        let justify = if ri + 1 == rows.len() {
            "flex-start"
        } else {
            "space-between"
        };
        row_els.push(rsx! {
            div {
                key: "row{ri}",
                class: "flex flex-nowrap items-center w-full",
                style: "height:{row_h}px; gap:{gap}; justify-content:{justify};",
                { word_els.into_iter() }
            }
        });
    }

    let status_label = match st {
        WpmState::Idle => "ready",
        WpmState::Running => "typing",
        WpmState::Finished => "done",
    };

    let active_btn = "px-2.5 py-1 rounded bg-indigo-600 text-white cursor-pointer";
    let idle_btn = "px-2.5 py-1 rounded bg-gray-700 text-gray-300 hover:bg-gray-600 cursor-pointer";

    let finished = matches!(st, WpmState::Finished);
    let panel_h = "35rem"; // results-card height (also the expanded backdrop height)

    // The text layer doubles as the frosted backdrop once finished: it grows to
    // fill the card and is blurred. We ALWAYS write `filter` (= `none` when not
    // finished) plus a single `height`/`transform` value — never appending and
    // never *removing* a property. Property removal does not repaint reliably in
    // WebKitGTK (the blur lingered over the next live test after "New text");
    // toggling the value instead is a plain change, which does apply.
    let clip_h_css = if finished {
        panel_h.to_string()
    } else {
        format!("{clip_h}px")
    };
    let blur_css = if finished { "blur(5px)" } else { "none" };
    let clip_dyn_style = format!(
        "height:{clip_h_css}; overflow:hidden; \
         -webkit-mask-image:linear-gradient(to bottom, transparent 0%, #000 22%, #000 78%, transparent 100%); \
         mask-image:linear-gradient(to bottom, transparent 0%, #000 22%, #000 78%, transparent 100%); \
         filter:{blur_css};"
    );

    // When finished, pin the expanded backdrop to the top of the card (show the
    // passage from the start) instead of the last typing position.
    let inner_translate = if finished { 0 } else { translate };
    let inner_dyn_style = format!(
        "transform: translateY(-{inner_translate}px); transition: transform 0.25s ease; will-change: transform;"
    );

    let surface_class = "relative w-full";
    let surface_style = if finished {
        format!("min-height: {panel_h};")
    } else {
        "min-height: 0;".to_string()
    };
    let text_class = if help {
        "text-xl tracking-wide select-none px-1"
    } else {
        "text-2xl tracking-wide select-none px-1"
    };

    rsx! {
        div { class: "flex flex-col gap-4 p-4 w-full max-w-3xl mx-auto",

            style { dangerous_inner_html: WPM_CARET_CSS }

            // ── selectors: duration + hints ──────────────────────────────────
            if !finished {
                div { class: "flex flex-col items-center gap-2",
                    div { class: "flex items-center justify-center gap-2 text-xs",
                        span { class: "text-gray-400 mr-1", "Duration" }
                        for d in [15u64, 30, 60, 120] {
                            button {
                                key: "{d}",
                                class: if d == dur { active_btn } else { idle_btn },
                                onclick: move |_| { duration.set(d); },
                                "{d}s"
                            }
                        }
                    }

                    div { class: "flex items-center justify-center gap-2 text-xs",
                        span { class: "text-gray-400 mr-1", "Hints" }
                        button {
                            key: "help-on",
                            class: if help { active_btn } else { idle_btn },
                            onclick: move |_| { show_help.set(true); },
                            "On"
                        }
                        button {
                            key: "help-off",
                            class: if !help { active_btn } else { idle_btn },
                            onclick: move |_| { show_help.set(false); },
                            "Off"
                        }
                    }
                }
            }

            // ── live stat bar ────────────────────────────────────────────────
            if !finished {
                div { class: "flex items-stretch justify-center gap-8 text-center",
                    div {
                        div { class: "text-3xl font-bold tabular-nums text-white",
                            if matches!(st, WpmState::Idle) { "{dur}" } else { "{rem}" }
                        }
                        div { class: "text-[0.65rem] uppercase tracking-wide text-gray-400", "seconds" }
                    }
                    div {
                        div { class: "text-3xl font-bold tabular-nums text-indigo-300", "{shown.net_wpm}" }
                        div { class: "text-[0.65rem] uppercase tracking-wide text-gray-400", "wpm" }
                    }
                    div {
                        div { class: "text-3xl font-bold tabular-nums text-emerald-300",
                            "{shown.accuracy:.0}%"
                        }
                        div { class: "text-[0.65rem] uppercase tracking-wide text-gray-400", "accuracy" }
                    }
                }
            }

            // ── typing surface: fixed 3-row clip window ──────────────────────
            div { class: "{surface_class}", style: "{surface_style}",
                div {
                    class: "{text_class}",
                    style: "{clip_dyn_style}",
                    div { style: "{inner_dyn_style}",
                        { row_els.into_iter() }
                    }
                }

                input {
                    r#type: "text",
                    value: "{typed_str}",
                    class: "absolute inset-0 w-full h-full opacity-0 cursor-text",
                    style: "caret-color: transparent; color: transparent; border: none; \
                            outline: none; box-shadow: none;",
                    autocomplete: "off",
                    autocorrect: "off",
                    spellcheck: "false",
                    autofocus: "true",
                    onkeydown: move |evt: KeyboardEvent| {
                        // Tab or Esc restarts the test (Tab would otherwise move focus)
                        match evt.key() {
                            Key::Tab | Key::Escape => {
                                evt.prevent_default();
                                restart.call(());
                            }
                            _ => {}
                        }
                    },
                    oninput: move |evt: FormEvent| {
                        if matches!(*state.peek(), WpmState::Finished) {
                            return;
                        }

                        let tlen = target.peek().chars().count();
                        let mut v = evt.value();
                        if v.chars().count() > tlen {
                            v = v.chars().take(tlen).collect();
                        }

                        if matches!(*state.peek(), WpmState::Idle) {
                            if v.is_empty() {
                                typed.set(v);
                                return;
                            }
                            state.set(WpmState::Running);
                            let next = run_id() + 1;
                            run_id.set(next);
                        }

                        typed.set(v.clone());

                        if tlen > 0 && v.chars().count() == tlen {
                            let total = *duration.peek();
                            let rem_now = *remaining.peek();
                            let el = total.saturating_sub(rem_now).max(1);
                            let mut s = compute_stats(&target.peek(), &v, el as f64);
                            samples.write().push(Sample {
                                t: el,
                                net: s.net_wpm,
                                raw: s.raw_wpm,
                                err: s.incorrect,
                                tc: s.typed,
                            });
                            s.consistency = consistency_pct(&samples.peek());
                            result.set(Some(s));
                            state.set(WpmState::Finished);
                        }
                    },
                }

                if matches!(st, WpmState::Idle) {
                    div {
                        class: "absolute inset-0 flex items-center justify-center pointer-events-none",
                        span { class: "text-sm text-gray-500 bg-gray-800/60 px-3 py-1 rounded",
                            "Start typing to begin…"
                        }
                    }
                }

                // ── results panel (translucent frosted overlay) ─────────────
                if let (WpmState::Finished, Some(res)) = (st, result()) {
                    div {
                        class: "absolute inset-0 z-20 overflow-y-auto rounded-lg",
                        style: "background: rgba(17,24,39,0.72);",
                        div {
                            class: "min-h-full flex flex-col items-center justify-center gap-3 p-6",

                            // headline
                            div { class: "flex items-end gap-8",
                                div { class: "text-center",
                                    div { class: "text-5xl font-bold tabular-nums text-indigo-300", "{res.net_wpm}" }
                                    div { class: "text-[0.6rem] uppercase tracking-widest text-gray-400", "wpm" }
                                }
                                div { class: "text-center",
                                    div { class: "text-5xl font-bold tabular-nums text-emerald-300", "{res.accuracy:.0}" }
                                    div { class: "text-[0.6rem] uppercase tracking-widest text-gray-400", "% acc" }
                                }
                            }

                            // metric grid
                            div { class: "grid grid-cols-3 gap-x-8 gap-y-2 text-center",
                                div {
                                    div { class: "text-lg font-semibold tabular-nums text-slate-200", "{res.raw_wpm}" }
                                    div { class: "text-[0.6rem] uppercase tracking-wide text-gray-400", "raw" }
                                }
                                div {
                                    div { class: "text-lg font-semibold tabular-nums text-slate-200", "{res.consistency:.0}%" }
                                    div { class: "text-[0.6rem] uppercase tracking-wide text-gray-400", "consistency" }
                                }
                                div {
                                    div { class: "text-lg font-semibold tabular-nums text-slate-200", "{res.secs}s" }
                                    div { class: "text-[0.6rem] uppercase tracking-wide text-gray-400", "time" }
                                }
                                div { class: "col-span-3",
                                    div { class: "text-lg font-semibold tabular-nums",
                                        span { class: "text-emerald-300", "{res.correct}" }
                                        span { class: "text-gray-500", " / " }
                                        span { class: "text-red-400", "{res.incorrect}" }
                                        span { class: "text-gray-500", " / " }
                                        span { class: "text-slate-300", "{res.typed}" }
                                    }
                                    div { class: "text-[0.6rem] uppercase tracking-wide text-gray-400",
                                        "characters · correct / incorrect / total"
                                    }
                                }
                            }

                            // graph
                            div { class: "w-full max-w-xl",
                                { results_graph(&sample_data) }
                                div { class: "flex justify-center gap-4 text-[0.65rem] mt-1",
                                    span { class: "text-indigo-300", "— wpm" }
                                    span { class: "text-slate-400", "— raw" }
                                    span { class: "text-red-400", "• errors" }
                                }
                            }

                            // buttons
                            div { class: "flex items-center gap-3",
                                button {
                                    class: "px-4 py-1.5 rounded bg-indigo-600 hover:bg-indigo-500 text-white text-sm cursor-pointer",
                                    onclick: move |_| restart.call(()),
                                    "Restart"
                                }
                                button {
                                    class: "px-4 py-1.5 rounded bg-gray-700 hover:bg-gray-600 text-gray-200 text-sm cursor-pointer",
                                    onclick: move |_| new_text.call(()),
                                    "New text"
                                }
                            }
                            div { class: "text-[0.6rem] text-gray-500", "Tab or Esc to restart" }
                        }
                    }
                }
            }

            // ── controls (hidden once finished — the panel carries them) ─────
            if !matches!(st, WpmState::Finished) {
                div { class: "flex items-center justify-center gap-4 text-xs text-gray-400",
                    span { "status: {status_label}" }
                    button {
                        class: "px-3 py-1 rounded bg-gray-700 hover:bg-gray-600 text-gray-200 cursor-pointer",
                        onclick: move |_| restart.call(()),
                        "Restart"
                    }
                    button {
                        class: "px-3 py-1 rounded bg-gray-700 hover:bg-gray-600 text-gray-200 cursor-pointer",
                        onclick: move |_| new_text.call(()),
                        "New text"
                    }
                    span { class: "text-gray-600", "Tab / Esc to restart" }
                }
            }
        }
    }
}
