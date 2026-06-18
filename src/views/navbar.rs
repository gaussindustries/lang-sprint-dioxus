use crate::components::DictSearch;
use crate::Route;
use dioxus::prelude::*;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

#[component]
pub fn Navbar() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }
        div {
            id: "navbar",
            style: "display:flex; justify-content: center; align-items:center; gap:1.25rem;",
            Link { to: Route::Home {}, "Home" }
            Link { to: Route::DictionaryPage {}, "Dictionary" }
            div {
                DictSearch {}
            }
        }
        Outlet::<Route> {}
    }
}
