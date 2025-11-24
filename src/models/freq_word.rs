use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct FrequencyWord {
    pub rank: u32,
    pub xx: String,   // target language word (Georgian spelling)
    pub en: String,   // English gloss

    #[serde(default)]
    pub cb: Option<String>,   // transliteration (optional but useful)

    #[serde(default)]
    pub pos: Option<String>,  // part of speech

    #[serde(default)]
    pub example: Option<String>, // example sentence
}