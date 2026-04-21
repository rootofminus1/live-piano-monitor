use bevy::prelude::*;
use crate::settings::data::AppSettings;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(AppSettings::load())
            .insert_resource(SettingsDirty(false))
            .add_systems(Last, save_if_dirty);
    }
}

#[derive(Resource)]
pub struct SettingsDirty(pub bool);

fn save_if_dirty(
    mut dirty: ResMut<SettingsDirty>,
    settings: Res<AppSettings>,
) {
    if dirty.0 {
        settings.save();
        dirty.0 = false;
    }
}