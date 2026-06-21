//! Proficiency axes (the radar's dimensions) and the prerequisite DAG.
//!
//! A `Skill` is one measurable axis. Drills emit evidence toward a skill; the
//! model turns that into a per-skill mastery estimate. The prerequisite edges
//! drive the "demonstrated downstream competence floors its prerequisites"
//! rule in `model.rs` — and that floor only ever lifts, never bleeds sideways.

use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter)]
#[serde(rename_all = "snake_case")]
pub enum Skill {
    ScriptSound,
    Listening,
    VocabRecognition,
    VocabProduction,
    Grammar,
    Reading,
}

impl Skill {
    /// Skills that must hold for this one to be demonstrable. Floors propagate
    /// from a skill *up* to these; never the reverse, never to siblings.
    pub fn prerequisites(self) -> &'static [Skill] {
        use Skill::*;
        match self {
            Reading => &[ScriptSound, VocabRecognition, Grammar],
            VocabProduction => &[ScriptSound, VocabRecognition],
            VocabRecognition => &[ScriptSound],
            Grammar => &[VocabRecognition],
            Listening => &[],
            ScriptSound => &[],
        }
    }

    /// Human label for the dashboard.
    pub fn label(self) -> &'static str {
        use Skill::*;
        match self {
            ScriptSound => "Script & Sound",
            Listening => "Listening",
            VocabRecognition => "Vocab — Recognition",
            VocabProduction => "Vocab — Production",
            Grammar => "Grammar",
            Reading => "Reading",
        }
    }
}
