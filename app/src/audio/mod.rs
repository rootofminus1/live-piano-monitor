pub mod capture;

pub use capture::{Backend, SR};

use std::sync::{atomic::AtomicBool, Arc};

use bevy::prelude::*;
use crossbeam::channel::{unbounded, Receiver};

use capture::start_capture;

pub struct AudioCapturePlugin;

impl Plugin for AudioCapturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, start_audio_capture);
    }
}

// TODO: move to a data plugin
#[derive(Resource)]
pub struct AudioConfig {
    pub device_index: usize,
    pub backend: Backend,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            device_index: 0,
            backend: Backend::PortAudio,
        }
    }
}


#[derive(Resource)]
pub struct RawBlockReceiver {
    pub rx: Receiver<Vec<f32>>,
    pub stop: Arc<AtomicBool>,
}

pub fn start_audio_capture(mut commands: Commands, config: Option<Res<AudioConfig>>) {
    let (device_index, backend) = match config {
        Some(c) => (c.device_index, c.backend),
        None => (0, Backend::PortAudio),
    };

    let (tx, rx) = unbounded();
    let stop = Arc::new(AtomicBool::new(false));

    start_capture(device_index, backend, tx, Arc::clone(&stop));

    commands.insert_resource(RawBlockReceiver { rx, stop });
}