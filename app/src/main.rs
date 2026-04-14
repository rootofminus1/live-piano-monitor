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

use crate::alg::PolyphonicPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin::default()))
        .add_plugins(EguiPlugin::default())
        // .add_plugins(AudioPlugin)  // wont work for now, only works with PolyphonicState
        .add_plugins(PolyphonicPlugin)
        .add_plugins(DisplayPlugin)
        .add_plugins(UiPlugin)
        .run();
}



