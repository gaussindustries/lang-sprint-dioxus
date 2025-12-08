//for the goys who have (((windows)))
//#![windows_subsystem = "windows"]
mod components;
use views::{Home, Navbar};
mod views;
mod calibration;
pub mod models;
pub mod audio;
pub mod assets;
use dioxus::prelude::*;

// ---------------------------------------------------------------------
// Assets – keep them exactly where you have them
// ---------------------------------------------------------------------
const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const DIOXUS_CSS: Asset = asset!("/assets/dx-components-theme.css");

// ---------------------------------------------------------------------
// Routes – one page now, easy to add more later
// ---------------------------------------------------------------------
#[derive(Routable, Debug, Clone, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
		#[route("/")]
		Home {},
		// Future tabs – just uncomment when you need them
		// #[route("/alphabet")]
		// Alphabet {},
		// #[route("/test")]
		// Test {},
		// #[route("/conjugate")]
		// Conjugate {},
		// #[route("/grammar")]
		// Grammar {},
}

// ---------------------------------------------------------------------
// Root App – inject assets + router
// ---------------------------------------------------------------------
fn main() {
    // The `launch` function is the main entry point for a dioxus app. It takes a component and renders it with the platform feature
    // you have enabled
    dioxus::launch(App);
}

/// App is the main component of our app. Components are the building blocks of dioxus apps. Each component is a function
/// that takes some props and returns an Element. In this case, App takes no props because it is the root of our app.
///
/// Components should be annotated with `#[component]` to support props, better error messages, and autocomplete
#[component]
fn App() -> Element {
    // The `rsx!` macro lets us define HTML inside of rust. It expands to an Element with all of our HTML inside.
	
	rsx! {
        // In addition to element and text (which we will see later), rsx can contain other components. In this case,
        // we are using the `document::Link` component to add a link to our favicon and main CSS file into the head of our app.
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: DIOXUS_CSS }
		document::Title{ "Lang Sprint V0.4.1"}
        // The router component renders the route enum we defined above. It will handle synchronization of the URL and render
        // the layouts and components for the active route.
        Router::<Route> {}
    }
}