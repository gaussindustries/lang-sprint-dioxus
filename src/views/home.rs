use dioxus::prelude::*;
use std::fs;
use strum::IntoEnumIterator;
use crate::assets::alphabet_json_for;

use crate::{
    // we don't actually USE these here, but keeping the import is fine
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
    let mut lang = use_signal(|| "georgian".to_string());
    let mut lang_sig = lang.clone();

    let mut load_error = use_signal(|| None::<String>);

    let languages = Languages::iter().enumerate().map(|(i, f)| {
        rsx! {
            SelectOption::<Languages> {
                index: i,
                value: f,
                text_value: format!("{f}"),
                { format!("{f} {}", f.emoji()) }
                SelectItemIndicator {}
            }
        }
    });

    let letters = {
        let mut load_error = load_error.clone();
        let lang = lang.clone();

        use_resource(move || {
            let lang_name = lang.read().clone();

            async move {
                let json = alphabet_json_for(&lang_name);

                match serde_json::from_str::<Vec<Letter>>(json) {
                    Ok(vec) => vec,
                    Err(e) => {
                        let msg = format!("Failed to parse alphabet for {lang_name}: {e}");
                        eprintln!("{msg}");
                        load_error.set(Some(msg));
                        Vec::new()
                    }
                }
            }
        })
    };


    let letters_vec  = letters.read().clone().unwrap_or_default();
    let current_lang = lang();

    rsx! {
        div { class: "flex flex-col min-h-screen bg-gray-800 text-white",
            header { class: "bg-indigo-600 text-center p-3 space-y-2 flex justify-center gap-6 items-center",
                h1 { class: "text-3xl font-bold", "LangSprint" }

                div { class: "text-black",
                    Select::<Languages> {
                        placeholder: "Select a Language...",

                        on_value_change: move |value: Option<Languages>| {
                            if let Some(lang_enum) = value {
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

            if let Some(err) = load_error() {
                div { class: "bg-red-900 text-red-200 px-4 py-2 text-sm text-center",
                    "{err}"
                }
            }

            div {//TODO FIGURE OUT HOW TO FADE IN AND OUT PROPERLY!!
                key: "{current_lang}",
                class: "fade-in-soft",

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
}
