// src/components/dictionary.rs
//
// A self-contained dictionary view you can wire anywhere: `Dictionary { lang }`.
//
// Everything is data-driven. The A–Z order is taken from
// `assets/langs/<lang>/alphabet.json` (its array order *is* the collation
// order), and the entries come from the `Lexicon` (currently seeded by
// `1000.json`, but it grows automatically as the lexicon grows). Adding a new
// language is just dropping in its `alphabet.json` + `1000.json`; the
// dictionary then sorts and paginates itself.
//
// Aesthetic: a printed-paperback page floating in the dark app. The one
// deliberate flourish is the thumb index — the stair-stepped letter tabs down
// the right edge, in the language's own script — which doubles as jump
// navigation, plus guide words (first/last headword) in the running head.
//
// Requires `src/models/lexicon.rs` (the LexEntry / Lexicon module).

use dioxus::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::assets::{alphabet_json_for, freq_json_for};
use crate::components::WordDetail;
use crate::models::letter::Letter;
use crate::models::lexicon::{LexEntry, Lexicon};

/// First lowercased scalar of a string (the collation / leading-letter key).
fn first_key_char(s: &str) -> Option<char> {
    s.chars().flat_map(|c| c.to_lowercase()).next()
}

/// Map each char of `word` to its position in `order`; unknown chars sort last.
fn collate_key(word: &str, order: &HashMap<char, usize>, unknown: usize) -> Vec<usize> {
    word.chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| order.get(&c).copied().unwrap_or(unknown))
        .collect()
}

/// Dictionary-style POS abbreviation (falls back to the raw tag).
fn pos_abbr(p: &str) -> String {
    let a = match p.to_lowercase().as_str() {
        "noun" => "n.",
        "verb" => "v.",
        "adjective" => "adj.",
        "adverb" => "adv.",
        "pronoun" => "pron.",
        "numeral" => "num.",
        "conjunction" => "conj.",
        "adposition" | "preposition" | "postposition" => "prep.",
        "determiner" => "det.",
        "particle" => "part.",
        "interjection" => "interj.",
        other => return other.to_string(),
    };
    a.to_string()
}

/// A page row prepared for rendering (no method calls left for the template).
#[derive(Clone)]
struct Row {
    /// Some(glyph) when this row opens a new letter section.
    divider: Option<String>,
    key: String,
    head: String,
    pos: Option<String>,
    gloss: String,
    example: Option<String>,
    /// Full entry, opened in the word card on click.
    entry: LexEntry,
}

/// Hover highlight for clickable entries (Tailwind v4 JIT can miss arbitrary
/// colors, so this lives in a tiny injected stylesheet).
const ENTRY_CSS: &str = "\
.lex-entry{cursor:pointer;border-radius:3px;transition:background 0.1s ease;}\
.lex-entry:hover{background:rgba(110,43,43,0.09);}\
";

