use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SizedSample, FromSample};
use crossbeam::channel;
use yin::Yin;

const BUFFER_SIZE: usize = 2048;

fn main() {
    let (tx, rx) = channel::bounded::<f64>(8);

    let host = cpal::default_host();
    let device = host.default_input_device().expect("no input device");

    let config = device
        .default_input_config()
        .expect("no default input config");

    let sample_rate = config.sample_rate() as usize;
    let channels = config.channels() as usize;

    let yin = Yin::init(
        0.15,
        50.0,
        1000.0,
        sample_rate,
    );

    let yin = Arc::new(Mutex::new(yin));
    let tx_audio = tx.clone();
    let yin_audio = yin.clone();

    

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => build_stream_f32(&device, &config.into(), channels, yin_audio, tx.clone()),
        cpal::SampleFormat::I16 => build_stream_i16(&device, &config.into(), channels, yin_audio, tx.clone()),
        cpal::SampleFormat::U16 => build_stream_u16(&device, &config.into(), channels, yin_audio, tx.clone()),
        _ => panic!("unsupported sample format"),
    };

    stream.play().unwrap();

    while let Ok(freq) = rx.recv() {
        if freq > 0.0 {
            println!("Pitch: {:.2} Hz", freq);
        }
    }
}





fn build_stream_f32(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    yin: Arc<Mutex<Yin>>,
    tx: crossbeam::channel::Sender<f64>,
) -> cpal::Stream {
    let mut buffer: Vec<f64> = Vec::with_capacity(BUFFER_SIZE);

    device.build_input_stream(
        config,
        move |data: &[f32], _: &_| {
            let normalized: Vec<f64> = data.iter().map(|s| *s as f64).collect();
            process_audio_buffer(&normalized, channels, &mut buffer, &yin, &tx);
        },
        |err| eprintln!("stream error: {err}"),
        None,
    ).unwrap()
}

fn build_stream_i16(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    yin: Arc<Mutex<Yin>>,
    tx: crossbeam::channel::Sender<f64>,
) -> cpal::Stream {
    let mut buffer: Vec<f64> = Vec::with_capacity(BUFFER_SIZE);

    device.build_input_stream(
        config,
        move |data: &[i16], _: &_| {
            let normalized: Vec<f64> = data.iter().map(|s| *s as f64 / i16::MAX as f64).collect();
            process_audio_buffer(&normalized, channels, &mut buffer, &yin, &tx);
        },
        |err| eprintln!("stream error: {err}"),
        None,
    ).unwrap()
}

fn build_stream_u16(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    yin: Arc<Mutex<Yin>>,
    tx: crossbeam::channel::Sender<f64>,
) -> cpal::Stream {
    let mut buffer: Vec<f64> = Vec::with_capacity(BUFFER_SIZE);

    device.build_input_stream(
        config,
        move |data: &[u16], _: &_| {
            let normalized: Vec<f64> = data.iter().map(|s| (*s as f64 - 32768.0) / 32768.0).collect();
            process_audio_buffer(&normalized, channels, &mut buffer, &yin, &tx);
        },
        |err| eprintln!("stream error: {err}"),
        None,
    ).unwrap()
}

fn process_audio_buffer(
    data: &[f64],
    channels: usize,
    buffer: &mut Vec<f64>,
    yin: &Mutex<Yin>,
    tx: &crossbeam::channel::Sender<f64>,
) {
    for frame in data.chunks(channels) {
        buffer.push(frame[0]); // take first channel for mono

        if buffer.len() >= BUFFER_SIZE {
            if let Ok(yin) = yin.lock() {
                let freq = yin.estimate_freq(&buffer);
                let _ = tx.try_send(freq);
            }
            buffer.clear();
        }
    }
}
