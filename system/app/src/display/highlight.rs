use bevy::prelude::*;
use core::Tone;
use crate::{alg_plugin::DetectedPitches, display::KeyPitch};

pub fn highlight_keys(
    detected: Res<DetectedPitches>,
    mut keys: Query<(&KeyPitch, &mut Sprite)>,
) {
    for (key, mut sprite) in &mut keys {
        sprite.color = if key.note.is_white() {
            Color::srgb(0.95, 0.95, 0.95)
        } else {
            Color::BLACK
        };

        for Tone { note, octave } in &detected.notes {
            if key.note == *note && key.octave == *octave {
                sprite.color = Color::srgb(1.0, 0.0, 0.0);
            }
        }
    }
}