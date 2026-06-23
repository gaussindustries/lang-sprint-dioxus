use dioxus::prelude::*;

use crate::{components::separator::Separator, views::DashboardPage, Route};
/// Landing hub. The alphabet and typing test are now their own routed views;
/// Home is just the entry point that links into the sections. Slim on purpose —
/// flesh it out or retire it later.
#[component]
pub fn Home() -> Element {
    let tiles = [
        (
            "Alphabet",
            "Learn to pronounce the script, letter by letter",
            Route::AlphabetPage {},
        ),
        (
            "Typing test",
            "Drill muscle memory of the keyboard and the meaning of the lexicon. Or test your WPM!",
            Route::TypingPage {},
        ),
        (
            "Reading",
            "Read sentences and learn new words",
            Route::ReadingPage {},
        ),
        (
            "Dictionary",
            "Browse the 1000 most common words in the lexicon",
            Route::DictionaryPage {},
        ),
    ];

    rsx! {
        div { class: "flex flex-col min-h-screen bg-gray-800 text-white",
            header { class: "bg-indigo-600 text-center p-6",
                h1 { class: "text-3xl font-bold", "LangSprint" }
                p { class: "text-indigo-200 text-sm mt-1", "Comprehension is the only finish line." }
            }
            div { class: "max-w-3xl mx-auto w-full p-6 grid grid-cols-1 sm:grid-cols-2 gap-4",
                for (title, blurb, route) in tiles {
                    Link {
                        key: "{title}",
                        to: route,
                        class: "block rounded-full border border-gray-700 bg-gray-900/40 p-6 \
                                hover:border-indigo-400 transition-colors",
                        div { class: "text-lg font-semibold text-indigo-200", "{title}" }
                        div { class: "text-sm text-gray-400 mt-1", "{blurb}" }
                    }
                }
            }
            div { class: "flex justify-center",
                div { class: "w-11/12",
                    Separator { horizontal: true }
                }
            }
            section { class: "flex justify-center",
                DashboardPage{}
            }
        }
    }
}
