use bevy::prelude::*;
use bevy::log::LogPlugin;
use bevy::log::info;



#[derive(Debug, Clone, Copy, PartialEq, Eq, strum_macros::Display)]
enum Note {
    C, Cs,
    D, Ds,
    E,
    F, Fs,
    G, Gs,
    A, As,
    B,
}

impl Note {
    fn is_black(&self) -> bool {
        matches!(self, Note::Cs | Note::Ds | Note::Fs | Note::Gs | Note::As)
    }

    fn is_white(&self) -> bool {
        !self.is_black()
    }

    // fn next_key_black(&self) -> bool {
    //     matches!(self, Note::C | Note::D | Note::F | Note::G | Note::A)
    // }

    // fn next_key_white(&self) -> bool {
    //     matches!(self, Note::Cs | Note::Ds | Note::E | Note::Fs | Note::Gs | Note::As | Note::B)
    // }

    fn all_notes() -> [Self; 12] {
            [
            Note::C, Note::Cs, Note::D, Note::Ds, Note::E,
            Note::F, Note::Fs, Note::G, Note::Gs,
            Note::A, Note::As, Note::B,
        ]
    }
}

#[derive(Clone)]
struct KeyInfo {
    note: Note,
    octave: i32,
}

impl KeyInfo {
    pub fn name(&self) -> String {
        format!("{}{}", self.note, self.octave)
    }
}


struct KeyboardSpec {
    start_note: Note,
    start_octave: i32,
    key_count: usize,
}

impl KeyboardSpec {
    fn from_octaves(
        start_note: Note,
        start_octave: i32,
        octaves: usize,
    ) -> Self {
        Self {
            start_note,
            start_octave,
            key_count: octaves * 12 + 1,  // this might not work for when we start from some cursed key rather than a C, TODO: test
        }
    }

    // TODO: could instead be kept in a text file or config etc, maybe
    fn piano_88() -> Self {
        Self {
            start_note: Note::A,
            start_octave: 0,
            key_count: 88,
        }
    }

    fn piano_smaller() -> Self {
        Self::from_octaves(Note::C, 3, 5)
    }
}

fn generate_keys(spec: &KeyboardSpec) -> Vec<KeyInfo> {
    let scale = Note::all_notes();

    let mut keys = Vec::with_capacity(spec.key_count);

    let mut octave = spec.start_octave;
    let mut index = scale
        .iter()
        .position(|&n| n == spec.start_note)
        .unwrap();

    for _ in 0..spec.key_count {
        let note = scale[index];
        keys.push(KeyInfo { note, octave });

        index += 1;
        if index == 12 {
            index = 0;
            octave += 1;
        }
    }

    keys
}



const WHITE_KEY_HEIGHT_RATIO: f32 = 0.35;
const BLACK_KEY_HEIGHT_RATIO: f32 = 0.22;
const KEY_GAP: f32 = 2.0;

#[derive(Resource, Clone)]
struct PianoLayout {
    white_key_width: f32,
    white_key_height: f32,
    black_key_width: f32,
    black_key_height: f32,
    start_x: f32,
    bottom_y: f32,
}

#[derive(Component)]
enum PianoKey {
    White,
    Black,
}

#[derive(Component)]
struct PianoKeyPitch {
    note: Note,
    octave: i32,
}


fn calculate_layout(window: &Window, keys: &[KeyInfo]) -> PianoLayout {
    let white_key_count = keys.iter().filter(|k| k.note.is_white()).count();

    let usable_width = window.width();

    let white_key_width =
        (usable_width / white_key_count as f32) - KEY_GAP;
    let white_key_height = window.height() * WHITE_KEY_HEIGHT_RATIO;

    let black_key_width = white_key_width * 0.6;
    let black_key_height = window.height() * BLACK_KEY_HEIGHT_RATIO;

    let total_width =
        white_key_count as f32 * (white_key_width + KEY_GAP);

    let start_x = -window.width() / 2.0
        + (window.width() - total_width) / 2.0;

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

// fn calculate_layout(window: &Window, keys: &[KeyInfo]) -> PianoLayout {
//     let white_key_count = keys.iter().filter(|k| k.note.is_white()).count();

//     let usable_width = window.width();

//     let white_key_width = 
//         (usable_width - KEY_GAP * (white_key_count as f32 - 1.0))
//         / white_key_count as f32;

//     let white_key_height = window.height() * WHITE_KEY_HEIGHT_RATIO;

//     let black_key_width = white_key_width * 0.6;
//     let black_key_height = window.height() * BLACK_KEY_HEIGHT_RATIO;

//     // Now compute total width correctly
//     let total_width =
//         white_key_count as f32 * white_key_width +
//         (white_key_count as f32 - 1.0) * KEY_GAP;

//     let start_x = -window.width() / 2.0 + (window.width() - total_width) / 2.0;
//     let bottom_y = -window.height() / 2.0;

//     PianoLayout {
//         white_key_width,
//         white_key_height,
//         black_key_width,
//         black_key_height,
//         start_x,
//         bottom_y,
//     }
// }


fn spawn_keyboard(
    commands: &mut Commands,
    keys: &[KeyInfo],
    layout: &PianoLayout,
) {
    info!("weoa");
    let mut next_white_x = layout.start_x;
    info!("{}", next_white_x);
    let mut prev_white_x: Option<f32> = None;

    for key in keys {
        if key.note.is_white() {
            let x = next_white_x;

            commands.spawn((
                Sprite {
                    color: Color::rgb(0.95, 0.95, 0.95),
                    custom_size: Some(Vec2::new(
                        layout.white_key_width,
                        layout.white_key_height,
                    )),
                    ..default()
                },
                Transform::from_xyz(
                    x,
                    layout.bottom_y + layout.white_key_height / 2.0,
                    0.0,
                ),
                PianoKey::White,
                PianoKeyPitch {
                    note: key.note,
                    octave: key.octave
                },
            ));

            prev_white_x = Some(x);
            next_white_x += layout.white_key_width + KEY_GAP;

        } else {

            // what psycho would start with a black key? doesnt hurt to just do this anyway 
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
                        1.0,
                    ),
                    PianoKey::Black,
                    PianoKeyPitch {
                        note: key.note,
                        octave: key.octave
                    },
                ));
            }
        }
    }
}



