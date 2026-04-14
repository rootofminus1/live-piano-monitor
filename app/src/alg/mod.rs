pub mod engine;
pub mod dsp;
pub mod data;
pub mod omp;
pub mod detect;


use bevy::prelude::*;
use crossbeam::channel::{unbounded, Receiver};

pub struct PolyphonicPlugin;

use crate::{alg::{detect::Detector, engine::{AudioEngine, Backend}}, core::Pitch};

#[derive(Resource, Default)]
pub struct PolyphonicState {
    pub notes: Vec<Pitch>,
}

#[derive(Resource)]
struct PolyReceiver {
    rx: Receiver<Vec<Pitch>>
}

impl Plugin for PolyphonicPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PolyphonicState>()
            .add_systems(Startup, start_engine)
            .add_systems(Update, update_notes);
    }
}

fn start_engine(mut commands: Commands) {
    let (tx, rx) = unbounded();

    std::thread::spawn(move || {
        let detector = Detector::new();

        println!("dict size: {}", detector.dict.len());
        println!("note_map size: {}", detector.note_map.len());

        let tx_onset = tx.clone();
        let tx_fft   = tx.clone();

        let engine = AudioEngine::new(
            15,
            Backend::PortAudio,

            move || {},

            move || {
                let _ = tx_onset.send(vec![]);
            },

            move |fft| {
                let notes = detector.process(fft);
                let _ = tx_fft.send(notes);
            },
        );

        engine.run();
    });

    commands.insert_resource(PolyReceiver { rx });
}

fn update_notes(
    receiver: Res<PolyReceiver>,
    mut state: ResMut<PolyphonicState>,
) {
    for msg in receiver.rx.try_iter() {
        info!("[NOTES] {:?}", msg);
        state.notes = msg;
    }
}