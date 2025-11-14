use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Letter {
    pub letter: String,
    pub name:   String,
    pub pron:   String,
    pub audio:  Option<String>,
    pub finger: String,
    pub key_code: String,
	pub shifted:bool
}