fn setup(
    mut commands: Commands,
    windows: Query<&Window>,
) {
    commands.spawn(Camera2d::default());

    let spec = KeyboardSpec::piano_smaller();
    let spec = KeyboardSpec::piano_88();

    let keys = generate_keys(&spec);

    let window = windows.single();
    let layout = calculate_layout(window, &keys);

    commands.insert_resource(layout.clone());
    spawn_keyboard(&mut commands, &keys, &layout);
}



#[derive(Resource, Default)]
struct PitchState {
    current_hz: Option<f32>,
}

#[derive(Resource)]
struct PitchReceiver {
    rx: crossbeam::channel::Receiver<f32>,
}

fn audio_loop(tx: crossbeam::channel::Sender<f32>) {
    loop {
        std::thread::sleep(std::time::Duration::from_millis(300));
        let test_values = [220.0, 247.0, 262.0, 294.0, 330.0]; // random
        let v = test_values[rand::random::<usize>() % test_values.len()];
        tx.send(v).ok();
    }
}

fn poll_pitch(mut pitch: ResMut<PitchState>, recv: Res<PitchReceiver>) {
    while let Ok(freq) = recv.rx.try_recv() {
        pitch.current_hz = Some(freq);
    }
}


fn freq_to_note(freq: f32) -> Option<(Note, i32)> {
    if freq <= 0.0 { return None; }

    let midi = 69.0 + 12.0 * (freq / 440.0).log2();
    let midi = midi.round() as i32;

    let note_index = midi % 12;
    let octave = midi / 12 - 1;

    let notes = Note::all_notes();

    Some((notes[note_index as usize], octave))
}


fn highlight_keys(
    pitch: Res<PitchState>,
    mut keys: Query<(&PianoKeyPitch, &mut Sprite)>
) {
    let Some(freq) = pitch.current_hz else { return };
    let Some((note, octave)) = freq_to_note(freq) else { return };

    for (key, mut sprite) in &mut keys {
        if key.note == note && key.octave == octave {
            sprite.color = Color::rgb(1., 0., 0.);
        } else if key.note.is_white() {
            sprite.color = Color::rgb(0.95, 0.95, 0.95);
        } else {
            sprite.color = Color::BLACK;
        }
    }
}

fn start_audio(mut commands: Commands) {
    let (tx, rx) = crossbeam::channel::bounded(128);

    std::thread::spawn(move || { audio_loop(tx); });

    commands.insert_resource(PitchReceiver { rx });
    commands.insert_resource(PitchState::default());
}


fn move_note_blocks(
    time: Res<Time>,
    mut query: Query<(&NoteBlock, &mut Transform)>
) {
    for (note, mut transform) in &mut query {
        transform.translation.y += note.speed * time.delta_secs();
    }
}

fn despawn_offscreen_notes(
    windows: Query<&Window>,
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<NoteBlock>>,
) {
    let window = windows.single();
    for (entity, transform) in &query {
        if transform.translation.y > window.height() / 2.0 + 50.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_note_block(
    pitch: Res<PitchState>,
    layout: Res<PianoLayout>,
    keys: Query<(&PianoKeyPitch, &Transform)>,
    mut commands: Commands,
) {
    let Some(freq) = pitch.current_hz else { return };
    let Some((note, octave)) = freq_to_note(freq) else { return };


    let key_x = keys
        .iter()
        .find(|(k, _)| k.note == note && k.octave == octave)
        .map(|(_, t)| t.translation.x)
        .unwrap_or(0.0);

    let key_width = layout.white_key_width;
    let note_height = 20.0;
    let start_y = layout.bottom_y + layout.white_key_height;

    commands.spawn((
        Sprite {
            color: Color::rgb(0.3, 0.7, 1.0),
            custom_size: Some(Vec2::new(key_width, note_height)),
            ..default()
        },
        Transform::from_xyz(key_x, start_y, 0.5),
        NoteBlock {
            note,
            octave,
            velocity: 1.0,
            speed: 100.0,
        }
    ));
}



#[derive(Component)]
struct NoteBlock {
    note: Note,
    octave: i32,
    velocity: f32,
    speed: f32,
}






fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin::default()))
        .add_systems(Startup, (setup, start_audio))
        .add_systems(Update, (poll_pitch, highlight_keys, spawn_note_block, move_note_blocks, despawn_offscreen_notes))
        .run();
}


