use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use bevy::prelude::*;
use crossbeam::channel::{Receiver, unbounded};
use alg::{PitchProcessor, lars_processor::LarsProcessor, processor::DetectionMode, yin_processor::YinProcessor};
use core::{ModelInfo, Tone};
use crate::settings::data::AppSettings;



#[derive(Event)]
pub struct StartPipeline;
#[derive(Event)]
pub struct StopPipeline;
#[derive(Event)]
pub struct RestartPipeline;


#[derive(Message)]
pub struct PitchesDetected(pub Vec<Tone>);  // TODO: prob remove this as unused


#[derive(Resource)]
pub struct Pipeline {
    pub stop: Arc<AtomicBool>,
    pub pitch_rx: Receiver<Vec<Tone>>,
}

impl Pipeline {
    pub fn new(settings: &AppSettings, devices: &[DeviceInfo]) -> Self {
        info!("building pipeline");
        let stop = Arc::new(AtomicBool::new(false));

        let (audio_tx, audio_rx) = unbounded::<Vec<f32>>();
        let (pitch_tx, pitch_rx) = unbounded::<Vec<Tone>>();

        let device_index = resolve_device_index(settings, devices);
        spawn_audio_thread(device_index, audio_tx, Arc::clone(&stop));
        spawn_detection_thread(
            audio_rx,
            pitch_tx,
            settings.detection_mode,
            active_model(settings),
            Arc::clone(&stop),
        );

        Pipeline { stop, pitch_rx }
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

#[derive(Resource, Default)]
pub struct DetectedPitches {
    pub notes: Vec<Tone>,
}

#[derive(Resource)]
pub struct DeviceList(pub Vec<DeviceInfo>);


pub struct PipelinePlugin;

impl Plugin for PipelinePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<PitchesDetected>()
            .add_observer(on_start_pipeline)
            .add_observer(on_stop_pipeline)
            .add_observer(on_restart_pipeline)
            .add_systems(Update, receive_pitches.run_if(resource_exists::<Pipeline>));
    }
}


pub fn on_start_pipeline(
    _trigger: On<StartPipeline>,
    mut commands: Commands, 
    settings: Res<AppSettings>
) {
    info!("starting pipeline");
    let devices = list_input_devices();
    for d in &devices {
        info!("Input device [{}]: {}", d.index, d.name);
    }

    commands.insert_resource(DetectedPitches::default());

    let pipeline = Pipeline::new(&settings, &devices);
    commands.insert_resource(DeviceList(devices));
    commands.insert_resource(pipeline);
}

pub fn on_stop_pipeline(
    _trigger: On<StopPipeline>,
    pipeline: Option<Res<Pipeline>>, 
    mut commands: Commands) 
{
    info!("stopping pipeline");

    if let Some(p) = pipeline {
        p.stop();
    }
    
    commands.insert_resource(DetectedPitches::default());
}

fn on_restart_pipeline(
    _trigger: On<RestartPipeline>,
    mut commands: Commands,
    pipeline: Option<Res<Pipeline>>,
    device_list: Res<DeviceList>,
    settings: Res<AppSettings>,
    mut detected: ResMut<DetectedPitches>,
) {
    info!("restarting pipeline");
    if let Some(p) = pipeline {
        p.stop();
    }

    detected.notes.clear();

    info!("on_restart_pipeline: rebuilding audio+detection threads");
    commands.insert_resource(Pipeline::new(&settings, &device_list.0));
}


pub fn receive_pitches(
    pipeline: Res<Pipeline>,
    mut detected: ResMut<DetectedPitches>,
) {
    let mut last = None;

    for batch in pipeline.pitch_rx.try_iter() {
        last = Some(batch);
    }

    if let Some(notes) = last {
        detected.notes = notes;
    }
}




fn resolve_device_index(settings: &AppSettings, devices: &[DeviceInfo]) -> usize {
    settings
        .device_name
        .as_deref()
        .and_then(|name| devices.iter().find(|d| d.name == name))
        .map(|d| d.index)
        .unwrap_or(0)
}

