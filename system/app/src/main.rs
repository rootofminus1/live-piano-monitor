use bevy::{log::LogPlugin, prelude::*};
use bevy_egui::EguiPlugin;

use crate::{features::{keyboard::KeyboardPlugin, pipeline::PipelinePlugin}, screens::ScreensPlugin, settings::plugin::SettingsPlugin, state::AppState, ui::UiPlugin};

mod state;
mod testing;
mod screens;
mod features;
mod settings;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin::default()))
        .add_plugins(EguiPlugin::default())
        .init_state::<AppState>()
        .add_plugins(SettingsPlugin)
        // lifecycle for features live HERE... should they tho?
        .add_plugins(PipelinePlugin)
        .add_plugins(KeyboardPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(ScreensPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}


fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
