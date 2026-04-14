use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crossbeam::channel::Sender;

pub const SR: u32 = 44100;
pub const DEFAULT_BLOCK_SIZE: u32 = 1024;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Backend {
    PortAudio,
    Cpal,  // might no longer be used
}

impl Backend {
    pub fn from_str(s: &str) -> Self {
        if s.eq_ignore_ascii_case("cpal") { Backend::Cpal } else { Backend::PortAudio }
    }
}

pub fn list_devices(backend: Backend) {
    match backend {
        Backend::PortAudio => {
            let pa = portaudio::PortAudio::new().expect("PortAudio init failed");
            println!("\navailable audio input devices (portaudio):");
            for i in 0..pa.device_count().unwrap_or(0) {
                if let Ok(info) = pa.device_info(portaudio::DeviceIndex(i)) {
                    if info.max_input_channels > 0 {
                        println!("  [{i:2}]  {}", info.name);
                    }
                }
            }
            println!();
        }
        Backend::Cpal => {
            use cpal::traits::{DeviceTrait, HostTrait};
            let host = cpal::default_host();
            println!("\navailable audio input devices (cpal / {}):", host.id().name());
            if let Ok(devices) = host.input_devices() {
                for (i, dev) in devices.enumerate() {
                    println!("  [{i:2}]  {}", dev.name().unwrap_or_default());
                }
            }
            println!();
        }
    }
}


pub fn start_capture(
    device_index: usize,
    backend: Backend,
    tx: Sender<Vec<f32>>,
    stop: Arc<AtomicBool>,
) {
    match backend {
        Backend::PortAudio => spawn_portaudio(device_index, tx, stop),
        Backend::Cpal => spawn_cpal(device_index, tx, stop),
    }
}

fn spawn_portaudio(device_index: usize, tx: Sender<Vec<f32>>, stop: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let pa = portaudio::PortAudio::new().expect("PortAudio init failed");
        let dev = portaudio::DeviceIndex(device_index as u32);
        let info = pa.device_info(dev).expect("invalid PortAudio device index");

        let params = portaudio::StreamParameters::<f32>::new(
            dev, 1, true, info.default_low_input_latency,
        );
        let settings = portaudio::InputStreamSettings::new(
            params, info.default_sample_rate, DEFAULT_BLOCK_SIZE,
        );

        let callback = move |portaudio::InputStreamCallbackArgs { buffer, .. }| {
            let _ = tx.try_send(buffer.to_vec());
            if stop.load(Ordering::Relaxed) { portaudio::Complete }
            else { portaudio::Continue }
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

fn spawn_cpal(device_index: usize, tx: Sender<Vec<f32>>, stop: Arc<AtomicBool>) {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = host
            .input_devices()
            .expect("cannot enumerate devices")
            .nth(device_index)
            .expect("cpal device index out of range");

        let supported = device.default_input_config().expect("no input config");
        let channels = supported.channels() as usize;

        let config = cpal::StreamConfig {
            channels: supported.channels(),
            sample_rate: SR,
            buffer_size: cpal::BufferSize::Fixed(DEFAULT_BLOCK_SIZE),
        };

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _| {
                    let mono: Vec<f32> = if channels == 1 {
                        data.to_vec()
                    } else {
                        data.chunks(channels).map(|ch| ch[0]).collect()
                    };
                    let _ = tx.try_send(mono);
                },
                |e| eprintln!("cpal error: {e}"),
                None,
            )
            .expect("failed to build cpal stream");

        stream.play().expect("failed to start cpal stream");
        while !stop.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        stream.pause().ok();
    });
}