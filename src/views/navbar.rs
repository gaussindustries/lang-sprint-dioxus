use crate::components::{
    select::{
        Select, SelectGroup, SelectGroupLabel, SelectItemIndicator, SelectList, SelectOption,
        SelectTrigger, SelectValue,
    },
    DictSearch,
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

    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }
        div {
            id: "navbar",
            style: "display:flex; justify-content:center; align-items:center; gap:1.25rem;",

            Link { to: Route::Home {}, "Home" }
            Link { to: Route::DashboardPage {}, "Dashboard" }
            Link { to: Route::ReadingPage {}, "Reading" }
            Link { to: Route::DictionaryPage {}, "Dictionary" }

            div { class: "text-black",
                Select::<Languages> {
                    placeholder: "Select a Language...",
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

            // right-aligned group: language switcher + search
            div {
                style: "display:flex; align-items:center; gap:0.75rem;",



                DictSearch {}
            }
        }
        Outlet::<Route> {}
    }
}
