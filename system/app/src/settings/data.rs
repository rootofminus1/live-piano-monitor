use alg::processor::DetectionMode;
use bevy::prelude::*;
use core::ModelInfo;
use serde::{Deserialize, Serialize};

pub const SETTINGS_PATH: &str = "settings.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyboardKind {
    Piano88,
    PianoSmall,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Resource)]
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
            models_dir: None,
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
            info!("using self {}", dir);
            return std::path::PathBuf::from(dir);
        }
        if let Ok(dir) = std::env::var("PIANO_MODELS_DIR") {
            info!("using env {}", dir);
            return std::path::PathBuf::from(dir);
        }
        info!("using fallback");
        std::path::PathBuf::from(".models")
    }
}