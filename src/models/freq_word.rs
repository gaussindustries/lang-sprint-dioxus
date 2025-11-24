use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct FrequencyWord {
    pub rank: u32,

    /// Word *in the target language*
    pub word: String,

    /// Translation (default to English)
    pub en: String,

    #[serde(default)]
    pub pos: Option<String>,

    #[serde(default)]
    pub example: Option<String>,
}
