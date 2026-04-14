use bevy::prelude::*;
use crate::{alg::PolyphonicState, core::Pitch, display::{DetectedNotes, KeyPitch}};


pub fn highlight_keys(
    detected: Res<PolyphonicState>,
    mut keys: Query<(&KeyPitch, &mut Sprite)>
) {
    for (key, mut sprite) in &mut keys {
        if key.note.is_white() {
            sprite.color = Color::srgb(0.95, 0.95, 0.95);
        } else {
            sprite.color = Color::BLACK;
        }

        for pitch in &detected.notes {
            let Pitch { note, octave } = pitch;

            if key.note == *note && key.octave == *octave {
                sprite.color = Color::srgb(1.0, 0.0, 0.0);
            }
        }
    }
}