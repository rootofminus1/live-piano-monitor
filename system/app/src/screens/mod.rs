mod live_listen;
mod main_menu;
mod model_recording;

use bevy::prelude::*;
use live_listen::LiveListenPlugin;
use main_menu::MainMenuPlugin;
use model_recording::ModelRecordingPlugin;

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MainMenuPlugin, LiveListenPlugin, ModelRecordingPlugin));
    }
}