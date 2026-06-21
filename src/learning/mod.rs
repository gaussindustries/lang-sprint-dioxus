//! The learner model — a pure, Dioxus-free brain that turns evidence into
//! per-skill mastery estimates. The defining choice lives in `belief.rs`: the
//! mean moves only on evidence, time only grows uncertainty (input-driven, not
//! feed-forward). Everything else (radar, scheduler, dictionary loop) consumes
//! this.

pub mod belief;
pub mod evidence;
pub mod model;
pub mod scheduler;
pub mod skill;
pub mod store;

pub use belief::Belief;
pub use evidence::{word_accuracy, Evidence, ItemId, Millis, Source};
pub use model::{Config, LearnerModel};
pub use scheduler::Memory;
pub use skill::Skill;
