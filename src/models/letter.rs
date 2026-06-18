use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LetterKind {
    Vowel,
    Consonant,
    #[serde(other)]
    #[default]
    Other, // ъ/ь, or any letter with no `kind`
}

// ...in your existing Letter struct, add the one field:
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Letter {
    pub letter: String,
    pub name: String,
    pub pron: String,
    #[serde(default)]
    pub kind: LetterKind,
    pub audio: Option<String>,
    pub finger: String,
    pub key_code: String,
    pub shifted: bool,
}
