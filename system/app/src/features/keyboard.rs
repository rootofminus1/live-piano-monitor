use bevy::prelude::*;
use core::{KeyboardSpec, Tone, generate_keys};

use crate::features::note_blocks::NoteBlock;
use crate::settings::data::{AppSettings, KeyboardKind};
use crate::features::layout::{PianoLayout, calculate_layout};

// might be nice later? if we want to query only black or only white keys for some reason
// #[derive(Component)]
// pub enum KeyColor {
//     White,
//     Black,
// }

#[derive(Component, Debug, Clone, Copy)]
pub struct KeyTone(pub Tone);

#[derive(Component)]
pub struct PianoKeyEntity;


#[derive(Event)]
pub struct BuildKeyboard;
#[derive(Event)]
pub struct DestroyKeyboard;
#[derive(Event)]
pub struct RebuildKeyboard;

 
pub struct KeyboardPlugin;
 
impl Plugin for KeyboardPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(on_build_keyboard)
            .add_observer(on_destroy_keyboard)
            .add_observer(on_rebuild_keyboard);
    }
}

pub fn on_build_keyboard(
    _trigger: On<BuildKeyboard>,
    mut commands: Commands,
    windows: Query<&Window>,
    settings: Res<AppSettings>,
) {
    build(&mut commands, &windows, &settings);
}

pub fn on_destroy_keyboard(
    _trigger: On<DestroyKeyboard>,
    mut commands: Commands,
    key_query: Query<Entity, With<PianoKeyEntity>>,
    note_query: Query<Entity, With<NoteBlock>>,
) {
    destroy(&mut commands, &key_query, &note_query);
}

fn on_rebuild_keyboard(
    _trigger: On<RebuildKeyboard>,
    mut commands: Commands,
    key_query: Query<Entity, With<PianoKeyEntity>>,
    note_query: Query<Entity, With<NoteBlock>>,
    windows: Query<&Window>,
    settings: Res<AppSettings>,
) {
    info!("rebuilding keyboard");
    destroy(&mut commands, &key_query, &note_query);
    build(&mut commands, &windows, &settings);
}

pub fn build(
    commands: &mut Commands,
    windows: &Query<&Window>,
    settings: &Res<AppSettings>,
) {
    let spec = keyboard_spec(&settings);
    let keys = generate_keys(&spec);
    let window = windows.single().expect("expected one window");
    let layout = calculate_layout(window, &keys);

    commands.insert_resource(layout.clone());
    spawn_key_entities(commands, &keys, &layout);
}

pub fn destroy(
    commands: &mut Commands,
    key_query: &Query<Entity, With<PianoKeyEntity>>,
    note_query: &Query<Entity, With<NoteBlock>>,
) {
    for entity in key_query {
        commands.entity(entity).despawn();
    }
 
    for entity in note_query {
        commands.entity(entity).despawn();
    }
}


pub fn keyboard_spec(settings: &AppSettings) -> KeyboardSpec {
    match settings.keyboard_kind {
        KeyboardKind::Piano88 => KeyboardSpec::piano_88(),
        KeyboardKind::PianoSmall => KeyboardSpec::piano_smaller(),
    }
}

fn spawn_key_entities(commands: &mut Commands, tones: &[Tone], layout: &PianoLayout) {
    let mut next_white_x = layout.start_x;
    let mut prev_white_x: Option<f32> = None;

    for tone in tones {
        if tone.note.is_white() {
            let x = next_white_x;

            commands.spawn((
                Sprite {
                    color: Color::srgb(0.95, 0.95, 0.95),
                    custom_size: Some(Vec2::new(layout.white_key_width, layout.white_key_height)),
                    ..default()
                },
                Transform::from_xyz(x, layout.bottom_y + layout.white_key_height / 2.0, 1.0),
                KeyTone(tone.clone()),
                PianoKeyEntity,
            ));

            prev_white_x = Some(x);
            next_white_x += layout.white_key_width + 2.0;
        } else if let Some(prev_x) = prev_white_x {
            let next_x = next_white_x;
            let black_x = (prev_x + next_x) / 2.0;

            commands.spawn((
                Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(layout.black_key_width, layout.black_key_height)),
                    ..default()
                },
                Transform::from_xyz(
                    black_x,
                    layout.bottom_y + layout.white_key_height - layout.black_key_height / 2.0,
                    2.0,
                ),
                KeyTone(tone.clone()), // TODO: deref better? maybe?
                PianoKeyEntity,
            ));
        }
    }
}