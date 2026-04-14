use bevy::prelude::*;
use crate::{core::{KeyboardSpec, generate_keys}, ui::{KeyboardKind, UiSettings}};

mod layout;
mod keyboard;
mod note_blocks;
mod highlight;

pub use layout::*;
pub use keyboard::*;
pub use note_blocks::*;
pub use highlight::*;

pub struct DisplayPlugin;


impl Plugin for DisplayPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(Update, (
                highlight_keys,
                spawn_note_block,
                move_note_blocks,
                despawn_offscreen_notes,
                rebuild_keyboard_system
            ));
    }
}

fn setup(
    mut commands: Commands,
    windows: Query<&Window>,
) {
    commands.spawn(Camera2d::default());

    let spec = KeyboardSpec::piano_88();
    let keys = generate_keys(&spec);

    // TODO: handle err case
    let window = windows.single().expect("expected one window");
    let layout = calculate_layout(window, &keys);

    commands.insert_resource(layout.clone());
    spawn_keyboard(&mut commands, &keys, &layout);
}


fn rebuild_keyboard_system(
    mut commands: Commands,
    windows: Query<&Window>,
    settings: Res<UiSettings>,
    mut old_settings: Local<Option<KeyboardKind>>,
    keys_query: Query<Entity, With<PianoKeyEntity>>,
    notes_query: Query<Entity, With<NoteBlock>>,
    mut layout_res: ResMut<PianoLayout>,
) {
    if Some(settings.keyboard_kind) == *old_settings {
        return;
    }

    *old_settings = Some(settings.keyboard_kind);

    for entity in &keys_query {
        commands.entity(entity).despawn();
    }

    for entity in &notes_query {
        commands.entity(entity).despawn();
    }

    let spec = match settings.keyboard_kind {
        KeyboardKind::Piano88 => KeyboardSpec::piano_88(),
        KeyboardKind::PianoSmall => KeyboardSpec::piano_smaller(),
    };

    let keys = generate_keys(&spec);

    let window = windows.single().expect("expected one window");
    let layout = calculate_layout(window, &keys);

    *layout_res = layout.clone();

    spawn_keyboard(&mut commands, &keys, &layout);
}
