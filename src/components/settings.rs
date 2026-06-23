// src/components/settings.rs
//
// The settings modal: a gear button that opens a dialog over the app. Reads and
// writes the shared Settings store (which persists itself). Drop `SettingsButton`
// into the navbar so it's reachable from every route.

use dioxus::prelude::*;
use dioxus_primitives::slider::SliderValue;

use crate::components::dialog::{DialogContent, DialogDescription, DialogRoot, DialogTitle};
use crate::components::slider::{Slider, SliderRange, SliderThumb, SliderTrack};
use crate::settings::use_settings;

const LANGS: [(&str, &str); 2] = [("georgian", "Georgian"), ("russian", "Russian")];

#[component]
pub fn SettingsButton() -> Element {
    let mut open = use_signal(|| false);
    let mut settings = use_settings();
    // the live (this-session) language, so changing the default also switches now
    let mut active_lang = use_context::<Signal<String>>();

    let volume = settings.read().volume;
    let volume_pct = (volume * 100.0).round() as i32;
    let default_lang = settings.read().default_language.clone();

    rsx! {
        button {
            class: "opacity-70 hover:opacity-100 transition-opacity hover:cursor-pointer",
            "aria-label": "Settings",
            onclick: move |_| open.set(true),
            "⚙"
        }

        // NOTE: `open`/`on_open_change` mirror your dioxus-primitives dialog. If the
        // compiler wants the signal itself, change `open: open()` to `open: open`.
        DialogRoot {
            open: open(),
            is_modal: true,
            on_open_change: move |v: bool| open.set(v),

            DialogContent {
                DialogTitle { "Settings" }
                DialogDescription { "Preferences are saved automatically." }

                // ── volume ───────────────────────────────────────────────
                div { style: "margin-top:1rem;",
                    div { style: "display:flex; justify-content:space-between; font-size:0.85rem; margin-bottom:0.35rem;",
                        span { "Volume" }
                        span { style: "opacity:0.7;", "{volume_pct}%" }
                    }
                    Slider {
                        default_value: SliderValue::Single(volume as f64),
                        min: 0.0,
                        max: 1.0,
                        step: 0.05,
                        horizontal: true,
                        on_value_change: move |value: SliderValue| {
                            let SliderValue::Single(v) = value;
                            settings.with_mut(|s| s.volume = v as f32);
                        },
                        SliderTrack {
                            SliderRange {}
                            SliderThumb {}
                        }
                    }
                }

                // ── default / boot language (also switches the current session) ──
                div { style: "margin-top:1.25rem;",
                    div { style: "font-size:0.85rem; margin-bottom:0.35rem;", "Default language" }
                    div { style: "display:flex; gap:0.5rem;",
                        for (code, name) in LANGS {
                            button {
                                key: "{code}",
                                style: format!(
                                    "padding:0.3rem 0.8rem; border-radius:0.5rem; cursor:pointer; border:1px solid {}; background:{}; color:{};",
                                    if default_lang == code { "#818cf8" } else { "#374151" },
                                    if default_lang == code { "#4f46e5" } else { "transparent" },
                                    if default_lang == code { "#ffffff" } else { "#d1d5db" },
                                ),
                                onclick: move |_| {
                                    settings.with_mut(|s| s.default_language = code.to_string());
                                    active_lang.set(code.to_string());
                                },
                                "{name}"
                            }
                        }
                    }
                    div { style: "font-size:0.7rem; opacity:0.6; margin-top:0.4rem;",
                        "Applied now and used on next launch."
                    }
                }
            }
        }
    }
}
