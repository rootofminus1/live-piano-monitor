use crate::{KeyboardSpec, Tone};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub filename: String,
    pub start_tone: Tone,
    pub key_count: usize,
}

impl ModelInfo {
    pub fn summary(&self) -> String {
        format!("{} ({} notes from {})", self.name, self.key_count, self.start_tone)
    }

    pub fn to_keyboard_spec(&self) -> KeyboardSpec {
        KeyboardSpec {
            start_tone: self.start_tone,
            key_count: self.key_count,
        }
    }
}