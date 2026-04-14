mod testing;

use bevy::{log::LogPlugin, prelude::*};
use bevy_egui::EguiPlugin;

mod core;
mod audio;
mod display;
mod ui;
mod alg;

use display::DisplayPlugin;
use ui::UiPlugin;

use crate::{alg::{DetectionPlugin, processor::DetectionMode}, audio::{AudioCapturePlugin, AudioConfig, Backend}};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin::default()))
        .add_plugins(EguiPlugin::default())
        
        .insert_resource(AudioConfig {
            device_index: 15,
            backend: Backend::PortAudio,
        })
        .insert_resource(DetectionMode::Polyphonic)
        // TODO: replace above resources with a data plugin
        .add_plugins(AudioCapturePlugin)
        .add_plugins(DetectionPlugin) 

        // .add_plugins(PolyphonicPlugin)
        .add_plugins(DisplayPlugin)
        .add_plugins(UiPlugin)
        .run();
}



