use bevy::prelude::*;
use crate::audio::PitchState;
use crate::core::freq_to_note;
use crate::display::KeyPitch;


pub fn highlight_keys(
    pitch: Res<PitchState>,
    mut keys: Query<(&KeyPitch, &mut Sprite)>
) {
    let maybe_note = pitch.current_hz.and_then(freq_to_note);

    for (key, mut sprite) in &mut keys {
        if key.note.is_white() {
            sprite.color = Color::srgb(0.95, 0.95, 0.95);
        } else {
            sprite.color = Color::BLACK;
        }

        if let Some((note, octave)) = maybe_note {
            if key.note == note && key.octave == octave {
                sprite.color = Color::srgb(1.0, 0.0, 0.0);
            }
        }
    }
}