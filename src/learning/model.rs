//! The brain: a pure fold over the evidence log into per-skill beliefs and
//! per-item memories, plus the prerequisite-floor inference. No I/O, no Dioxus.

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

pub struct LearnerModel {
    cfg: Config,
    skills: HashMap<Skill, Belief>,
    items: HashMap<ItemId, Memory>,
}

impl LearnerModel {
    pub fn new(cfg: Config) -> Self {
        let skills = Skill::iter()
            .map(|s| (s, Belief::prior(cfg.prior_mean, cfg.prior_var)))
            .collect();
        LearnerModel {
            cfg,
            skills,
            items: HashMap::new(),
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

    /// Fold a single observation: update the skill belief, the item memory, then
    /// lift any prerequisite floors.
    pub fn apply(&mut self, e: &Evidence) {
        if let Some(b) = self.skills.get_mut(&e.skill) {
            b.observe(e, self.cfg.q_per_day);
        }
        self.items
            .entry(e.item.clone())
            .or_insert_with(Memory::seed)
            .review(e.at, e.outcome >= 0.5);
        self.propagate_floor(e.skill, e.at);
    }

    /// Demonstrated competence at `from` raises (never lowers) the floor of its
    /// prerequisites, cascading up the DAG. Monotonic and directional.
    fn propagate_floor(&mut self, from: Skill, now: Millis) {
        let q = self.cfg.q_per_day;
        let margin = self.cfg.floor_margin;
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

    /// Per-axis mastery in `0.0..=1.0` for display (logistic of the conservative
    /// bound). Never-observed axes report `None`.
    pub fn mastery(&self, now: Millis) -> HashMap<Skill, Option<f32>> {
        Skill::iter()
            .map(|s| {
                let b = &self.skills[&s];
                let v = b.observed().then(|| {
                    logistic(b.conservative(now, self.cfg.q_per_day, self.cfg.conservative_k))
                });
                (s, v)
            })
            .collect()
    }

    pub fn belief(&self, s: Skill) -> Belief {
        self.skills[&s]
    }

    pub fn due(&self, now: Millis, target: f32) -> Vec<ItemId> {
        due_items(&self.items, now, target)
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::learning::evidence::{Evidence, Millis, Source};
    use crate::learning::model::{Config, LearnerModel};
    use crate::learning::skill::Skill;

    fn ev(at: Millis, skill: Skill, outcome: f32) -> Evidence {
        Evidence::new(at, "x", skill, outcome, 0.0, 800, Source::Wpm)
    }

    #[test]
    fn evidence_moves_mean_but_time_does_not() {
        let mut m = LearnerModel::new(Config::default());
        let before = m.belief(Skill::ScriptSound).mean;
        m.apply(&ev(1_000, Skill::ScriptSound, 1.0));
        let b = m.belief(Skill::ScriptSound);
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
        let reading = m.belief(Skill::Reading).mean;
        let script = m.belief(Skill::ScriptSound).mean;
        assert!(script > 0.0, "reading should floor script above its prior");
        assert!(
            script < reading,
            "the floor sits below the demonstrated skill"
        );
        assert_eq!(
            m.belief(Skill::Listening).mean,
            Config::default().prior_mean,
            "reading must not touch listening (no sideways flow)"
        );
    }

    #[test]
    fn untested_axis_reports_none() {
        let m = LearnerModel::new(Config::default());
        assert!(m.mastery(10_000)[&Skill::Reading].is_none());
    }
}
