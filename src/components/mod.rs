//! The components module contains all shared components for our app. Components are the building blocks of dioxus apps.
//! They can be used to defined common UI elements like buttons, forms, and modals. In this template, we define a Hero
//! component  to be used in our app.

pub mod wordcard;
pub mod keyboard;
pub mod typing_test;

pub use keyboard::Keyboard;
pub use wordcard::WordCard;
pub use typing_test::TypingTest;pub mod slider;
pub mod tooltip;
pub mod separator;
