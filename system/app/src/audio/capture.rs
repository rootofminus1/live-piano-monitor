use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crossbeam::channel::Sender;

pub const SR: u32 = 44100;
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

pub fn start_capture(
    device_index: usize,
    tx: Sender<Vec<f32>>,
    stop: Arc<AtomicBool>,
) {
    spawn_portaudio(device_index, tx, stop);
}

fn spawn_portaudio(device_index: usize, tx: Sender<Vec<f32>>, stop: Arc<AtomicBool>) {
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

        let settings = portaudio::InputStreamSettings::new(
            params,
            info.default_sample_rate,
            DEFAULT_BLOCK_SIZE,
        );

        let callback = move |portaudio::InputStreamCallbackArgs { buffer, .. }| {
            let _ = tx.try_send(buffer.to_vec());

            if stop.load(Ordering::Relaxed) {
                portaudio::Complete
            } else {
                portaudio::Continue
            }
        };

        let mut stream = pa
            .open_non_blocking_stream(settings, callback)
            .expect("failed to open PortAudio stream");

        stream.start().expect("failed to start PortAudio stream");

        while stream.is_active().unwrap_or(false) {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });
}