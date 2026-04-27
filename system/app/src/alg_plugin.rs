use bevy::prelude::*;
use crossbeam::channel::{unbounded, Receiver};
use alg::{processor::DetectionMode, PitchProcessor, lars_processor::LarsProcessor, yin_processor::YinProcessor};
use core::{ModelInfo, Tone};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::audio::{RawBlockReceiver, start_audio_capture};
use crate::settings::data::AppSettings;

pub struct DetectionPlugin;

impl Plugin for DetectionPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DetectedPitches>()
            .add_systems(Startup, start_detection_thread.after(start_audio_capture))
            .add_systems(Update, (
                restart_detection_on_device_change, 
                restart_detection_on_settings_change,
                receive_pitches
            ));
    }
}

#[derive(Resource, Default)]
pub struct DetectedPitches {
    pub notes: Vec<Tone>,
}

#[derive(Resource)]
struct DetectionReceiver {
    rx: Receiver<Vec<Tone>>,
    stop: Arc<AtomicBool>,
}

fn spawn_detection_thread(
    block_rx: crossbeam::channel::Receiver<Vec<f32>>,
    mode: DetectionMode,
    model_info: Option<(ModelInfo, std::path::PathBuf)>, // (info, models_dir)
    stop: Arc<AtomicBool>, 
) -> Receiver<Vec<Tone>> {
    let (tx, rx) = unbounded::<Vec<Tone>>();

    std::thread::spawn(move || {
        let mut processor: Box<dyn PitchProcessor> = match mode {
            DetectionMode::Polyphonic => {
                let Some((info, models_dir)) = model_info else {
                    error!("Polyphonic mode selected but no model configured");
                    return;
                };
                match std::panic::catch_unwind(|| LarsProcessor::new(&info, &models_dir)) {
                    Ok(p) => Box::new(p),
                    Err(e) => {
                        error!("Failed to load LARS model: {:?}", e);
                        return;
                    }
                }
            }
            DetectionMode::Monophonic => Box::new(YinProcessor::default()),
        };

        loop {
            if stop.load(Ordering::Relaxed) { break; }
            
            let Ok(block) = block_rx.recv() else { break };
            if let Some(pitches) = processor.process_block(&block) {
                let _ = tx.send(pitches);
            }
        }
    });

    rx
}

fn active_model(settings: &AppSettings) -> Option<(ModelInfo, std::path::PathBuf)> {
    settings.active_model_index
        .and_then(|i| settings.models.get(i))
        .map(|m| (m.clone(), settings.resolved_models_dir()))
}


#[derive(PartialEq, Clone)]
struct DetectionKey {
    mode: DetectionMode,
    model_index: Option<usize>,
}

fn start_detection_thread(
    mut commands: Commands,
    raw: Res<RawBlockReceiver>,
    settings: Res<AppSettings>,
) {
    let stop = Arc::new(AtomicBool::new(false));
    let rx = spawn_detection_thread(
        raw.rx.clone(),
        settings.detection_mode,
        active_model(&settings),
        Arc::clone(&stop),
    );
    commands.insert_resource(DetectionReceiver { rx, stop });
}

// Restart when the portaudio device changes (RawBlockReceiver is replaced)
fn restart_detection_on_device_change(
    mut commands: Commands,
    raw: Res<RawBlockReceiver>,
    settings: Res<AppSettings>,
    mut initialized: Local<bool>,
    mut detected: ResMut<DetectedPitches>,
    receiver: Option<Res<DetectionReceiver>>,
) {
    if !*initialized {
        *initialized = true;
        return;
    }
    
    if !raw.is_changed() {
        return;
    }

    if let Some(r) = receiver { r.stop.store(true, Ordering::Relaxed); }

    info!("RawBlockReceiver changed, restarting detection thread");
    detected.notes.clear();

    let stop = Arc::new(AtomicBool::new(false));

    let rx = spawn_detection_thread(
        raw.rx.clone(),
        settings.detection_mode,
        active_model(&settings),
        Arc::clone(&stop)
    );
    commands.insert_resource(DetectionReceiver { rx, stop });
}

// Restart when detection settings change (mode or model), independent of audio
fn restart_detection_on_settings_change(
    mut commands: Commands,
    raw: Res<RawBlockReceiver>,
    settings: Res<AppSettings>,
    mut last_key: Local<Option<DetectionKey>>,
    mut detected: ResMut<DetectedPitches>,
    receiver: Option<Res<DetectionReceiver>>,
) {
    let key = DetectionKey {
        mode: settings.detection_mode,
        model_index: settings.active_model_index,
    };

    if last_key.as_ref() == Some(&key) {
        return;
    }
    
    // Skip the very first frame (just initialize)
    if last_key.is_none() {
        *last_key = Some(key);
        return;
    }

    *last_key = Some(key);

    if let Some(r) = receiver { r.stop.store(true, Ordering::Relaxed); }

    info!("Detection settings changed, restarting detection thread");
    detected.notes.clear();

    let stop = Arc::new(AtomicBool::new(false));

    let rx = spawn_detection_thread(
        raw.rx.clone(),
        settings.detection_mode,
        active_model(&settings),
        Arc::clone(&stop)
    );
    commands.insert_resource(DetectionReceiver { rx, stop });
}


fn receive_pitches(receiver: Res<DetectionReceiver>, mut state: ResMut<DetectedPitches>) {
    let mut last = None;
    for pitches in receiver.rx.try_iter() { last = Some(pitches); }
    if let Some(pitches) = last { state.notes = pitches; }
}