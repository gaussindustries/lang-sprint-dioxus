//! App-wide settings: a tiny serde struct loaded from disk at boot and shared
//! via context. Audio reads `volume`; the app boots into `default_language`.
//! Anything that needs a setting calls `use_settings()` and reads the field —
//! the single source of truth. Saved to disk automatically on change.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// Master playback volume, 0.0..=1.0.
    #[serde(default = "default_volume")]
    pub volume: f32,
    /// Language the app boots into (e.g. "georgian").
    #[serde(default = "default_language")]
    pub default_language: String,
    #[serde(default)]
    pub tts_enabled: bool,
}

fn default_volume() -> f32 {
    0.4
}
fn default_language() -> String {
    "georgian".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            volume: default_volume(),
            default_language: default_language(),
            tts_enabled: false,
        }
    }
}

impl Settings {
    /// Read settings from disk, falling back to defaults if missing/corrupt.
    pub fn load() -> Self {
        match settings_path().and_then(|p| fs::read_to_string(p).ok()) {
            Some(s) => serde_json::from_str(&s).unwrap_or_default(),
            None => Settings::default(),
        }
    }

    /// Write settings to disk (best-effort; ignores I/O errors).
    pub fn save(&self) {
        let Some(path) = settings_path() else {
            return;
        };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }
}

// XDG_DATA_HOME/lang-sprint/settings.json (Linux desktop), same home as the
// learner's evidence log.
#[cfg(not(target_arch = "wasm32"))]
fn settings_path() -> Option<PathBuf> {
    crate::paths::data_root().map(|d| d.join("settings.json"))
}

#[cfg(target_arch = "wasm32")]
fn settings_path() -> Option<PathBuf> {
    None
}

// ─── Dioxus glue ───────────────────────────────────────────────────────────

use dioxus::prelude::*;

/// Call ONCE at the App root (like `provide_learner`). Loads settings, provides
/// them via context, and persists to disk whenever they change. Returns the
/// signal so the caller can seed the boot language from it.
pub fn provide_settings() -> Signal<Settings> {
    let settings = use_signal(Settings::load);
    use_context_provider(|| settings);
    // save on change (also writes once on mount — harmless)
    use_effect(move || {
        let snapshot = settings();
        snapshot.save();
    });
    settings
}

/// Read the shared settings anywhere below the provider.
pub fn use_settings() -> Signal<Settings> {
    use_context::<Signal<Settings>>()
}