#[component]
pub fn Dictionary(lang: Signal<String>, #[props(default = 28)] per_page: usize) -> Element {
    let per_page = per_page.max(1);
    let mut page = use_signal(|| 0usize);
    let mut selected = use_signal(|| None::<LexEntry>);

    // Back to the first page whenever the language changes.
    use_effect(move || {
        let _ = lang();
        page.set(0);
    });

    // ── load lexicon + alphabet for the active language ───────────────────
    let lex_res = use_resource(move || {
        let l = lang.read().clone();
        async move {
            Lexicon::from_json(freq_json_for(&l)).unwrap_or_else(|e| {
                eprintln!("dictionary: failed to parse lexicon for {l}: {e}");
                Lexicon::default()
            })
        }
    });
    let alpha_res = use_resource(move || {
        let l = lang.read().clone();
        async move {
            serde_json::from_str::<Vec<Letter>>(alphabet_json_for(&l)).unwrap_or_else(|e| {
                eprintln!("dictionary: failed to parse alphabet for {l}: {e}");
                Vec::new()
            })
        }
    });

    let loading = lex_res.read().is_none() || alpha_res.read().is_none();
    let lex = lex_res.read().clone().unwrap_or_default();
    let alphabet = alpha_res.read().clone().unwrap_or_default();
    let lang_now = lang.read().clone();

    // ── paperback palette / type (inline to dodge Tailwind v4 JIT gaps) ───
    let paper = "#efe7d3";
    let paper_edge = "#e7ddc6";
    let ink = "#2a2622";
    let accent = "#6e2b2b";
    let secondary = "#8a7f6b";
    let rule = "#cbbfa3";
    let muted_tab = "#ded3b8";
    // L2 (Georgian/Cyrillic-capable) serif, and a Latin serif for glosses.
    let l2 = "'Noto Serif Georgian','Noto Serif',Georgia,'Times New Roman',serif";
    let body = "Georgia,'Times New Roman','Noto Serif',serif";

    // ── alphabet order: distinct leading glyphs in alphabet.json order ────
    let mut order: HashMap<char, usize> = HashMap::new();
    let mut alpha_glyphs: Vec<char> = Vec::new();
    for letter in &alphabet {
        if let Some(c) = first_key_char(&letter.letter) {
            if !order.contains_key(&c) {
                order.insert(c, alpha_glyphs.len());
                alpha_glyphs.push(c);
            }
        }
    }
    let unknown = alpha_glyphs.len() + 1;

    // ── sort the whole lexicon by that collation ─────────────────────────
    let mut entries = lex.all().to_vec();
    entries.sort_by_cached_key(|e| collate_key(e.head(), &order, unknown));

    // Loading / empty states (paper-quiet, not error-loud).
    if loading {
        return rsx! {
            section { class: "w-full flex justify-center py-8 px-4",
                div {
                    class: "max-w-sm shadow-xl",
                    style: "background:{paper}; color:{secondary}; font-family:{body}; \
                            padding:2.5rem; border:1px solid {rule}; border-radius:2px;",
                    "Opening the dictionary…"
                }
            }
        };
    }
    if entries.is_empty() {
        return rsx! {
            section { class: "w-full flex justify-center py-8 px-4",
                div {
                    class: "max-w-md shadow-xl",
                    style: "background:{paper}; color:{ink}; font-family:{body}; \
                            padding:2.5rem; border:1px solid {rule}; border-radius:2px;",
                    div { style: "font-family:{l2}; font-size:1.2rem; margin-bottom:0.4rem;", "No entries" }
                    div { style: "font-size:0.9rem; color:{secondary};",
                        "Add assets/langs/{lang_now}/1000.json and alphabet.json, and the dictionary fills itself in."
                    }
                }
            }
        };
    }

    // ── pagination ───────────────────────────────────────────────────────
    let total = entries.len();
    let pages = ((total + per_page - 1) / per_page).max(1);
    let cur = page().min(pages.saturating_sub(1));
    let start = cur * per_page;
    let end = (start + per_page).min(total);
    let slice = entries.get(start..end).unwrap_or(&[]);

    // Guide words = first & last headword on this page.
    let guide_first = slice
        .first()
        .map(|e| e.head().to_string())
        .unwrap_or_default();
    let guide_last = slice
        .last()
        .map(|e| e.head().to_string())
        .unwrap_or_default();
    let cur_disp = cur + 1;

    // First page index for each letter (thumb-index jump targets).
    let mut letter_first_page: HashMap<char, usize> = HashMap::new();
    for (i, e) in entries.iter().enumerate() {
        if let Some(c) = first_key_char(e.head()) {
            letter_first_page.entry(c).or_insert(i / per_page);
        }
    }
    // Which letters appear on the current page (highlight those tabs).
    let active_letters: HashSet<char> = slice
        .iter()
        .filter_map(|e| first_key_char(e.head()))
        .collect();

    // ── prepare display rows (with section dividers) ─────────────────────
    let mut display_rows: Vec<Row> = Vec::with_capacity(slice.len());
    let mut prev: Option<char> = None;
    for e in slice {
        let lead = first_key_char(e.head());
        let divider = match lead {
            Some(c) if Some(c) != prev => Some(c.to_string()),
            _ => None,
        };
        prev = lead;
        display_rows.push(Row {
            divider,
            key: format!("e{}", e.rank),
            head: e.head().to_string(),
            pos: e.pos.clone().map(|p| pos_abbr(&p)),
            gloss: e.en.clone(),
            example: e.example.clone(),
            entry: e.clone(),
        });
    }

    // ── prepare thumb tabs (colors precomputed, no logic in template) ────
    let tabs: Vec<(char, Option<usize>, &str, &str, &str, bool)> = alpha_glyphs
        .iter()
        .map(|&g| {
            let target = letter_first_page.get(&g).copied();
            let has = target.is_some();
            let active = active_letters.contains(&g);
            let bg = if active {
                accent
            } else if has {
                paper_edge
            } else {
                muted_tab
            };
            let fg = if active {
                paper
            } else if has {
                ink
            } else {
                secondary
            };
            let cursor = if has { "pointer" } else { "default" };
            (g, target, bg, fg, cursor, !has)
        })
        .collect();

    // ── pager appearance ─────────────────────────────────────────────────
    let prev_disabled = cur == 0;
    let next_disabled = cur + 1 >= pages;
    let prev_color = if prev_disabled { secondary } else { accent };
    let next_color = if next_disabled { secondary } else { accent };
    let prev_cursor = if prev_disabled { "default" } else { "pointer" };
    let next_cursor = if next_disabled { "default" } else { "pointer" };

    // Title-cased language label for the running head.
    let lang_title = {
        let mut ch = lang_now.chars();
        match ch.next() {
            Some(f) => f.to_uppercase().collect::<String>() + ch.as_str(),
            None => String::new(),
        }
    };

    rsx! {
        style { dangerous_inner_html: ENTRY_CSS }
        section { class: "w-full flex justify-center py-8 px-4",
            div { class: "w-full max-w-4xl flex items-stretch",

                // ── the page ─────────────────────────────────────────────
                div {
                    style: "flex:1 1 auto; background:{paper}; color:{ink}; \
                            font-family:{body}; display:flex; flex-direction:column; \
                            min-height:35rem; padding:2.4rem 2.6rem; \
                            border:1px solid {rule}; border-radius:3px 0 0 3px; \
                            box-shadow:0 12px 32px rgba(0,0,0,0.38), \
                            inset 0 0 70px rgba(120,90,40,0.06);",

                    // running head: language + DICTIONARY · guide words
                    div {
                        style: "display:flex; align-items:baseline; justify-content:space-between; \
                                padding-bottom:0.5rem; border-bottom:1.5px solid {accent};",
                        div { style: "display:flex; align-items:baseline; gap:0.6rem;",
                            span { style: "font-family:{l2}; font-size:1.05rem; color:{ink};", "{lang_title}" }
                            span {
                                style: "font-size:0.6rem; letter-spacing:0.3em; text-transform:uppercase; color:{secondary};",
                                "Dictionary"
                            }
                        }
                        span { style: "font-family:{l2}; font-size:0.95rem; color:{accent};",
                            "{guide_first}  —  {guide_last}"
                        }
                    }

                    // entries — responsive multicolumn (1 col narrow, 2+ wide)
                    div {
                        style: "flex:1 1 auto; margin-top:1.2rem; \
                                column-width:19rem; column-gap:2.75rem; \
                                column-rule:1px solid {rule};",

                        for row in display_rows.iter().cloned() {
                            if let Some(g) = row.divider.clone() {
                                div {
                                    key: "d-{row.key}",
                                    style: "column-span:all; -webkit-column-span:all; \
                                            break-after:avoid; display:flex; align-items:center; \
                                            gap:0.7rem; margin:1.05rem 0 0.45rem;",
                                    span { style: "font-family:{l2}; font-size:1.55rem; color:{accent}; line-height:1;", "{g}" }
                                    span { style: "flex:1; height:1px; background:{rule};" }
                                }
                            }
                            div {
                                key: "{row.key}",
                                class: "lex-entry",
                                style: "break-inside:avoid; -webkit-column-break-inside:avoid; \
                                        margin-bottom:0.35rem; text-indent:-0.9rem; \
                                        padding:0.12rem 0.4rem 0.12rem 0.9rem; line-height:1.3;",
                                onclick: move |_| { selected.set(Some(row.entry.clone())); },
                                span { style: "font-family:{l2}; font-weight:700; font-size:1.02rem; color:{ink};", "{row.head}" }
                                if let Some(p) = row.pos.clone() {
                                    span { style: "font-style:italic; font-size:0.76rem; color:{secondary}; margin-left:0.4rem;", "{p}" }
                                }
                                span { style: "font-size:0.9rem; color:{ink}; margin-left:0.4rem;", "{row.gloss}" }
                                if let Some(ex) = row.example.clone() {
                                    div { style: "font-family:{l2}; font-style:italic; font-size:0.82rem; color:{secondary}; margin-top:0.08rem;", "{ex}" }
                                }
                            }
                        }
                    }

                    // pager footer (pinned to page bottom)
                    div {
                        style: "display:flex; align-items:center; justify-content:space-between; \
                                margin-top:1.4rem; padding-top:0.7rem; border-top:1.5px solid {accent};",
                        button {
                            class: "focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-700",
                            style: "font-family:{body}; font-size:0.95rem; background:none; border:none; \
                                    color:{prev_color}; cursor:{prev_cursor};",
                            disabled: prev_disabled,
                            title: "Previous page",
                            onclick: move |_| {
                                let c = page();
                                if c > 0 { page.set(c - 1); }
                            },
                            "‹ Prev"
                        }
                        span { style: "font-family:{body}; font-size:0.82rem; letter-spacing:0.12em; color:{secondary};",
                            "— {cur_disp} / {pages} —"
                        }
                        button {
                            class: "focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-700",
                            style: "font-family:{body}; font-size:0.95rem; background:none; border:none; \
                                    color:{next_color}; cursor:{next_cursor};",
                            disabled: next_disabled,
                            title: "Next page",
                            onclick: move |_| {
                                let c = page();
                                if c + 1 < pages { page.set(c + 1); }
                            },
                            "Next ›"
                        }
                    }
                }

                // ── thumb index rail ─────────────────────────────────────
                div {
                    class: "flex flex-col justify-between",
                    style: "align-self:stretch;",
                    for (g, target, bg, fg, cursor, disabled) in tabs.iter().cloned() {
                        button {
                            key: "tab-{g}",
                            class: "focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-700",
                            style: "font-family:{l2}; font-size:0.74rem; line-height:1; \
                                    width:1.55rem; padding:0.18rem 0; border:none; \
                                    border-radius:0 4px 4px 0; background:{bg}; color:{fg}; cursor:{cursor};",
                            disabled: disabled,
                            title: "Jump to {g}",
                            onclick: move |_| {
                                if let Some(p) = target { page.set(p); }
                            },
                            "{g}"
                        }
                    }
                }
            }
        }

        if let Some(entry) = selected() {
            WordDetail { entry, on_close: move |_| selected.set(None) }
        }
    }
}
