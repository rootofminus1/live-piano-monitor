use bevy::prelude::*;
use crate::state::AppState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        use AppState::MainMenu as Screen;
        app.add_systems(OnEnter(Screen), enter)
            .add_systems(OnExit(Screen), exit);
    }
}

#[derive(Component)]
struct MainMenuEntity;

fn enter(mut commands: Commands) {
    info!("entering MainMenu");

    commands.spawn((
        Text::new("Live Piano Monitor\n\nNavigate to another screen"),
        MainMenuEntity,
    ));
}

fn exit(mut commands: Commands, query: Query<Entity, With<MainMenuEntity>>) {
    info!("exiting MainMenu");
    for entity in &query {
        commands.entity(entity).despawn();
    }
}