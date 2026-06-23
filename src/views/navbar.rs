use crate::components::{
    select::{
        Select, SelectGroup, SelectGroupLabel, SelectItemIndicator, SelectList, SelectOption,
        SelectTrigger, SelectValue,
    },
    DictSearch, SettingsButton,
};
use crate::Route;
use dioxus::prelude::*;
use strum::IntoEnumIterator;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

#[derive(Debug, Clone, Copy, PartialEq, strum::EnumIter, strum::Display)]
enum Languages {
    Georgian,
    Russian,
}

impl Languages {
    const fn emoji(&self) -> &'static str {
        match self {
            Languages::Georgian => "🇬🇪",
            Languages::Russian => "🇷🇺",
        }
    }

    const fn code(&self) -> &'static str {
        match self {
            Languages::Georgian => "georgian",
            Languages::Russian => "russian",
        }
    }
}

/// Layout chrome rendered on every route: nav links, the language switcher
/// (now global), and the dictionary search. The active language lives in
/// context (provided by `App`), so switching here updates every page.
#[component]
pub fn Navbar() -> Element {
    let mut lang = use_context::<Signal<String>>();
    let route = use_route::<Route>();
    let lang_now = use_context::<Signal<String>>()();

    let languages = Languages::iter().enumerate().map(|(i, f)| {
        let label = format!("{} {f}", f.emoji()); // e.g. "🇬🇪 Georgian"
        rsx! {
            SelectOption::<Languages> {
                index: i,
                value: f,
                text_value: label.clone(),
                {label}
                SelectItemIndicator {}
            }
        }
    });
    // somewhere before the rsx
    let current = Languages::iter().find(|l| l.code() == lang_now);
    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }
        div {
            id: "navbar",
            style: "display:flex; justify-content:center; align-items:center; gap:1.25rem;",
            class: "fade-in-soft p-2",

            Link { to: Route::Home {}, "Dashboard" }
            Link { to: Route::AlphabetPage {  }, "Alphabet" }
            Link { to: Route::GrammarPage {}, "Grammar" }
            Link { to: Route::TypingPage {  }, "Typing Test" }
            Link { to: Route::ReadingPage {}, "Reading" }
            Link { to: Route::DictionaryPage {}, "Dictionary" }

            div {
                            style: "display:flex; align-items:center; gap:0.75rem;",
                            DictSearch {}

            }
            div { class: "text-black",
                Select::<Languages> {
                    placeholder: "Select a Language...",
                    value:current,
                    on_value_change: move |value: Option<Languages>| {
                        if let Some(lang_enum) = value {
                            lang.set(lang_enum.code().to_string());
                        }
                    },

                    SelectTrigger {
                        aria_label: "Select Trigger",
                        width: "11rem",
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
            SettingsButton {}


        }
        for fade_key in [format!("{route:?}-{lang_now}")] {
            div {
                key: "{fade_key}",
                class: "fade-in-soft",
                Outlet::<Route> {}
            }
        }
    }
}
