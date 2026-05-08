use bevy::prelude::*;

use crate::features::{keyboard::KeyTone, pipeline::DetectedPitches};

pub fn highlight_keys(
    detected: Res<DetectedPitches>,
    mut keys: Query<(&KeyTone, &mut Sprite)>,
) {
    for (key, mut sprite) in &mut keys {
        sprite.color = if key.0.note.is_white() {
            Color::srgb(0.95, 0.95, 0.95)
        } else {
            Color::BLACK
        };

        for tone in &detected.notes {
            if key.0 == *tone {
                sprite.color = Color::srgb(1.0, 0.0, 0.0);
            }
        }
    }
}