fn active_model(settings: &AppSettings) -> Option<(ModelInfo, std::path::PathBuf)> {
    settings
        .active_model_index
        .and_then(|i| settings.models.get(i))
        .map(|m| (m.clone(), settings.resolved_models_dir()))
}



pub const SR: u32 = 44100;  // TODO: better SR handling accross files algs etc
pub const DEFAULT_BLOCK_SIZE: u32 = 1024;

pub struct DeviceInfo {
    pub index: usize,
    pub name: String,
}

pub fn list_input_devices() -> Vec<DeviceInfo> {
    let pa = portaudio::PortAudio::new().expect("PortAudio init failed");
    let mut devices = Vec::new();
    for i in 0..pa.device_count().unwrap_or(0) {
        if let Ok(info) = pa.device_info(portaudio::DeviceIndex(i as u32)) {
            if info.max_input_channels > 0 {
                devices.push(DeviceInfo {
                    index: i as usize,
                    name: info.name.to_string(),
                });
            }
        }
    }
    devices
}

fn spawn_audio_thread(
    device_index: usize,
    tx: crossbeam::channel::Sender<Vec<f32>>,
    stop: Arc<AtomicBool>,
) {
    std::thread::spawn(move || {
        let pa = portaudio::PortAudio::new().expect("PortAudio init failed");
        let dev = portaudio::DeviceIndex(device_index as u32);
        let info = pa.device_info(dev).expect("invalid PortAudio device index");

        let params = portaudio::StreamParameters::<f32>::new(
            dev,
            1, // mono
            true,
            info.default_low_input_latency,
        );

        let pa_settings = portaudio::InputStreamSettings::new(
            params,
            info.default_sample_rate,
            DEFAULT_BLOCK_SIZE,
        );

        let stop_clone = Arc::clone(&stop);
        let callback = move |portaudio::InputStreamCallbackArgs { buffer, .. }| {
            if stop_clone.load(Ordering::Relaxed) {
                return portaudio::Complete;
            }
            let _ = tx.try_send(buffer.to_vec());
            portaudio::Continue
        };

        let mut stream = pa
            .open_non_blocking_stream(pa_settings, callback)
            .expect("failed to open PortAudio stream");

        stream.start().expect("failed to start PortAudio stream");

        while stream.is_active().unwrap_or(false) {
            if stop.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        info!("Audio capture thread exiting");
    });
}

fn spawn_detection_thread(
    block_rx: crossbeam::channel::Receiver<Vec<f32>>,
    pitch_tx: crossbeam::channel::Sender<Vec<Tone>>,
    mode: DetectionMode,
    model_info: Option<(ModelInfo, std::path::PathBuf)>,
    stop: Arc<AtomicBool>,
) {
    std::thread::spawn(move || {
        let mut processor: Box<dyn PitchProcessor> = match mode {
            DetectionMode::Polyphonic => {
                let Some((info, models_dir)) = model_info else {
                    error!("polyphonic mode selected but no model configured");
                    return;
                };
                match std::panic::catch_unwind(|| LarsProcessor::new(&info, &models_dir)) {
                    Ok(p) => Box::new(p),
                    Err(e) => {
                        error!("failed to load LARS model: {:?}", e);
                        return;
                    }
                }
            }
            DetectionMode::Monophonic => Box::new(YinProcessor::default()),
        };

        loop {
            if stop.load(Ordering::Relaxed) {
                break;
            }
            match block_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(block) => {
                    if let Some(pitches) = processor.process_block(&block) {
                        let _ = pitch_tx.try_send(pitches);
                    }
                }
                Err(crossbeam::channel::RecvTimeoutError::Timeout) => continue,
                Err(crossbeam::channel::RecvTimeoutError::Disconnected) => break,
            }
        }

        info!("detection thread exiting");
    });
}