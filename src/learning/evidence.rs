//! The single unit of input to the model. Everything the learner does that
//! says something about ability becomes one of these, appended to a log. The
//! model is a pure fold over that log, so it's replayable and re-derivable.

use serde::{Deserialize, Serialize};

use super::skill::Skill;

/// Unix epoch milliseconds.
pub type Millis = u64;

/// Stable identifier for the thing tested, e.g. `"georgian:word:კაცი"`. Stringly
/// typed on purpose so any drill can mint one.
pub type ItemId = String;

/// Which drill (or non-drill action) produced the evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Wpm,
    WordDrill,
    Reading,
    Listening,
    Conjugation,
    /// A dictionary lookup — weak "encountered" signal that also queues the item.
    Lookup,
    /// A scheduler-initiated probe to resolve uncertainty.
    Probe,
    /// Timed meaning-recall drill (“WPM, but type the meaning”).
    Recall,
    /// Authored grammar drill (the grammar “Practice” mode).
    Grammar,
}

/// One graded observation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    pub at: Millis,
    /// Language bucket, e.g. `"georgian"`. `serde(default)` so logs written before
    /// this field existed still parse; `lang_key()` then recovers the language
    /// from the item-id prefix.
    #[serde(default)]
    pub lang: String,
    pub item: ItemId,
    pub skill: Skill,
    /// Graded outcome in `0.0..=1.0` (1.0 = fully correct).
    pub outcome: f32,
    /// Item difficulty on the same scale as ability (logit-like).
    pub difficulty: f32,
    /// Response time, for the automaticity signal.
    pub latency_ms: u32,
    pub source: Source,
}

impl Evidence {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        at: Millis,
        lang: impl Into<String>,
        item: impl Into<ItemId>,
        skill: Skill,
        outcome: f32,
        difficulty: f32,
        latency_ms: u32,
        source: Source,
    ) -> Self {
        Self {
            at,
            lang: lang.into(),
            item: item.into(),
            skill,
            outcome: outcome.clamp(0.0, 1.0),
            difficulty,
            latency_ms,
            source,
        }
    }

    /// The language bucket this evidence belongs to: the `lang` field if set,
    /// else the item-id prefix before the first ':' (migrates pre-`lang` logs).
    pub fn lang_key(&self) -> &str {
        if !self.lang.is_empty() {
            &self.lang
        } else {
            self.item.split(':').next().unwrap_or("")
        }
    }
}

/// Cheap word grader: exact → 1.0, else fraction of aligned chars that match.
/// Swap for edit distance when the reading drill needs fuzzy answers.
pub fn word_accuracy(target: &str, typed: &str) -> f32 {
    if typed == target {
        return 1.0;
    }
    let t: Vec<char> = target.chars().collect();
    if t.is_empty() || typed.is_empty() {
        return 0.0;
    }
    let matches = t
        .iter()
        .zip(typed.chars())
        .filter(|(a, b)| **a == *b)
        .count();
    matches as f32 / t.len() as f32
}
