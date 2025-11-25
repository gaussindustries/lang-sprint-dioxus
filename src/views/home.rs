// src/views/home.rs
use dioxus::prelude::*;
use std::fs;
use strum::IntoEnumIterator;

use crate::{
    components::{
        Keyboard,
        TypingTest,
        select::{
            Select, SelectTrigger, SelectValue, SelectList,
            SelectGroup, SelectGroupLabel, SelectOption, SelectItemIndicator,
        },
        separator::Separator,
    },
    models::letter::Letter,
    views::Alphabet,
};

#[derive(Debug, Clone, Copy, PartialEq, strum::EnumIter, strum::Display)]
enum Languages {
    Georgian,
    Russian,
}

impl Languages {
    const fn emoji(&self) -> &'static str {
        match self {
            Languages::Georgian => "🇬🇪",
            Languages::Russian  => "🇷🇺",
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Languages::Georgian => "georgian",
            Languages::Russian  => "russian",
        }
    }
}

#[component]
pub fn Home() -> Element {
    // Single source of truth for the language in the rest of the app.
    // This stays a String like before.
    let mut lang = use_signal(|| "georgian".to_string());

    // Clone for the closure
    let mut lang_sig = lang.clone();

    // Build the options list from the enum
    let languages = Languages::iter().enumerate().map(|(i, f)| {
        rsx! {
            SelectOption::<Languages> {
                index: i,
                value: f,
                text_value: format!("{f}"), // for typeahead / a11y
                { format!("{f} {}", f.emoji()) }
                SelectItemIndicator {}
            }
        }
    });

    // Load alphabet based on current lang (string)
    let letters = use_resource(move || {
        let cur_lang = lang.read().clone();
        async move {
            let path = format!("langs/{}/alphabet.json", cur_lang);
            let raw = fs::read_to_string(&path).unwrap_or_else(|_| {
                fs::read_to_string("langs/georgian/alphabet.json").unwrap_or_default()
            });
            serde_json::from_str::<Vec<Letter>>(&raw).unwrap_or_default()
        }
    });

    let letters_vec = letters.read().clone().unwrap_or_default();

    rsx! {
        div { class: "flex flex-col min-h-screen bg-gray-800 text-white",

            header { class: "bg-indigo-600 text-center p-3 space-y-2 flex justify-center gap-6 items-center",
                h1 { class: "text-3xl font-bold", "LangSprint" }

                // Language select
                div { class: "text-black",
                    Select::<Languages> {
                        placeholder: "Select a Language...",

                        // We do NOT pass `value:` here → uncontrolled Select.
                        // We only react to changes.

                        on_value_change: move |value: Option<Languages>| {
                            if let Some(lang_enum) = value {
                                // Map enum → folder string
                                lang_sig.set(lang_enum.code().to_string());
                            }
                        },

                        SelectTrigger {
                            aria_label: "Select Trigger",
                            width: "12rem",
                            SelectValue {}
                        }

                        SelectList { aria_label: "Select Languages",
                            SelectGroup {
                                SelectGroupLabel { "Languages" }
                                {languages}
                            }
                        }
                    }
                }
            }

            div { class: "shadow-inner",
                Alphabet { letters: letters_vec.clone(), lang: lang.clone() }
            }

            div { class:"flex justify-center",
                div { class:"w-11/12",
                    Separator { horizontal: true }
                }
            }

            section { class: "flex justify-center",
                div { class: "mt-auto p-4 w-full shadow-xs",
                    h2 { class: "text-2xl font-semibold text-center", "Typing Test" }

                    Keyboard { letters: letters_vec.clone(),
                        TypingTest { lang: lang.clone(), letters_vec: letters_vec.clone() }
                    }
                }
            }
        }
    }
}
