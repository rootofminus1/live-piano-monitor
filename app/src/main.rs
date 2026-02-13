mod testing;

use bevy::{log::LogPlugin, prelude::*};
use bevy_egui::EguiPlugin;

mod core;
mod audio;
mod display;
mod ui;

use audio::AudioPlugin;
use display::DisplayPlugin;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin::default()))
        .add_plugins(EguiPlugin::default())
        .add_plugins(AudioPlugin)
        .add_plugins(DisplayPlugin)
        .add_plugins(UiPlugin)
        .run();
}



