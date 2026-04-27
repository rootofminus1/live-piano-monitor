use core::Tone;

use bevy::prelude::*;


const WHITE_KEY_HEIGHT_RATIO: f32 = 0.35;
const BLACK_KEY_HEIGHT_RATIO: f32 = 0.22;
const KEY_GAP: f32 = 2.0;


#[derive(Resource, Clone)]
pub struct PianoLayout {
    pub white_key_width: f32,
    pub white_key_height: f32,
    pub black_key_width: f32,
    pub black_key_height: f32,
    pub start_x: f32,
    pub bottom_y: f32,
}


pub fn calculate_layout(window: &Window, keys: &[Tone]) -> PianoLayout {
    let white_key_count = keys.iter().filter(|k| k.note.is_white()).count();
    let usable_width = window.width();

    let white_key_width =
        (usable_width - KEY_GAP * (white_key_count as f32 - 1.0))
        / white_key_count as f32;

    let white_key_height = window.height() * WHITE_KEY_HEIGHT_RATIO;
    let black_key_width = white_key_width * 0.6;
    let black_key_height = window.height() * BLACK_KEY_HEIGHT_RATIO;

    let total_width =
        white_key_count as f32 * white_key_width +
        (white_key_count as f32 - 1.0) * KEY_GAP;

    let start_x = -window.width() / 2.0 + (window.width() - total_width) / 2.0 + white_key_width / 2.0;
    let bottom_y = -window.height() / 2.0;

    PianoLayout {
        white_key_width,
        white_key_height,
        black_key_width,
        black_key_height,
        start_x,
        bottom_y,
    }
}