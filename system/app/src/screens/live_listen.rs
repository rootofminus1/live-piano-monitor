use bevy::prelude::*;

use crate::features::{
    highlight::highlight_keys, keyboard::{BuildKeyboard, DestroyKeyboard}, note_blocks::{
        despawn_offscreen_note_blocks, move_note_blocks, spawn_note_blocks,
    }, pipeline::{StartPipeline, StopPipeline}
};
use crate::state::AppState;

pub struct LiveListenPlugin;

impl Plugin for LiveListenPlugin {
    fn build(&self, app: &mut App) {
        use AppState::LiveListen as Screen;
        app.add_systems(OnEnter(Screen), enter)
        .add_systems(Update, (
            highlight_keys,
            spawn_note_blocks,
            move_note_blocks,
            despawn_offscreen_note_blocks,
        ).run_if(in_state(Screen)))
        .add_systems(OnExit(Screen), exit);
    }
}

fn enter(mut commands: Commands) {
    commands.trigger(StartPipeline);
    commands.trigger(BuildKeyboard);
}

fn exit(mut commands: Commands) {
    commands.trigger(StopPipeline);
    commands.trigger(DestroyKeyboard);
}