use bevy::prelude::*;
use crate::state::AppState;

pub struct ModelRecordingPlugin;

impl Plugin for ModelRecordingPlugin {
    fn build(&self, app: &mut App) {
        use AppState::ModelRecording as Screen;
        app.add_systems(OnEnter(Screen), enter)
            .add_systems(OnExit(Screen), exit);
    }
}

#[derive(Component)]
struct ModelRecordingEntity;

fn enter(mut commands: Commands) {
    info!("entering ModelRecording");
    commands.spawn((
        Text::new("Model Recording\n\n(coming soon)"),
        ModelRecordingEntity,
    ));
}

fn exit(
    mut commands: Commands,
    query: Query<Entity, With<ModelRecordingEntity>>,
) {
    info!("exiting ModelRecording");
    for entity in &query {
        commands.entity(entity).despawn();
    }
}