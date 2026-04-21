use bevy::prelude::*;
use crossbeam::channel::{unbounded, Receiver};
use alg::{processor::DetectionMode, PitchProcessor, lars_processor::LarsProcessor, yin_processor::YinProcessor};
use core::Tone;
use crate::audio::{RawBlockReceiver, start_audio_capture};
use crate::settings::data::AppSettings;

pub struct DetectionPlugin;

impl Plugin for DetectionPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DetectedPitches>()
            .add_systems(Startup, start_detection_thread.after(start_audio_capture))
            .add_systems(Update, (restart_detection_on_device_change, receive_pitches));
    }
}

#[derive(Resource, Default)]
pub struct DetectedPitches {
    pub notes: Vec<Tone>,
}

#[derive(Resource)]
struct DetectionReceiver {
    rx: Receiver<Vec<Tone>>,
}

fn spawn_detection_thread(
    block_rx: crossbeam::channel::Receiver<Vec<f32>>, 
    mode: DetectionMode,
    model_path: std::path::PathBuf,
) -> Receiver<Vec<Tone>> {
    let (tx, rx) = unbounded::<Vec<Tone>>();

    std::thread::spawn(move || {
        info!("Detection thread starting, mode: {:?}", mode);
        
        let mut processor: Box<dyn PitchProcessor> = match mode {
            DetectionMode::Polyphonic => {
                info!("Loading LARS model from {:?}", model_path);
                match std::panic::catch_unwind(|| LarsProcessor::new()) {
                    Ok(p) => {
                        info!("LARS model loaded successfully");
                        Box::new(p)
                    }
                    Err(e) => {
                        error!("Failed to load LARS model: {:?}", e);
                        return;
                    }
                }
            }
            DetectionMode::Monophonic => {
                info!("Using YIN processor");
                Box::new(YinProcessor::default())
            }
        };

        info!("Detection thread running, waiting for blocks");
        let mut block_count = 0u64;
        loop {
            let Ok(block) = block_rx.recv() else { 
                info!("Detection thread: block channel closed, exiting");
                break 
            };
            block_count += 1;
            if block_count % 100 == 0 {
                info!("Detection thread: processed {} blocks", block_count);
            }
            if let Some(pitches) = processor.process_block(&block) {
                if !pitches.is_empty() {
                    info!("Detected pitches: {:?}", pitches);
                }
                let _ = tx.send(pitches);
            }
        }
    });

    rx
}
fn start_detection_thread(
    mut commands: Commands,
    raw: Res<RawBlockReceiver>,
    settings: Res<AppSettings>,
) {
    let rx = spawn_detection_thread(raw.rx.clone(), settings.detection_mode, "fretdata.bin".into());
    commands.insert_resource(DetectionReceiver { rx });
}
fn restart_detection_on_device_change(
    mut commands: Commands,
    raw: Res<RawBlockReceiver>,
    settings: Res<AppSettings>,
    mut current_name: Local<Option<Option<String>>>,
    mut detected: ResMut<DetectedPitches>,
) {
    // First frame: just record state
    if current_name.is_none() {
        *current_name = Some(settings.device_name.clone());
        return;
    }

    if current_name.as_ref().unwrap() == &settings.device_name {
        return;
    }

    // only restart if the resource actually changed (like when an audio restart happened previously)
    if !raw.is_changed() {
        return;
    }

    info!("device changed and new RawBlockReceiver is ready, restarting detection thread");

    *current_name = Some(settings.device_name.clone());
    detected.notes.clear();

    let rx = spawn_detection_thread(raw.rx.clone(), settings.detection_mode, "fretdata.bin".into());
    commands.insert_resource(DetectionReceiver { rx });
}

fn receive_pitches(receiver: Res<DetectionReceiver>, mut state: ResMut<DetectedPitches>) {
    let mut last = None;
    for pitches in receiver.rx.try_iter() { last = Some(pitches); }
    if let Some(pitches) = last { state.notes = pitches; }
}