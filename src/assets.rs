// src/assets.rs
use dioxus::prelude::*;

// If you still want CSS/images handled by asset!, keep those here...
// pub const FAVICON: Asset = asset!("/assets/favicon.ico");
// etc.

// ---- JSON EMBEDDED AT COMPILE TIME ----

// Alphabets
pub const GEORGIAN_ALPHABET_JSON: &str = include_str!("../assets/langs/georgian/alphabet.json");
pub const RUSSIAN_ALPHABET_JSON:  &str = include_str!("../assets/langs/russian/alphabet.json");

// Frequency lists
pub const GEORGIAN_1000_JSON: &str = include_str!("../assets/langs/georgian/1000.json");
pub const RUSSIAN_1000_JSON:  &str = include_str!("../assets/langs/russian/1000.json");

// Helpers for language → JSON
pub fn alphabet_json_for(lang: &str) -> &'static str {
    match lang {
        "russian"  => RUSSIAN_ALPHABET_JSON,
        "georgian" => GEORGIAN_ALPHABET_JSON,
        _          => GEORGIAN_ALPHABET_JSON,
    }
}

pub fn freq_json_for(lang: &str) -> &'static str {
    match lang {
        "russian"  => RUSSIAN_1000_JSON,
        "georgian" => GEORGIAN_1000_JSON,
        _          => GEORGIAN_1000_JSON,
    }
}

// ── AUDIO: embed WAV files as bytes ───────────────────────────────────

// Georgian alphabet audio
pub const GEO_A: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/a.wav");
pub const GEO_B: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/b.wav");
pub const GEO_CH_COMMA: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/ch'.wav");
pub const GEO_CH: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/ch.wav");
pub const GEO_D: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/d.wav");
pub const GEO_DZ: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/dz.wav");
pub const GEO_E: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/e.wav");
pub const GEO_G: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/g.wav");
pub const GEO_GH: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/gh.wav");
pub const GEO_H: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/h.wav");
pub const GEO_I: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/i.wav");
pub const GEO_J: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/j.wav");
pub const GEO_K: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/k.wav");
pub const GEO_KH_COMMA: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/kh'.wav");
pub const GEO_KH: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/kh.wav");
pub const GEO_L: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/l.wav");
pub const GEO_M: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/m.wav");
pub const GEO_N: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/n.wav");
pub const GEO_O: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/o.wav");
pub const GEO_P: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/p.wav");
pub const GEO_PH: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/ph.wav");
pub const GEO_Q: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/q.wav");
pub const GEO_R: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/r.wav");
pub const GEO_S: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/s.wav");
pub const GEO_SH: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/sh.wav");
pub const GEO_T_COMMA: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/t'.wav");
pub const GEO_T: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/t.wav");
pub const GEO_TS_COMMA: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/ts'.wav");
pub const GEO_TS: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/ts.wav");
pub const GEO_U: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/u.wav");
pub const GEO_V: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/v.wav");
pub const GEO_Z: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/z.wav");
pub const GEO_ZH: &[u8] = include_bytes!("../assets/langs/georgian/pronunciation/alphabet/zh.wav");

// Russian alphabet audio
pub const RU_A: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/a.wav");
pub const RU_B: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/b.wav");
pub const RU_CH: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/ch.wav");
pub const RU_D: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/d.wav");
pub const RU_E: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/e.wav");
pub const RU_F: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/f.wav");
pub const RU_G: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/g.wav");
pub const RU_HARD: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/hard.wav");
pub const RU_I: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/i.wav");
pub const RU_K: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/k.wav");
pub const RU_KH: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/kh.wav");
pub const RU_L: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/l.wav");
pub const RU_M: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/m.wav");
pub const RU_N: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/n.wav");
pub const RU_O: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/o.wav");
pub const RU_P: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/p.wav");
pub const RU_R: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/r.wav");
pub const RU_S: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/s.wav");
pub const RU_SH: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/sh.wav");
pub const RU_SHCH: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/shch.wav");
pub const RU_SOFT: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/soft.wav");
pub const RU_T: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/t.wav");
pub const RU_TS: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/ts.wav");
pub const RU_U: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/u.wav");
pub const RU_V: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/v.wav");
pub const RU_Y: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/y.wav");
pub const RU_YA: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/ya.wav");
pub const RU_YE: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/ye.wav");
pub const RU_YO: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/yo.wav");
pub const RU_YU: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/yu.wav");
pub const RU_YY: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/yy.wav");
pub const RU_Z: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/z.wav");
pub const RU_ZH: &[u8] = include_bytes!("../assets/langs/russian/pronunciation/alphabet/zh.wav");

