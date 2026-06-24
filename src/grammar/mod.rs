//! Grammar content as data. One renderer (`components::Grammar`) walks a
//! `GrammarDoc` for instruction; one quiz (`components::GrammarQuiz`) walks its
//! authored `Drill`s for assessment. Adding a language means writing its `doc()`
//! and a match arm — no new UI. Content is authored (not user-generated), so it
//! lives in Rust: type-checked, and rich prose + tables don't fight a schema.
//!
//! Note the deliberate split: `Block`s are for *reading* (formatted display),
//! `Drill`s are for *testing* (clean fields + accepted answers). Display tables
//! make poor auto-questions, so questions are authored, not derived.

mod georgian;

#[derive(Clone, PartialEq)]
pub enum Block {
    Para(String),
    Note(String),
    Example {
        ka: String,
        translit: String,
        gloss: String,
    },
    Bullets(Vec<String>),
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

/// How a drill is answered.
#[derive(Clone, PartialEq)]
pub enum Answer {
    /// Free text; any listed answer counts (graded fuzzily by the typing core).
    /// Best for producing forms — "give the ergative of კაცი" → ["კაცმა"].
    TypeIn(Vec<String>),
    /// Pick one of `options`; `options[correct]` is right. Best for concepts,
    /// where spelling the term shouldn't be the test.
    Choice {
        options: Vec<String>,
        correct: usize,
    },
}

/// One authored grammar question. `note` is the "why," shown after answering.
#[derive(Clone, PartialEq)]
pub struct Drill {
    pub prompt: String,
    pub answer: Answer,
    pub note: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct GrammarDoc {
    pub language: String,
    pub intro: String,
    pub sections: Vec<Section>,
    /// The question bank. Empty for languages whose drills aren't written yet.
    pub drills: Vec<Drill>,
}

/// Look up the grammar doc for a language. Unknown languages get a friendly
/// placeholder so neither the page nor the quiz breaks.
pub fn doc_for(lang: &str) -> GrammarDoc {
    match lang {
        "georgian" => georgian::doc(),
        other => GrammarDoc {
            language: other.to_string(),
            intro: format!("Grammar notes for \u{201c}{other}\u{201d} haven't been written yet."),
            sections: Vec::new(),
            drills: Vec::new(),
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

// drill builders
pub(crate) fn tin(prompt: &str, answers: &[&str], note: &str) -> Drill {
    Drill {
        prompt: prompt.to_string(),
        answer: Answer::TypeIn(answers.iter().map(|s| s.to_string()).collect()),
        note: (!note.is_empty()).then(|| note.to_string()),
    }
}
pub(crate) fn mc(prompt: &str, options: &[&str], correct: usize, note: &str) -> Drill {
    Drill {
        prompt: prompt.to_string(),
        answer: Answer::Choice {
            options: options.iter().map(|s| s.to_string()).collect(),
            correct,
        },
        note: (!note.is_empty()).then(|| note.to_string()),
    }
}
