use bevy::prelude::*;

use crate::features::keyboard::KeyTone;
use crate::features::pipeline::DetectedPitches;
use crate::features::{
    layout::PianoLayout,
};
use crate::settings::data::AppSettings;


#[derive(Component)]
pub struct NoteBlock;

pub fn spawn_note_blocks(
    detected: Res<DetectedPitches>,
    layout: Res<PianoLayout>,
    keys: Query<(&KeyTone, &Transform, &Sprite)>, // TODO: is this rly a good way to query? why not marker components?
    mut commands: Commands,
) {
    for tone in &detected.notes {
        let Some((_, transform, sprite)) = keys
            .iter()
            .find(|(k, _, _)| k.0 == *tone)
        else {
            continue;
        };

        let key_x = transform.translation.x;
        let key_width = sprite.custom_size.map(|v| v.x).unwrap_or(layout.white_key_width);
        let note_height = 20.0;
        let start_y = layout.bottom_y + layout.white_key_height;

        commands.spawn((
            Sprite {
                color: Color::srgb(0.3, 0.7, 1.0),
                custom_size: Some(Vec2::new(key_width, note_height)),
                ..default()
            },
            Transform::from_xyz(key_x, start_y, 0.5),
            NoteBlock
        ));
    }
}

pub fn move_note_blocks(
    time: Res<Time>,
    settings: Res<AppSettings>,
    mut query: Query<&mut Transform, With<NoteBlock>>,
) {
    for mut transform in &mut query {
        transform.translation.y += settings.note_speed * time.delta_secs();
    }
}

pub fn despawn_offscreen_note_blocks(
    windows: Query<&Window>,
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<NoteBlock>>,
) {
    let window = windows.single().expect("expected window");
    for (entity, transform) in &query {
        if transform.translation.y > window.height() / 2.0 + 50.0 {
            commands.entity(entity).despawn();
        }
    }
}