
use std::sync::{Arc, Mutex};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use yin::Yin;

const BUFFER_SIZE: usize = 2048;

pub fn start_audio_input(tx: crossbeam::channel::Sender<Option<f32>>) {
    println!("starting audio input");

    // Setup audio input
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("input device not found");

    let config = device
        .default_input_config()
        .expect("default input config not found");

    let sample_rate = config.sample_rate() as usize;
    let channels = config.channels() as usize;

    let yin = Yin::init(0.15, 50.0, 1000.0, sample_rate);
    let yin = Arc::new(Mutex::new(yin));

    // TODO: figure out a way to avoid this kind of annoying repetition here, possibly with some trait objects or wrappers
    // alternatively just associate each type with a conversion into f32/f64
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => build_stream_f32(&device, &config.into(), channels, yin.clone(), tx.clone()),
        cpal::SampleFormat::I16 => build_stream_i16(&device, &config.into(), channels, yin.clone(), tx.clone()),
        cpal::SampleFormat::U16 => build_stream_u16(&device, &config.into(), channels, yin.clone(), tx.clone()),
        _ => panic!("NOT SUPPORTED FORMAT"),
    };

    stream.play().unwrap();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn build_stream_f32(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    yin: Arc<Mutex<Yin>>,
    tx: crossbeam::channel::Sender<Option<f32>>,
) -> cpal::Stream {
    let mut buffer: Vec<f64> = Vec::with_capacity(BUFFER_SIZE);

    device.build_input_stream(
        config,
        move |data: &[f32], _: &_| {
            let normalized: Vec<f64> = data.iter().map(|s| *s as f64).collect();
            process_audio_buffer(&normalized, channels, &mut buffer, &yin, &tx);
        },
        |err| eprintln!("build stream error: {err}"),
        None,
    ).unwrap()
}

fn build_stream_i16(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    yin: Arc<Mutex<Yin>>,
    tx: crossbeam::channel::Sender<Option<f32>>,
) -> cpal::Stream {
    let mut buffer: Vec<f64> = Vec::with_capacity(BUFFER_SIZE);

    device.build_input_stream(
        config,
        move |data: &[i16], _: &_| {
            let normalized: Vec<f64> = data.iter().map(|s| *s as f64 / i16::MAX as f64).collect();
            process_audio_buffer(&normalized, channels, &mut buffer, &yin, &tx);
        },
        |err| eprintln!("Stream error: {err}"),
        None,
    ).unwrap()
}

fn build_stream_u16(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    yin: Arc<Mutex<Yin>>,
    tx: crossbeam::channel::Sender<Option<f32>>,
) -> cpal::Stream {
    let mut buffer: Vec<f64> = Vec::with_capacity(BUFFER_SIZE);

    device.build_input_stream(
        config,
        move |data: &[u16], _: &_| {
            let normalized: Vec<f64> = data.iter().map(|s| (*s as f64 - 32768.0) / 32768.0).collect();
            process_audio_buffer(&normalized, channels, &mut buffer, &yin, &tx);
        },
        |err| eprintln!("Stream error: {err}"),
        None,
    ).unwrap()
}

fn process_audio_buffer(
    data: &[f64],
    channels: usize,
    buffer: &mut Vec<f64>,
    yin: &Mutex<Yin>,
    tx: &crossbeam::channel::Sender<Option<f32>>,
) {
    for frame in data.chunks(channels) {
        buffer.push(frame[0]); // TODO: handle stereo for cases with more than 1 channel

        if buffer.len() >= BUFFER_SIZE {
            if let Ok(yin) = yin.lock() {
                let freq = yin.estimate_freq(buffer);

                // println!("[Yin] freq: {:.2} Hz", freq);

                // if freq > 0.0 {
                //     let _ = tx.try_send(freq as f32);
                // }

                let msg = if freq.is_finite() && freq > 20.0 && freq < 5000.0 {
                    Some(freq as f32)
                } else {
                    None
                };
                let _ = tx.try_send(msg);
            }
            buffer.clear();
        }
    }
}
