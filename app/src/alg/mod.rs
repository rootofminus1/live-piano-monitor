pub mod data;
pub mod dsp;
pub mod lars_processor;
pub mod processor;
pub mod sparse;
pub mod yin_processor;

use bevy::prelude::*;
use crossbeam::channel::{unbounded, Receiver};
use crate::{alg::{lars_processor::LarsProcessor, processor::{DetectionMode, PitchProcessor}, yin_processor::YinProcessor}, audio::{RawBlockReceiver, start_audio_capture}, core::Pitch};

pub struct DetectionPlugin;

impl Plugin for DetectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DetectedPitches>()
            .init_resource::<DetectionMode>()
            // TODO: IMPORTANT!!!! change the after to a better startup system
            .add_systems(Startup, start_detection_thread.after(start_audio_capture))
            .add_systems(Update, receive_pitches);
    }
}

#[derive(Resource, Default)]
pub struct DetectedPitches {
    pub notes: Vec<Pitch>,
}

#[derive(Resource)]
struct DetectionReceiver {
    rx: Receiver<Vec<Pitch>>,
}

fn start_detection_thread(
    mut commands: Commands,
    raw: Res<RawBlockReceiver>,
    mode: Res<DetectionMode>,
) {
    let (tx, rx) = unbounded::<Vec<Pitch>>();
    let block_rx = raw.rx.clone();
    let initial_mode = *mode;

    std::thread::spawn(move || {
        let mut processor: Box<dyn PitchProcessor> = match initial_mode {
            DetectionMode::Polyphonic => Box::new(LarsProcessor::new()),
            DetectionMode::Monophonic => Box::new(YinProcessor::default()),
        };

        loop {
            let block = match block_rx.recv() {
                Ok(b) => b,
                Err(_) => break,
            };

            if let Some(pitches) = processor.process_block(&block) {
                let _ = tx.send(pitches);
            }
        }
    });

    commands.insert_resource(DetectionReceiver { rx });
}

fn receive_pitches(receiver: Res<DetectionReceiver>, mut state: ResMut<DetectedPitches>) {
    let mut last = None;
    for pitches in receiver.rx.try_iter() {
        last = Some(pitches);
    }
    if let Some(pitches) = last {
        state.notes = pitches;
    }
}