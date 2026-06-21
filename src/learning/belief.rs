//! A skill's belief: a Gaussian over latent ability. The mean moves *only* on
//! evidence; elapsed time inflates the variance (uncertainty), never the mean.
//! This is Glicko seen as a 1-D Kalman filter — the resolution of "driven by
//! input, not feed-forward". The scheduler's half-life decides *when* to
//! re-measure; the answer is the only thing that moves the estimate.

use super::evidence::{Evidence, Millis};

/// Logistic squash, shared with the model/dashboard for logit ↔ unit maps.
pub fn logistic(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// A correct answer counts for more when fast (automaticity); a wrong answer
/// stays wrong regardless of speed.
fn speed_adjust(outcome: f32, latency_ms: u32) -> f32 {
    if outcome <= 0.0 {
        return 0.0;
    }
    let secs = latency_ms as f32 / 1000.0;
    let fast = (1.5 / (1.0 + (secs - 1.5).max(0.0))).min(1.0); // ~1 under 1.5s, decays after
    ((0.7 + 0.3 * fast) * outcome).clamp(0.0, 1.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Belief {
    pub mean: f32,
    pub var: f32,
    pub last_seen: Millis,
}

impl Belief {
    pub fn prior(mean: f32, var: f32) -> Self {
        Belief {
            mean,
            var,
            last_seen: 0,
        }
    }

    pub fn observed(&self) -> bool {
        self.last_seen != 0
    }

    /// Project to `now`: mean unchanged, variance grows with idle time.
    /// `q_per_day` is process noise — how fast confidence rots.
    pub fn at(&self, now: Millis, q_per_day: f32) -> Belief {
        if self.last_seen == 0 {
            return *self;
        }
        let days = now.saturating_sub(self.last_seen) as f32 / 86_400_000.0;
        Belief {
            mean: self.mean,
            var: self.var + q_per_day * days.max(0.0),
            last_seen: self.last_seen,
        }
    }

    /// Fold one observation in (Kalman update on the projected prior).
    pub fn observe(&mut self, e: &Evidence, q_per_day: f32) {
        let prior = self.at(e.at, q_per_day);
        let expected = logistic(prior.mean - e.difficulty);
        let y = speed_adjust(e.outcome, e.latency_ms);
        // measurement variance ~ inverse Fisher information of a Bernoulli trial
        let meas_var = 1.0 / (expected * (1.0 - expected)).max(1e-3);
        let k = prior.var / (prior.var + meas_var);
        self.mean = prior.mean + k * (y - expected);
        self.var = ((1.0 - k) * prior.var).max(1e-3);
        self.last_seen = e.at;
    }

    /// Conservative ability for the radar: `mean − k·sd`. Contracts as variance
    /// grows, so neglected axes visibly pull in until you re-probe.
    pub fn conservative(&self, now: Millis, q_per_day: f32, k: f32) -> f32 {
        let b = self.at(now, q_per_day);
        b.mean - k * b.var.sqrt()
    }
}
