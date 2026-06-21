//! Item-level memory, for *scheduling only*. It predicts when to re-ask an
//! item; it never feeds the mastery estimate (that's evidence-only). FSRS/SM-2
//! can slot in later — the contract is just `retrievability(now)`.

use std::collections::HashMap;

use super::evidence::{ItemId, Millis};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Memory {
    pub half_life_days: f32,
    pub last_seen: Millis,
    pub reps: u32,
}

impl Memory {
    pub fn seed() -> Self {
        Memory {
            half_life_days: 1.0,
            last_seen: 0,
            reps: 0,
        }
    }

    /// Predicted recall probability at `now` (decides what's due).
    pub fn retrievability(&self, now: Millis) -> f32 {
        if self.last_seen == 0 {
            return 0.0;
        }
        let days = now.saturating_sub(self.last_seen) as f32 / 86_400_000.0;
        0.5f32.powf(days / self.half_life_days.max(1e-3))
    }

    /// Expand the half-life on success, contract on failure (HLR-flavoured).
    pub fn review(&mut self, now: Millis, success: bool) {
        self.reps = self.reps.saturating_add(1);
        let factor = if success { 1.8 } else { 0.5 };
        self.half_life_days = (self.half_life_days * factor).clamp(0.02, 365.0);
        self.last_seen = now;
    }
}

/// Items whose predicted recall has fallen below `target`, most overdue first.
pub fn due_items(mems: &HashMap<ItemId, Memory>, now: Millis, target: f32) -> Vec<ItemId> {
    let mut due: Vec<(f32, ItemId)> = mems
        .iter()
        .filter_map(|(id, m)| {
            let r = m.retrievability(now);
            (r < target).then(|| (r, id.clone()))
        })
        .collect();
    due.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    due.into_iter().map(|(_, id)| id).collect()
}
