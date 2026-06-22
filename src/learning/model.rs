//! The brain: a pure fold over the evidence log into per-skill beliefs and
//! per-item memories, plus the prerequisite-floor inference. No I/O, no Dioxus.
//!
//! Everything is bucketed by language. Skills NEVER mix across languages —
//! Georgian VocabRecognition and Russian VocabRecognition are independent
//! beliefs, and the prerequisite floor only propagates within one language.

use std::collections::HashMap;

use strum::IntoEnumIterator;

use super::belief::{logistic, Belief};
use super::evidence::{Evidence, ItemId, Millis};
use super::scheduler::{due_items, Memory};
use super::skill::Skill;

/// Tunables. Defaults are a starting point — expect to sweep these against logs.
#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Variance added per idle day (uncertainty growth, not assumed forgetting).
    pub q_per_day: f32,
    pub prior_mean: f32,
    pub prior_var: f32,
    /// Radar shows `mean − conservative_k · sd`.
    pub conservative_k: f32,
    /// A demonstrated skill floors each prerequisite to `downstream_mean − margin`.
    pub floor_margin: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            q_per_day: 0.05,
            prior_mean: 0.0,
            prior_var: 1.0,
            conservative_k: 1.0,
            floor_margin: 0.25,
        }
    }
}

/// All beliefs and memories for ONE language.
struct LangState {
    skills: HashMap<Skill, Belief>,
    items: HashMap<ItemId, Memory>,
    events: u64,
}

impl LangState {
    fn new(cfg: &Config) -> Self {
        let skills = Skill::iter()
            .map(|s| (s, Belief::prior(cfg.prior_mean, cfg.prior_var)))
            .collect();
        LangState {
            skills,
            items: HashMap::new(),
            events: 0,
        }
    }

    fn observe(&mut self, e: &Evidence, cfg: &Config) {
        if let Some(b) = self.skills.get_mut(&e.skill) {
            b.observe(e, cfg.q_per_day);
        }
        self.items
            .entry(e.item.clone())
            .or_insert_with(Memory::seed)
            .review(e.at, e.outcome >= 0.5);
        self.events += 1;
        self.propagate_floor(e.skill, e.at, cfg);
    }

    /// Demonstrated competence at `from` raises (never lowers) the floor of its
    /// prerequisites, cascading up the DAG. Monotonic, directional, within-language.
    fn propagate_floor(&mut self, from: Skill, now: Millis, cfg: &Config) {
        let q = cfg.q_per_day;
        let margin = cfg.floor_margin;
        let mut stack = vec![from];
        while let Some(s) = stack.pop() {
            let dmean = self
                .skills
                .get(&s)
                .map(|b| b.at(now, q).mean)
                .unwrap_or(0.0);
            let floor = dmean - margin;
            for &p in s.prerequisites() {
                let cur = self
                    .skills
                    .get(&p)
                    .map(|b| b.at(now, q).mean)
                    .unwrap_or(0.0);
                if floor > cur {
                    if let Some(b) = self.skills.get_mut(&p) {
                        b.mean = floor;
                        b.last_seen = now;
                    }
                    stack.push(p);
                }
            }
        }
    }

    fn mastery(&self, now: Millis, cfg: &Config) -> HashMap<Skill, Option<f32>> {
        Skill::iter()
            .map(|s| {
                let b = &self.skills[&s];
                let v = b
                    .observed()
                    .then(|| logistic(b.conservative(now, cfg.q_per_day, cfg.conservative_k)));
                (s, v)
            })
            .collect()
    }

    fn mean_mastery(&self, now: Millis, cfg: &Config) -> Option<f32> {
        let vals: Vec<f32> = Skill::iter()
            .filter_map(|s| {
                let b = &self.skills[&s];
                b.observed()
                    .then(|| logistic(b.conservative(now, cfg.q_per_day, cfg.conservative_k)))
            })
            .collect();
        if vals.is_empty() {
            None
        } else {
            Some(vals.iter().sum::<f32>() / vals.len() as f32)
        }
    }
}

pub struct LearnerModel {
    cfg: Config,
    langs: HashMap<String, LangState>,
}

impl LearnerModel {
    pub fn new(cfg: Config) -> Self {
        LearnerModel {
            cfg,
            langs: HashMap::new(),
        }
    }

    /// Rebuild from scratch by folding the whole log (event-sourced).
    pub fn from_log(log: &[Evidence]) -> Self {
        Self::from_log_with(Config::default(), log)
    }

    pub fn from_log_with(cfg: Config, log: &[Evidence]) -> Self {
        let mut m = Self::new(cfg);
        for e in log {
            m.apply(e);
        }
        m
    }

    /// Fold a single observation into its language bucket.
    pub fn apply(&mut self, e: &Evidence) {
        let key = e.lang_key().to_string();
        let cfg = self.cfg;
        self.langs
            .entry(key)
            .or_insert_with(|| LangState::new(&cfg))
            .observe(e, &cfg);
    }

