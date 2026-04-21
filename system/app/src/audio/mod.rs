pub mod capture;

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use bevy::prelude::*;
use crossbeam::channel::{unbounded, Receiver};

use capture::{start_capture, list_input_devices, DeviceInfo};
use crate::settings::data::AppSettings;

pub struct AudioCapturePlugin;

impl Plugin for AudioCapturePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, start_audio_capture)
            .add_systems(Update, restart_audio_on_device_change);
    }
}

#[derive(Resource)]
pub struct RawBlockReceiver {
    pub rx: Receiver<Vec<f32>>,
    pub stop: Arc<AtomicBool>,
}

#[derive(Resource)]
pub struct DeviceList(pub Vec<DeviceInfo>);

pub fn start_audio_capture(
    mut commands: Commands,
    settings: Res<AppSettings>,
) {
    let devices = list_input_devices();

    for d in &devices {
        info!("Input device [{}]: {}", d.index, d.name);
    }

    let index = settings.device_name.as_deref()
        .and_then(|name| devices.iter().find(|d| d.name == name))
        .map(|d| d.index)
        .unwrap_or(0);

    info!("Starting audio capture on device index {}", index);

    let (tx, rx) = unbounded();
    let stop = Arc::new(AtomicBool::new(false));
    start_capture(index, tx, Arc::clone(&stop));

    commands.insert_resource(RawBlockReceiver { rx, stop });
    commands.insert_resource(DeviceList(devices));
}

fn restart_audio_on_device_change(
    mut commands: Commands,
    settings: Res<AppSettings>,
    device_list: Res<DeviceList>,
    mut current_name: Local<Option<Option<String>>>,
    receiver: Option<ResMut<RawBlockReceiver>>,
) {
    if current_name.is_none() {
        *current_name = Some(settings.device_name.clone());
        return;
    }

    if current_name.as_ref().unwrap() == &settings.device_name {
        return;
    }

    info!("Audio device changed to {:?}, restarting capture", settings.device_name);

    if let Some(old) = receiver {
        old.stop.store(true, Ordering::Relaxed);
    }

    *current_name = Some(settings.device_name.clone());

    let index = settings.device_name.as_deref()
        .and_then(|name| device_list.0.iter().find(|d| d.name == name))
        .map(|d| d.index)
        .unwrap_or(0);

    let (tx, rx) = unbounded();
    let stop = Arc::new(AtomicBool::new(false));
    start_capture(index, tx, Arc::clone(&stop));

    commands.insert_resource(RawBlockReceiver { rx, stop });
}