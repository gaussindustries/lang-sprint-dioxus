// src/components/grammar.rs
//
// One renderer for all grammar content. Feed it a language; it looks up that
// language's `GrammarDoc` and walks the typed blocks. Adding a language touches
// only the content module (crate::grammar) — never this file.
//
// Styled inline (not Tailwind) so the reference page renders identically
// regardless of what the JIT emits, and a Georgian-capable serif is set on the
// container so inline Georgian in the prose renders too.

use dioxus::prelude::*;

use crate::grammar::{doc_for, Block};

const GEO: &str = "'Noto Serif Georgian','Noto Serif',Georgia,serif";

#[component]
pub fn Grammar(lang: String) -> Element {
    let doc = doc_for(&lang);

    rsx! {
        div { style: "max-width:42rem; margin:0 auto; font-family:{GEO};",
            if !doc.intro.is_empty() {
                p { style: "color:#cbd5e1; line-height:1.65; margin-bottom:1.75rem;", "{doc.intro}" }
            }
            for (si, sec) in doc.sections.iter().enumerate() {
                section { key: "{si}", style: "margin-bottom:2rem;",
                    h2 {
                        style: "font-size:1.15rem; font-weight:600; color:#a5b4fc; \
                                border-bottom:1px solid #374151; padding-bottom:0.35rem; margin-bottom:0.85rem;",
                        "{sec.title}"
                    }
                    for (bi, block) in sec.blocks.iter().cloned().enumerate() {
                        BlockView { key: "{si}-{bi}", block }
                    }
                }
            }
        }
    }
}

#[component]
fn BlockView(block: Block) -> Element {
    match block {
        Block::Para(t) => rsx! {
            p { style: "color:#d1d5db; line-height:1.65; margin:0.55rem 0;", "{t}" }
        },
        Block::Note(t) => rsx! {
            div {
                style: "border-left:3px solid #6366f1; background:rgba(99,102,241,0.08); color:#e0e7ff; \
                        padding:0.65rem 0.9rem; margin:0.85rem 0; border-radius:0 0.375rem 0.375rem 0; \
                        font-size:0.92rem; line-height:1.55;",
                "{t}"
            }
        },
        Block::Example {
            ka,
            translit,
            gloss,
        } => rsx! {
            div { style: "margin:0.6rem 0; padding:0.55rem 0.8rem; background:#111827; border-radius:0.5rem;",
                div { style: "font-size:1.3rem; color:#f3f4f6; line-height:1.4;", "{ka}" }
                div { style: "font-size:0.8rem; color:#9ca3af; font-style:italic; margin-top:0.1rem;", "{translit}" }
                div { style: "font-size:0.92rem; color:#cbd5e1; margin-top:0.15rem;", "{gloss}" }
            }
        },
        Block::Bullets(items) => rsx! {
            ul { style: "margin:0.5rem 0 0.5rem 1.15rem; color:#d1d5db; line-height:1.65;",
                for (i, it) in items.iter().cloned().enumerate() {
                    li { key: "{i}", style: "margin:0.25rem 0;", "{it}" }
                }
            }
        },
        Block::Table {
            caption,
            headers,
            rows,
        } => rsx! {
            div { style: "margin:0.9rem 0;",
                if !caption.is_empty() {
                    div { style: "font-size:0.8rem; color:#9ca3af; margin-bottom:0.4rem;", "{caption}" }
                }
                table { style: "width:100%; border-collapse:collapse;",
                    thead {
                        tr {
                            for (hi, h) in headers.iter().cloned().enumerate() {
                                th {
                                    key: "{hi}",
                                    style: "text-align:left; font-size:0.68rem; text-transform:uppercase; \
                                            letter-spacing:0.05em; color:#9ca3af; background:#1f2937; \
                                            border:1px solid #374151; padding:0.4rem 0.6rem;",
                                    "{h}"
                                }
                            }
                        }
                    }
                    tbody {
                        for (ri, row) in rows.iter().cloned().enumerate() {
                            tr { key: "{ri}",
                                for (ci, cell) in row.iter().cloned().enumerate() {
                                    td {
                                        key: "{ci}",
                                        style: "border:1px solid #374151; padding:0.4rem 0.6rem; \
                                                color:#e5e7eb; font-size:0.9rem; vertical-align:top;",
                                        "{cell}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    }
}
