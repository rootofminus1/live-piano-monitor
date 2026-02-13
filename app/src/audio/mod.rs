
use bevy::prelude::*;

mod input;

// pub use input::start_audio_input;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PitchState::default())
            .add_systems(Startup, start_audio)
            .add_systems(Update, poll_pitch);
    }
}

#[derive(Resource, Default)]
pub struct PitchState {
    pub current_hz: Option<f32>,
}

#[derive(Resource)]
struct PitchReceiver {
    rx: crossbeam::channel::Receiver<Option<f32>>,
}

fn start_audio(mut commands: Commands) {
    let (tx, rx) = crossbeam::channel::bounded(128);

    std::thread::spawn(move || {
        input::start_audio_input(tx);
    });

    commands.insert_resource(PitchReceiver { rx });
}

fn poll_pitch(
    mut pitch: ResMut<PitchState>,
    recv: Res<PitchReceiver>,
) {
    // while let Ok(freq) = recv.rx.try_recv() {
    //     pitch.current_hz = freq;
    // }
    while let Ok(freq_option) = recv.rx.try_recv() {
        pitch.current_hz = freq_option;
        if let Some(freq) = freq_option {
            println!("[Bevy] freq: {:.2} Hz", freq);
        } else {
            println!("[Bevy] freq: None");
        }
    }
}










