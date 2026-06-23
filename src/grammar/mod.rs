//! Grammar content as data. One renderer (`components::Grammar`) walks a
//! `GrammarDoc`; adding a language means writing its `doc()` and a match arm —
//! no new UI. The content is authored (not user-generated), so it lives in Rust:
//! type-checked, and rich prose + tables don't fight a JSON schema.

mod georgian;

#[derive(Clone, PartialEq)]
pub enum Block {
    /// A paragraph of explanatory prose.
    Para(String),
    /// A highlighted call-out for a key intricacy.
    Note(String),
    /// An L2 example with romanization and gloss.
    Example {
        ka: String,
        translit: String,
        gloss: String,
    },
    /// A simple bulleted list.
    Bullets(Vec<String>),
    /// A captioned table (headers + rows).
    Table {
        caption: String,
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}

#[derive(Clone, PartialEq)]
pub struct Section {
    pub title: String,
    pub blocks: Vec<Block>,
}

#[derive(Clone, PartialEq)]
pub struct GrammarDoc {
    pub language: String,
    pub intro: String,
    pub sections: Vec<Section>,
}

/// Look up the grammar doc for a language. Unknown languages get a friendly
/// placeholder so the page never breaks.
pub fn doc_for(lang: &str) -> GrammarDoc {
    match lang {
        "georgian" => georgian::doc(),
        other => GrammarDoc {
            language: other.to_string(),
            intro: format!("Grammar notes for \u{201c}{other}\u{201d} haven't been written yet."),
            sections: Vec::new(),
        },
    }
}

// ── terse builders so the content modules read like an outline ──
pub(crate) fn para(s: &str) -> Block {
    Block::Para(s.to_string())
}
pub(crate) fn note(s: &str) -> Block {
    Block::Note(s.to_string())
}
pub(crate) fn ex(ka: &str, translit: &str, gloss: &str) -> Block {
    Block::Example {
        ka: ka.to_string(),
        translit: translit.to_string(),
        gloss: gloss.to_string(),
    }
}
pub(crate) fn bullets(items: &[&str]) -> Block {
    Block::Bullets(items.iter().map(|s| s.to_string()).collect())
}
pub(crate) fn table(caption: &str, headers: &[&str], rows: &[&[&str]]) -> Block {
    Block::Table {
        caption: caption.to_string(),
        headers: headers.iter().map(|s| s.to_string()).collect(),
        rows: rows
            .iter()
            .map(|r| r.iter().map(|c| c.to_string()).collect())
            .collect(),
    }
}
pub(crate) fn section(title: &str, blocks: Vec<Block>) -> Section {
    Section {
        title: title.to_string(),
        blocks,
    }
}
