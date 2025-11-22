use dioxus::{events::KeyboardEvent, html::div};
use dioxus::prelude::*;
use std::collections::{HashMap,HashSet};

use crate::{components::{WordCard, TypingTest}, models::letter::Letter};


// QWERTY rows: physical keys
const ROW1: [&str; 10] = ["KeyQ","KeyW","KeyE","KeyR","KeyT","KeyY","KeyU","KeyI","KeyO","KeyP"];
const ROW2: [&str; 9]  = ["KeyA","KeyS","KeyD","KeyF","KeyG","KeyH","KeyJ","KeyK","KeyL"];
const ROW3: [&str; 9]  = ["KeyLShift","KeyZ","KeyX","KeyC","KeyV","KeyB","KeyN","KeyM", "KeyRShift"];

#[derive(Clone,PartialEq)]
struct KeySlot {
    key_code: String,
    qwerty_label: String,
    base: Option<Letter>,    // shifted == false
    shifted: Option<Letter>, // shifted == true
}

#[component]
pub fn Keyboard(letters: Vec<Letter>) -> Element {
    // track currently pressed: (key_code, shift_down)
    let mut pressed = use_signal(|| HashSet::<String>::new());

    // Build lookup: (key_code, shifted) -> Letter
    let mut map: HashMap<(String, bool), Letter> = HashMap::new();

	for l in letters.into_iter() {
        map.insert((l.key_code.clone(), l.shifted), l);
    }

	let space_pressed = pressed().contains("Space");

	let space_classes = if space_pressed {
    "px-20 py-3 bg-blue-500 text-white rounded text-2xl font-bold \
     transition ring-4 ring-white shadow-lg"
	} else {
		"px-20 py-3 bg-gray-300 rounded text-2xl font-bold text-gray-700 \
		hover:bg-gray-400 transition ring-1"
	};

    let make_row = |row_codes: &[&str]| -> Vec<KeySlot> {
        row_codes
            .iter()
            .map(|code| {
                let base = map
                    .get(&((*code).to_string(), false))
                    .cloned();
                let shifted = map
                    .get(&((*code).to_string(), true))
                    .cloned();

                KeySlot {
                    key_code: (*code).to_string(),
                    qwerty_label: code_to_qwerty_label(code).to_string(),
                    base,
                    shifted,
                }
            })
            .collect()
    };

    let row1 = make_row(&ROW1);
    let row2 = make_row(&ROW2);
    let row3 = make_row(&ROW3);
	
	let lang_signal = use_signal(|| "georgian".to_string());

    rsx! {
        // outer "global" listener: focusable container
		
        div {
            class: "keyboard p-4 bg-gray-800 rounded-lg shadow-inner select-none",
            tabindex: "0",

            onkeydown: move |evt: KeyboardEvent| {
				let code = evt.code().to_string();

				pressed.with_mut(|set| {
					set.insert(code);
				});
			},

            onkeyup: move |evt: KeyboardEvent| {
				let code = evt.code().to_string();

				pressed.with_mut(|set| {
					set.remove(&code);
				});
			},
			// ── Typing test using frequency list ───────────────
			// (Pass `lang` down from Home instead if you want to respect language choice)
			section { class: "p-6 flex justify-center",
				div {
					h2 { class: "text-2xl font-semibold mb-4 text-center", "Typing Test" }
					// if you keep `lang` up in Home, pass it in as a prop.
					// For now, hardcode Georgian:
					TypingTest { lang: lang_signal }
				}
			}


            // Row 1
            div { class: "flex justify-center gap-1 mb-1",
                {row1.iter().map(|slot| rsx! {
                    KeySlotView { slot: slot.clone(), pressed: pressed() }
                })}
            }

            // Row 2
            div { class: "flex justify-center gap-1 mb-1",
                {row2.iter().map(|slot| rsx! {
                    KeySlotView { slot: slot.clone(), pressed: pressed() }
                })}
            }

            // Row 3
            div { class: "flex justify-center gap-1 mb-1",
                {row3.iter().map(|slot| rsx! {
                    KeySlotView { slot: slot.clone(), pressed: pressed() }
                })}
            }

            div { class: "flex justify-center mt-1",
				div {
					id: "space",
					class: "{space_classes}",
					"Space"
				}
			}
			div { class:"flex justify-center white",
				div { 
					div { class:"text-center", 
						"Legend"
					}
					div{ class:"flex",
						section{h5{class:"text-center", "Left"}
							div{ class:"flex gap-2 place-items-center",
								"Ring =>" div{class:"ring_left h-[15px] w-[15px] rounded",}
							}
							div{ class:"flex gap-2 place-items-center",
								"Middle =>" div{class:"middle_left h-[15px] w-[15px] rounded",}
							}
							div{ class:"flex gap-2 place-items-center",
								"Index =>" div{class:"index_left h-[15px] w-[15px] rounded",}
							}
						}
						
						div { class:"flex h-[50px] w-[2px] rounded bg-black my-1 mx-2" }

						section{h5{class:"text-center", "Right"}
							div{ class:"flex gap-2 place-items-center",
								"Ring =>" div{class:"ring_right h-[15px] w-[15px] rounded",}
							}
							div{ class:"flex gap-2 place-items-center",
								"Middle =>" div{class:"middle_right h-[15px] w-[15px] rounded",}
							}
							div{ class:"flex gap-2 place-items-center",
								"Index =>" div{class:"index_right h-[15px] w-[15px] rounded",}
							}
						}
					}
					div{ class:"flex gap-2 place-items-center justify-center",
						"Little Finger =>" div{class:"little_left h-[15px] w-[15px] rounded",}
					}
				}
			}
        }
    }
}

