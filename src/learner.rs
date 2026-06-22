// src/learner.rs
//
// The Dioxus-facing bridge to the pure `learning` brain. It owns the in-memory
// evidence log (a Signal), persists each new event to disk, and exposes a cheap
// `emit` for drills plus a freshly-folded `LearnerModel` for readers like the
// dashboard. The brain in `learning/` never imports any of this.

use std::path::PathBuf;
use std::sync::Arc;

use dioxus::prelude::*;

use crate::learning::{store, Evidence, LearnerModel, Millis};

/// Current time in unix-epoch ms (desktop). On wasm this needs a JS shim.
pub fn now_ms() -> Millis {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Where the append-only evidence log lives. Linux/XDG for now; swap in the
/// `directories` crate when you want macOS/Windows correctness.
fn log_path() -> PathBuf {
    let base = std::env::var("XDG_DATA_HOME")
        .ok()
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| format!("{h}/.local/share"))
        })
        .unwrap_or_else(|| ".".to_string());
    PathBuf::from(base)
        .join("lang-sprint")
        .join("evidence.jsonl")
}

/// Cheap, cloneable handle that drills and views read from context.
#[derive(Clone)]
pub struct Learner {
    log: Signal<Vec<Evidence>>,
    path: Arc<PathBuf>,
}

impl Learner {
    /// Append one observation: persist it, then push to the in-memory log so any
    /// `model()` reader recomputes. Persistence failure is logged, not fatal.
    pub fn emit(&self, e: Evidence) {
        if let Err(err) = store::append(&self.path, &e) {
            eprintln!("evidence persist failed: {err}");
        }
        let mut log = self.log;
        log.write().push(e);
    }

    pub fn emit_all(&self, evidence: impl IntoIterator<Item = Evidence>) {
        for e in evidence {
            self.emit(e);
        }
    }

    /// Fold the whole log into a model (cheap; the log is small).
    pub fn model(&self) -> LearnerModel {
        LearnerModel::from_log(&self.log.read())
    }

    pub fn log_len(&self) -> usize {
        self.log.read().len()
    }
}

/// Call once at the App root: loads any persisted log and provides the handle.
pub fn provide_learner() {
    let path = log_path();
    let log = use_signal({
        let p = path.clone();
        move || store::load_log(&p)
    });
    use_context_provider(|| Learner {
        log,
        path: Arc::new(path),
    });
}

/// Read the handle anywhere beneath `provide_learner`.
pub fn use_learner() -> Learner {
    use_context::<Learner>()
}

/// Build per-word evidence from a finished WPM run. The current WPM test is
/// copy-typing displayed L2 words, so it bears on Script & Sound (decode/encode
/// the script under time pressure) — not yet Vocab-Production, which needs an
/// L1→L2 prompt. Adjust skill/difficulty as drills specialize.
pub fn wpm_evidence(
    lang: &str,
    target: &str,
    typed: &str,
    elapsed_secs: f64,
    now: Millis,
) -> Vec<Evidence> {
    use crate::learning::{word_accuracy, Skill, Source};

    let tw: Vec<&str> = target.split_whitespace().collect();
    let yw: Vec<&str> = typed.split_whitespace().collect();
    if tw.is_empty() {
        return Vec::new();
    }
    let per_word_ms = ((elapsed_secs * 1000.0) / tw.len() as f64).max(0.0) as u32;
    tw.iter()
        .enumerate()
        .map(|(i, t)| {
            let y = yw.get(i).copied().unwrap_or("");
            Evidence::new(
                now,
                lang,
                format!("{lang}:word:{t}"),
                Skill::ScriptSound,
                word_accuracy(t, y),
                0.0, // TODO: scale difficulty by length / frequency rank
                per_word_ms,
                Source::Wpm,
            )
        })
        .collect()
}

/// Evidence from the word drill (copy-typing a displayed L2 word). The answer is
/// on screen, so this is mostly a Script & Sound / hand-eye signal, logged at a
/// low difficulty so a clean fast type nudges the axis only a little — it can't
/// stand in for recall. `difficulty` is the weighting knob; lower = counts less.
pub fn word_drill_evidence(lang: &str, word: &str, latency_ms: u32) -> Vec<Evidence> {
    use crate::learning::{Skill, Source};
    if word.trim().is_empty() {
        return Vec::new();
    }
    vec![Evidence::new(
        now_ms(),
        lang,
        format!("{lang}:word:{word}"),
        Skill::ScriptSound,
        1.0,  // the drill only advances on a correct match
        -1.0, // low difficulty: success here is weak positive evidence
        latency_ms,
        Source::WordDrill,
    )]
}

/// Evidence from the timed meaning-recall drill ("WPM, but type the meaning").
/// Answering in English tests recognition (L2 word -> meaning); answering in the
/// target language tests production (meaning -> L2 word). Full difficulty (0.0):
/// this is real recall, not copy-typing. `grade` is the continuous score from the
/// grading core (1.0 exact; lower for typos / wrong answers).
pub fn meaning_evidence(
    lang: &str,
    to_english: bool,
    headword: &str,
    grade: f32,
    latency_ms: u32,
) -> Vec<Evidence> {
    use crate::learning::{Skill, Source};
    if headword.trim().is_empty() {
        return Vec::new();
    }
    let (skill, item) = if to_english {
        (Skill::VocabRecognition, format!("{lang}:rec:{headword}"))
    } else {
        (Skill::VocabProduction, format!("{lang}:prod:{headword}"))
    };
    vec![Evidence::new(
        now_ms(),
        lang,
        item,
        skill,
        grade,
        0.0,
        latency_ms,
        Source::Recall,
    )]
}
