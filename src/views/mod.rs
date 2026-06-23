//! The views module contains the components for all Layouts and Routes for our app. Each layout and route in our [`Route`]
//! enum will render one of these components.
//!
//!
//! The [`Home`] and [`Blog`] components will be rendered when the current route is [`Route::Home`] or [`Route::Blog`] respectively.
//!
//!
//! The [`Navbar`] component will be rendered on all pages of our app since every page is under the layout. The layout defines
//! a common wrapper around all child routes.

mod home;
pub use home::Home;

mod navbar;
pub use navbar::Navbar;

mod alphabet;
pub use alphabet::AlphabetPage;

mod dictionary;
pub use dictionary::DictionaryPage;

mod dashboard;
pub use dashboard::DashboardPage;

mod reading;
pub use reading::ReadingPage;

mod typing_test;
pub use typing_test::TypingPage;