// Render one physical key (slot), handling base/shifted + pressed state
#[component]
fn KeySlotView(slot: KeySlot, pressed: HashSet<String>) -> Element {

    let is_pressed = pressed.contains(&slot.key_code);

	let shift_down =
		pressed.contains("ShiftLeft") || pressed.contains("ShiftRight");


    // Which letter do we *show*? If Shift is down and we have a shifted letter, use that.
    let active_letter = if shift_down && slot.shifted.is_some() {
        slot.shifted.as_ref()
    } else {
        slot.base.as_ref()
    };

    // No mapping at all for this key → gray placeholder with QWERTY label
    if active_letter.is_none() && slot.base.is_none() && slot.shifted.is_none() {
        return rsx! {
            div {
                class: "key w-12 h-12 flex items-center justify-center rounded \
                        bg-gray-700 text-gray-500 opacity-50 text-sm",
                "{slot.qwerty_label}"
            }
        };
    }

    // Fallback: if no active letter (e.g. only shift exists but shift not pressed),
    // still show whatever we have so the key isn't "empty".
    let letter = active_letter
        .or(slot.base.as_ref())
        .or(slot.shifted.as_ref())
        .unwrap();

    let finger_class = match letter.finger.as_str() {
        "index_left"   => "index_left",
        "index_right"  => "index_right",
        "middle_left"  => "middle_left",
        "middle_right" => "middle_right",
        "ring_left"    => "ring_left",
        "ring_right"   => "ring_right",
        "little_left" | "little_right" => "little_left",
        _ => "",
    };

    let classes = if is_pressed {
        format!("ring-4 ring-white shadow-lg {finger_class} text-white bg-blue-500")
    } else {
        format!("bg-white hover:bg-gray-200 shadow {finger_class} text-gray-800")
    };

    rsx! {
        div {
            class: format!(
                "key w-12 h-12 flex flex-col items-center justify-center rounded \
                 font-bold transition {}",
                classes
            ),
            // Main glyph: Georgian (or whatever)
            span { class: "text-xl leading-none", "{letter.letter}" }
            // Small QWERTY hint below
            span { class: "text-[0.6rem] text-gray-500 mt-1", "{slot.qwerty_label}" }
        }
    }
}

// QWERTY label for physical key codes
fn code_to_qwerty_label(code: &str) -> &'static str {
    match code {
        "KeyQ" => "Q",
        "KeyW" => "W",
        "KeyE" => "E",
        "KeyR" => "R",
        "KeyT" => "T",
        "KeyY" => "Y",
        "KeyU" => "U",
        "KeyI" => "I",
        "KeyO" => "O",
        "KeyP" => "P",
        "KeyA" => "A",
        "KeyS" => "S",
        "KeyD" => "D",
        "KeyF" => "F",
        "KeyG" => "G",
        "KeyH" => "H",
        "KeyJ" => "J",
        "KeyK" => "K",
        "KeyL" => "L",
        "KeyZ" => "Z",
        "KeyX" => "X",
        "KeyC" => "C",
        "KeyV" => "V",
        "KeyB" => "B",
        "KeyN" => "N",
        "KeyM" => "M",
		"KeyLShift" => "Shift",
		"KeyRShift" => "Shift",
        _ => "",
    }
}
