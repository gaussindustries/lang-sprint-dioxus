// src/components/word_detail.rs
//
// The focused word card (modal): head word, POS, gloss, example, and — when the
// lexicon entry carries one — the full Georgian declension table, with a hover
// tooltip on each case name explaining what that case is for.
//
// Self-contained: render `WordDetail { entry, on_close }` and it draws its own
// backdrop. Reused by the navbar search and the dictionary browse view.

use dioxus::prelude::*;

use crate::models::lexicon::{GeoCase, LexEntry};

// Hover tooltip for the case names. Lives in an injected <style> because it
// needs a descendant :hover rule (not expressible inline / via Tailwind JIT).
const CASE_TIP_CSS: &str = "\
.lex-case{position:relative;cursor:help;border-bottom:1px dotted #9b8f76;}\
.lex-case .lex-tip{position:absolute;left:0;top:130%;z-index:80;width:15rem;\
background:#2a2622;color:#efe7d3;font-size:0.72rem;line-height:1.4;font-style:normal;\
font-family:Georgia,'Times New Roman',serif;padding:0.5rem 0.65rem;border-radius:4px;\
opacity:0;visibility:hidden;transition:opacity 0.12s ease;pointer-events:none;\
box-shadow:0 8px 22px rgba(0,0,0,0.4);}\
.lex-case:hover .lex-tip{opacity:1;visibility:visible;}\
";

#[component]
pub fn WordDetail(entry: LexEntry, on_close: EventHandler<()>) -> Element {
    // paperback palette (matches the dictionary)
    let paper = "#efe7d3";
    let paper_edge = "#e7ddc6";
    let ink = "#2a2622";
    let accent = "#6e2b2b";
    let secondary = "#8a7f6b";
    let rule = "#cbbfa3";
    let l2 = "'Noto Serif Georgian','Noto Serif',Georgia,'Times New Roman',serif";
    let body = "Georgia,'Times New Roman','Noto Serif',serif";

    // header / meaning values (precomputed; no method calls in the template)
    let head = entry.head().to_string();
    let rank = entry.rank;
    let pos = entry.pos.clone();
    let gloss = entry.en.clone();
    let example = entry.example.clone();

    // declension rows: (label, blurb, singular, plural, with_noun); empty → "—"
    let dash = |s: &str| {
        if s.trim().is_empty() {
            "—".to_string()
        } else {
            s.to_string()
        }
    };
    let decl_rows: Vec<(&'static str, &'static str, String, String, String)> = entry
        .declension
        .as_ref()
        .filter(|d| d.any_filled())
        .map(|d| {
            GeoCase::ALL
                .iter()
                .map(|&c| {
                    let f = d.forms(c);
                    (
                        c.label(),
                        c.blurb(),
                        dash(&f.singular),
                        dash(&f.plural),
                        dash(&f.with_noun),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let has_table = !decl_rows.is_empty();

    // shared cell styles
    let header_cell = format!(
        "text-align:left; font-size:0.64rem; letter-spacing:0.08em; text-transform:uppercase; \
         color:{secondary}; background:{paper_edge}; padding:0.4rem 0.6rem; border:1px solid {rule};"
    );
    let case_cell = format!(
        "padding:0.4rem 0.6rem; border:1px solid {rule}; font-family:{body}; \
         font-size:0.82rem; color:{ink};"
    );
    let form_cell = format!(
        "padding:0.4rem 0.6rem; border:1px solid {rule}; font-family:{l2}; \
         font-size:0.96rem; color:{ink}; word-break:break-word;"
    );

    rsx! {
        style { dangerous_inner_html: CASE_TIP_CSS }

        // backdrop (click to dismiss); scrolls if the card is taller than the viewport
        div {
            class: "fixed inset-0 flex items-start justify-center p-4 overflow-y-auto",
            style: "background: rgba(0,0,0,0.5); z-index:60;",
            onclick: move |_| on_close.call(()),

            // card (stop propagation so clicking inside doesn't dismiss)
            div {
                class: "w-full max-w-2xl my-8 shadow-2xl",
                style: "background:{paper}; color:{ink}; font-family:{body}; \
                        padding:1.9rem 2.1rem; border:1px solid {rule}; border-radius:4px; \
                        overflow:visible;",
                onclick: move |e| e.stop_propagation(),

                // headword + rank
                div {
                    style: "display:flex; align-items:baseline; justify-content:space-between; \
                            border-bottom:1.5px solid {accent}; padding-bottom:0.55rem;",
                    span { style: "font-family:{l2}; font-size:1.9rem; font-weight:700; color:{ink}; line-height:1;", "{head}" }
                    span { style: "font-size:0.68rem; color:{secondary};", "#{rank}" }
                }

                // POS badge
                if let Some(p) = pos {
                    div { style: "margin-top:0.7rem;",
                        span {
                            style: "display:inline-block; background:{paper_edge}; color:{accent}; \
                                    font-size:0.7rem; letter-spacing:0.04em; padding:0.16rem 0.65rem; \
                                    border-radius:999px;",
                            "{p}"
                        }
                    }
                }

                // gloss
                div { style: "margin-top:0.55rem; font-size:1.08rem; color:{ink};", "{gloss}" }

                // example
                if let Some(ex) = example {
                    div { style: "font-family:{l2}; font-style:italic; font-size:0.95rem; color:{secondary}; margin-top:0.75rem;", "{ex}" }
                }

                // declension table
                if has_table {
                    div { style: "margin-top:1.4rem;",
                        div {
                            style: "font-size:0.72rem; letter-spacing:0.12em; text-transform:uppercase; \
                                    color:{accent}; margin-bottom:0.5rem;",
                            "Declension"
                        }
                        table { style: "width:100%; border-collapse:collapse; table-layout:fixed;",
                            thead {
                                tr {
                                    th { style: "{header_cell} width:8.5rem;", "Case" }
                                    th { style: "{header_cell}", "Singular" }
                                    th { style: "{header_cell}", "Plural" }
                                    th { style: "{header_cell}", "With noun" }
                                }
                            }
                            tbody {
                                for (label, blurb, sg, pl, wn) in decl_rows.iter().cloned() {
                                    tr {
                                        key: "{label}",
                                        td { style: "{case_cell}",
                                            span { class: "lex-case",
                                                "{label}"
                                                span { class: "lex-tip", "{blurb}" }
                                            }
                                        }
                                        td { style: "{form_cell}", "{sg}" }
                                        td { style: "{form_cell}", "{pl}" }
                                        td { style: "{form_cell}", "{wn}" }
                                    }
                                }
                            }
                        }
                    }
                }

                // close
                div { style: "text-align:right; margin-top:1.3rem;",
                    button {
                        class: "focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-700",
                        style: "background:none; border:none; color:{accent}; cursor:pointer; \
                                font-size:0.85rem; font-family:{body};",
                        onclick: move |_| on_close.call(()),
                        "Close"
                    }
                }
            }
        }
    }
}
