// src/views/home.rs
use dioxus::prelude::*;
use std::fs;
use crate::{components::{TypingTest, Keyboard, separator::Separator}, models::letter::Letter, views::Alphabet};

pub fn Home() -> Element {
    let lang = use_signal(|| "georgian".to_string());

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

            header { class: "bg-indigo-600 text-center p-3",
                h1 { class: "text-3xl font-bold", "LangSprint – ქართული" }
            }
			div { class:"shadow-inner",
            	Alphabet { letters: letters_vec.clone(), lang: lang.clone() }
			}
			div{class:"flex justify-center",
				div{ class:"w-11/12 ", 
					Separator{
						horizontal:true
					}
				}
			}
            section { class: "flex justify-center",
					// ⬇️ Keyboard wraps TypingTest so it captures key events
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