use bevy::prelude::*;
use core::{KeyInfo, Note};
use crate::display::PianoLayout;


#[derive(Component)]
pub enum KeyColor {
    White,
    Black,
}


#[derive(Component)]
pub struct KeyPitch {
    pub note: Note,
    pub octave: i32,
}


#[derive(Component)]
pub struct PianoKeyEntity;



pub fn spawn_keyboard(
    commands: &mut Commands,
    keys: &[KeyInfo],
    layout: &PianoLayout,
) {
    let mut next_white_x = layout.start_x;
    let mut prev_white_x: Option<f32> = None;

    for key in keys {
        if key.note.is_white() {
            let x = next_white_x;

            commands.spawn((
                Sprite {
                    color: Color::srgb(0.95, 0.95, 0.95),
                    custom_size: Some(Vec2::new(
                        layout.white_key_width,
                        layout.white_key_height,
                    )),
                    ..default()
                },
                Transform::from_xyz(
                    x,
                    layout.bottom_y + layout.white_key_height / 2.0,
                    1.0,
                ),
                KeyColor::White,
                KeyPitch {
                    note: key.note,
                    octave: key.octave,
                },
                PianoKeyEntity
            ));

            prev_white_x = Some(x);
            next_white_x += layout.white_key_width + 2.0;
        } else {
            if let Some(prev_x) = prev_white_x {
                let next_x = next_white_x;
                let black_x = (prev_x + next_x) / 2.0;

                commands.spawn((
                    Sprite {
                        color: Color::BLACK,
                        custom_size: Some(Vec2::new(
                            layout.black_key_width,
                            layout.black_key_height,
                        )),
                        ..default()
                    },
                    Transform::from_xyz(
                        black_x,
                        layout.bottom_y + layout.white_key_height
                            - layout.black_key_height / 2.0,
                        2.0,
                    ),
                    KeyColor::Black,
                    KeyPitch {
                        note: key.note,
                        octave: key.octave,
                    },
                    PianoKeyEntity
                ));
            }
        }
    }
}