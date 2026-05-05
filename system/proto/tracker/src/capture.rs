use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crossbeam::channel::Sender;

use alg::dsp::DEFAULT_BLOCK_SIZE;

pub fn list_input_devices() {
    let pa = portaudio::PortAudio::new().expect("PortAudio INIT FAILED");

    println!("\navailable input devices:");
    for i in 0..pa.device_count().unwrap_or(0) {
        if let Ok(info) = pa.device_info(portaudio::DeviceIndex(i as u32)) {
            if info.max_input_channels > 0 {
                println!("[{i:2}]  {}", info.name);
            }
        }
    }
    println!();
}

pub fn start_capture(device_index: usize, tx: Sender<Vec<f32>>, stop: Arc<AtomicBool>) {
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