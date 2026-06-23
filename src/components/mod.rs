//! The components module contains all shared components for our app. Components are the building blocks of dioxus apps.
//! They can be used to defined common UI elements like buttons, forms, and modals. In this template, we define a Hero
//! component  to be used in our app.

pub mod alphabet;
pub mod dict_search;
pub mod dictionary;
pub mod keyboard;
pub mod meaning_test;
pub mod settings;
pub mod typing_test;
pub mod word_detail;
pub mod wordcard;
pub mod wpm_test;

pub use alphabet::Alphabet;
pub use dict_search::DictSearch;
pub use dictionary::Dictionary;
pub use keyboard::Keyboard;
pub use meaning_test::MeaningTest;
pub use settings::SettingsButton;
pub use typing_test::TypingTest;
pub use word_detail::WordDetail;
pub use wordcard::WordCard;

pub mod avatar;
pub mod button;
pub mod input;
pub mod radio_group;
pub mod select;
pub mod separator;
pub mod slider;
pub mod tabs;
pub mod toggle;
pub mod tooltip;

pub mod dialog;
pub mod drills;
