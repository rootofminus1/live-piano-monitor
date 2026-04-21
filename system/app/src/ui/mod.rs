use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

mod menu;
pub use menu::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(MenuState::default())
            .add_systems(EguiPrimaryContextPass, ui_menu_system);
    }
}