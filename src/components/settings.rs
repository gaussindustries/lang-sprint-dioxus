// src/components/settings.rs
//
// Settings modal: volume, boot/default language, and an OPT-IN Pronunciation
// (TTS) section. TTS is off by default; the toggle is the opt-in. When on, it
// detects espeak-ng and — if missing — shows install guidance branched per OS
// with cfg!(windows) (NOT the windows_subsystem attribute, which is unrelated).

use dioxus::prelude::*;
use dioxus_primitives::slider::SliderValue;

use crate::components::dialog::{DialogContent, DialogDescription, DialogRoot, DialogTitle};
use crate::components::slider::{Slider, SliderRange, SliderThumb, SliderTrack};
use crate::components::toggle::Toggle;
use crate::settings::use_settings;

const LANGS: [(&str, &str); 2] = [("georgian", "Georgian"), ("russian", "Russian")];

fn nice(lang: &str) -> String {
    let mut c = lang.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => "this language".to_string(),
    }
}

#[component]
pub fn SettingsButton() -> Element {
    let mut open = use_signal(|| false);
    let mut settings = use_settings();
    let mut active_lang = use_context::<Signal<String>>();

    let volume = settings.read().volume;
    let volume_pct = (volume * 100.0).round() as i32;
    let default_lang = settings.read().default_language.clone();
    let tts_enabled = settings.read().tts_enabled;

    // ── TTS detection (re-runs on language change or manual re-check) ──
    let mut recheck = use_signal(|| 0u32);
    let probe = use_resource(move || {
        let _ = recheck();
        let lang = active_lang();
        async move {
            (
                crate::audio::espeak_present(),
                crate::audio::voice_available(&lang),
                crate::audio::espeak_data_dir().is_some(),
            )
        }
    });
    let probe_now = probe.read().clone();
    let lang_name = nice(&active_lang());

    // Platform-specific "how to install the engine" guidance.
    let (engine_hint, engine_cmd): (&str, Option<&str>) = if cfg!(windows) {
        (
            "Spoken pronunciation needs the eSpeak NG engine. Install it from espeak-ng (GitHub releases), \
             or bundle it with the app, then re-check.",
            None,
        )
    } else {
        (
            "Spoken pronunciation needs the espeak-ng engine. Install it with your package manager \
             (e.g. on Arch), then re-check:",
            Some("sudo pacman -S espeak-ng"),
        )
    };

    let recheck_btn = "margin-top:0.45rem; padding:0.3rem 0.7rem; border-radius:0.4rem; background:#374151; color:#e5e7eb; cursor:pointer; border:none; font-size:0.8rem;";

    rsx! {
        button {
            class: "opacity-70 hover:opacity-100 transition-opacity hover:cursor-pointer",
            "aria-label": "Settings",
            onclick: move |_| open.set(true),
            "⚙"
        }

        DialogRoot {
            open: open(),
            is_modal: true,
            on_open_change: move |v: bool| open.set(v),

            DialogContent {
                DialogTitle { "Settings" }
                DialogDescription { "Preferences are saved automatically." }

                // ── volume ──
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

                // ── pronunciation / text-to-speech (opt-in) ──
                div { style: "margin-top:1.25rem;",
                    div { style: "display:flex; justify-content:space-between; align-items:center; gap:0.75rem;",
                        span { style: "font-size:0.85rem;", "Pronunciation (text-to-speech)" }
                        Toggle {
                            pressed: tts_enabled,
                            on_pressed_change: move |on: bool| {
                                settings.with_mut(|s| s.tts_enabled = on);
                                if on {
                                    recheck.with_mut(|n| *n += 1); // re-probe when turning it on
                                }
                            },
                            if tts_enabled { "On" } else { "Off" }
                        }
                    }

                    if tts_enabled {
                        div { style: "margin-top:0.5rem;",
                            {
                                match probe_now {
                                    None => rsx! {
                                        div { style: "font-size:0.8rem; color:#9ca3af;", "Checking for espeak-ng…" }
                                    },
                                    Some((false, _, _)) => rsx! {
                                        div {
                                            p { style: "font-size:0.8rem; color:#9ca3af; line-height:1.55;", "{engine_hint}" }
                                            if let Some(cmd) = engine_cmd {
                                                code { style: "display:inline-block; margin-top:0.35rem; padding:0.15rem 0.45rem; background:#111827; border-radius:0.3rem; font-size:0.8rem; color:#e5e7eb;",
                                                    "{cmd}"
                                                }
                                            }
                                            div {
                                                button {
                                                    style: "{recheck_btn}",
                                                    onclick: move |_| recheck.with_mut(|n| *n += 1),
                                                    "Re-check"
                                                }
                                            }
                                        }
                                    },
                                    Some((true, true, using_private)) => rsx! {
                                        div { style: "font-size:0.8rem; color:#86efac;",
                                            if using_private {
                                                "Ready — using installed voices."
                                            } else {
                                                "Ready — using system voices."
                                            }
                                        }
                                    },
                                    Some((true, false, _)) => rsx! {
                                        div {
                                            p { style: "font-size:0.8rem; color:#fca5a5; line-height:1.55;",
                                                "espeak-ng is installed, but the {lang_name} voice data wasn't found. Re-check after installing or bundling the voices:"
                                            }
                                            div {
                                                button {
                                                    style: "{recheck_btn}",
                                                    onclick: move |_| recheck.with_mut(|n| *n += 1),
                                                    "Re-check"
                                                }
                                            }
                                        }
                                    },
                                }
                            }
                        }
                    } else {
                        div { style: "font-size:0.78rem; color:#9ca3af; line-height:1.5; margin-top:0.5rem;",
                            "Off. Turn this on to hear whole words spoken aloud — it uses the free, offline espeak-ng engine."
                        }
                    }
                }
            }
        }
    }
}