    /// Per-axis mastery for one language. A language with no evidence (or any
    /// untouched axis) reports `None` so the radar shows it at center.
    pub fn mastery(&self, lang: &str, now: Millis) -> HashMap<Skill, Option<f32>> {
        match self.langs.get(lang) {
            Some(l) => l.mastery(now, &self.cfg),
            None => Skill::iter().map(|s| (s, None)).collect(),
        }
    }

    pub fn belief(&self, lang: &str, s: Skill) -> Belief {
        match self.langs.get(lang) {
            Some(l) => l.skills[&s],
            None => Belief::prior(self.cfg.prior_mean, self.cfg.prior_var),
        }
    }

    pub fn due(&self, lang: &str, now: Millis, target: f32) -> Vec<ItemId> {
        match self.langs.get(lang) {
            Some(l) => due_items(&l.items, now, target),
            None => Vec::new(),
        }
    }

    pub fn item_count(&self, lang: &str) -> usize {
        self.langs.get(lang).map(|l| l.items.len()).unwrap_or(0)
    }

    pub fn event_count(&self, lang: &str) -> usize {
        self.langs.get(lang).map(|l| l.events as usize).unwrap_or(0)
    }

    pub fn total_events(&self) -> usize {
        self.langs.values().map(|l| l.events as usize).sum()
    }

    /// Languages with at least one observation, sorted.
    pub fn languages(&self) -> Vec<String> {
        let mut v: Vec<String> = self.langs.keys().cloned().collect();
        v.sort();
        v
    }

    /// Mean mastery across tested axes for a language (for summary chips); `None`
    /// if nothing tested yet.
    pub fn mean_mastery(&self, lang: &str, now: Millis) -> Option<f32> {
        self.langs
            .get(lang)
            .and_then(|l| l.mean_mastery(now, &self.cfg))
    }
}

#[cfg(test)]
mod tests {
    use crate::learning::evidence::{Evidence, Millis, Source};
    use crate::learning::model::{Config, LearnerModel};
    use crate::learning::skill::Skill;

    fn ev(at: Millis, skill: Skill, outcome: f32) -> Evidence {
        Evidence::new(at, "ka", "x", skill, outcome, 0.0, 800, Source::Wpm)
    }

    #[test]
    fn evidence_moves_mean_but_time_does_not() {
        let mut m = LearnerModel::new(Config::default());
        let before = m.belief("ka", Skill::ScriptSound).mean;
        m.apply(&ev(1_000, Skill::ScriptSound, 1.0));
        let b = m.belief("ka", Skill::ScriptSound);
        assert!(b.mean > before, "a correct answer must raise the mean");

        let later = 1_000 + 30 * 86_400_000; // 30 days, no evidence
        let projected = b.at(later, Config::default().q_per_day);
        assert!(
            (projected.mean - b.mean).abs() < 1e-6,
            "time must not move the mean"
        );
        assert!(projected.var > b.var, "time must grow the variance");
    }

    #[test]
    fn floor_lifts_prereqs_not_siblings() {
        let mut m = LearnerModel::new(Config::default());
        for i in 0..10 {
            m.apply(&ev(1_000 + i, Skill::Reading, 1.0));
        }
        let reading = m.belief("ka", Skill::Reading).mean;
        let script = m.belief("ka", Skill::ScriptSound).mean;
        assert!(script > 0.0, "reading should floor script above its prior");
        assert!(
            script < reading,
            "the floor sits below the demonstrated skill"
        );
        assert_eq!(
            m.belief("ka", Skill::Listening).mean,
            Config::default().prior_mean,
            "reading must not touch listening (no sideways flow)"
        );
    }

    #[test]
    fn untested_axis_reports_none() {
        let m = LearnerModel::new(Config::default());
        assert!(m.mastery("ka", 10_000)[&Skill::Reading].is_none());
    }

    #[test]
    fn languages_are_separate_buckets() {
        let mut m = LearnerModel::new(Config::default());
        for i in 0..10 {
            m.apply(&Evidence::new(
                1_000 + i,
                "georgian",
                "g",
                Skill::Reading,
                1.0,
                0.0,
                800,
                Source::Reading,
            ));
        }
        m.apply(&Evidence::new(
            2_000,
            "russian",
            "r",
            Skill::ScriptSound,
            1.0,
            0.0,
            800,
            Source::Wpm,
        ));

        // Georgian reading is tested; Russian reading is not.
        assert!(m.mastery("georgian", 3_000)[&Skill::Reading].is_some());
        assert!(m.mastery("russian", 3_000)[&Skill::Reading].is_none());
        // Russian script is tested directly.
        assert!(m.mastery("russian", 3_000)[&Skill::ScriptSound].is_some());
        // Georgian script floored by Georgian reading — but that's a Georgian fact only.
        assert!(m.belief("georgian", Skill::ScriptSound).mean > 0.0);
        assert_eq!(
            m.languages(),
            vec!["georgian".to_string(), "russian".to_string()]
        );
        assert_eq!(m.event_count("georgian"), 10);
        assert_eq!(m.event_count("russian"), 1);
    }
}
