use bevy::prelude::*;
use bevy_egui::{EguiPrimaryContextPass, EguiPlugin};

mod menu;
pub use menu::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(UiState::default())
            .insert_resource(UiSettings::default())
            .add_systems(EguiPrimaryContextPass, ui_menu_system);
    }
}