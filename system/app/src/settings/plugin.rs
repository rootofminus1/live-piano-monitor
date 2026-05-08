use bevy::prelude::*;

use crate::settings::data::AppSettings;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(AppSettings::load())
            .add_systems(
                Last,
                save_settings.run_if(resource_changed::<AppSettings>),
            );
    }
}

fn save_settings(settings: Res<AppSettings>) {
    info!("saving settings");
    settings.save();
}