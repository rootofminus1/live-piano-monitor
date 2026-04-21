use alg::processor::DetectionMode;
use core::{KeyboardSpec, Note};
use serde::{Deserialize, Serialize};

pub const SETTINGS_PATH: &str = "settings.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyboardKind {
    Piano88,
    PianoSmall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub filename: String,
    pub start_note: Note,
    pub start_octave: i32,
    pub key_count: usize,
}

impl ModelInfo {
    pub fn summary(&self) -> String {
        format!("{} ({} notes from {}{})",
            self.name, self.key_count, self.start_note, self.start_octave)
    }

    pub fn to_keyboard_spec(&self) -> KeyboardSpec {
        KeyboardSpec {
            start_note: self.start_note,
            start_octave: self.start_octave,
            key_count: self.key_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, bevy::prelude::Resource)]
pub struct AppSettings {
    pub note_speed: f32,
    pub keyboard_kind: KeyboardKind,
    pub device_name: Option<String>, 
    pub detection_mode: DetectionMode,
    pub models: Vec<ModelInfo>,
    pub active_model_index: Option<usize>,
    pub models_dir: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            note_speed: 100.0,
            keyboard_kind: KeyboardKind::Piano88,
            device_name: None,
            detection_mode: DetectionMode::Polyphonic,
            models: vec![],
            active_model_index: None,
            models_dir: None
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        std::fs::read_to_string(SETTINGS_PATH)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(SETTINGS_PATH, json);
        }
    }

    pub fn resolved_models_dir(&self) -> std::path::PathBuf {
        if let Some(dir) = &self.models_dir {
            return std::path::PathBuf::from(dir);
        }
        if let Ok(dir) = std::env::var("PIANO_MODELS_DIR") {
            return std::path::PathBuf::from(dir);
        }
        std::path::PathBuf::from("models")
    }
}