/// Map (lang, filename from JSON) → audio bytes
pub fn letter_audio_bytes(lang: &str, filename: &str) -> Option<&'static [u8]> {
    match (lang, filename) {
        // Georgian
		("georgian", "a.wav") => Some(GEO_A),
        ("georgian", "b.wav") => Some(GEO_B),
        ("georgian", "ch'.wav") => Some(GEO_CH_COMMA),
        ("georgian", "ch.wav") => Some(GEO_CH),
        ("georgian", "d.wav") => Some(GEO_D),
        ("georgian", "dz.wav") => Some(GEO_DZ),
        ("georgian", "e.wav") => Some(GEO_E),
        ("georgian", "g.wav") => Some(GEO_G),
        ("georgian", "gh.wav") => Some(GEO_GH),
        ("georgian", "h.wav") => Some(GEO_H),
        ("georgian", "i.wav") => Some(GEO_I),
        ("georgian", "j.wav") => Some(GEO_J),
        ("georgian", "k.wav") => Some(GEO_K),
        ("georgian", "kh'.wav") => Some(GEO_KH_COMMA),
        ("georgian", "kh.wav") => Some(GEO_KH),
        ("georgian", "l.wav") => Some(GEO_L),
        ("georgian", "m.wav") => Some(GEO_M),
        ("georgian", "n.wav") => Some(GEO_N),
        ("georgian", "o.wav") => Some(GEO_O),
        ("georgian", "p.wav") => Some(GEO_P),
        ("georgian", "ph.wav") => Some(GEO_PH),
        ("georgian", "q.wav") => Some(GEO_Q),
        ("georgian", "r.wav") => Some(GEO_R),
        ("georgian", "s.wav") => Some(GEO_S),
        ("georgian", "sh.wav") => Some(GEO_SH),
        ("georgian", "t'.wav") => Some(GEO_T_COMMA),
        ("georgian", "t.wav") => Some(GEO_T),
        ("georgian", "ts'.wav") => Some(GEO_TS_COMMA),
        ("georgian", "ts.wav") => Some(GEO_TS),
        ("georgian", "u.wav") => Some(GEO_U),
        ("georgian", "v.wav") => Some(GEO_V),
        ("georgian", "z.wav") => Some(GEO_Z),
        ("georgian", "zh.wav") => Some(GEO_ZH),

        // Russian
                ("russian", "a.wav") => Some(RU_A),
        ("russian", "b.wav") => Some(RU_B),
        ("russian", "ch.wav") => Some(RU_CH),
        ("russian", "d.wav") => Some(RU_D),
        ("russian", "e.wav") => Some(RU_E),
        ("russian", "f.wav") => Some(RU_F),
        ("russian", "g.wav") => Some(RU_G),
        ("russian", "hard.wav") => Some(RU_HARD),
        ("russian", "i.wav") => Some(RU_I),
        ("russian", "k.wav") => Some(RU_K),
        ("russian", "kh.wav") => Some(RU_KH),
        ("russian", "l.wav") => Some(RU_L),
        ("russian", "m.wav") => Some(RU_M),
        ("russian", "n.wav") => Some(RU_N),
        ("russian", "o.wav") => Some(RU_O),
        ("russian", "p.wav") => Some(RU_P),
        ("russian", "r.wav") => Some(RU_R),
        ("russian", "s.wav") => Some(RU_S),
        ("russian", "sh.wav") => Some(RU_SH),
        ("russian", "shch.wav") => Some(RU_SHCH),
        ("russian", "soft.wav") => Some(RU_SOFT),
        ("russian", "t.wav") => Some(RU_T),
        ("russian", "ts.wav") => Some(RU_TS),
        ("russian", "u.wav") => Some(RU_U),
        ("russian", "v.wav") => Some(RU_V),
        ("russian", "y.wav") => Some(RU_Y),
        ("russian", "ya.wav") => Some(RU_YA),
        ("russian", "ye.wav") => Some(RU_YE),
        ("russian", "yo.wav") => Some(RU_YO),
        ("russian", "yu.wav") => Some(RU_YU),
        ("russian", "yy.wav") => Some(RU_YY),
        ("russian", "z.wav") => Some(RU_Z),
        ("russian", "zh.wav") => Some(RU_ZH),

        _ => None,
    }
}
