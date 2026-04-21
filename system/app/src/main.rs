mod testing;

use alg::DetectionMode;
use bevy::{log::LogPlugin, prelude::*};
use bevy_egui::EguiPlugin;

mod audio;
mod display;
mod ui;
mod alg_plugin;
mod settings;

use display::DisplayPlugin;
use ui::UiPlugin;

use crate::{alg_plugin::DetectionPlugin, audio::AudioCapturePlugin, settings::plugin::SettingsPlugin};

fn main() {

    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin::default()))
        .add_plugins(EguiPlugin::default())
        
        .add_plugins(SettingsPlugin)
        .add_plugins(AudioCapturePlugin)
        .add_plugins(DetectionPlugin) 
        .add_plugins(DisplayPlugin)
        .add_plugins(UiPlugin)
        .run();
}



