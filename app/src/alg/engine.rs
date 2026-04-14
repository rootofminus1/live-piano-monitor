use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use crossbeam::channel::{Receiver, Sender, bounded};
use crate::alg::dsp::{DEFAULT_BLOCK_SIZE, FFT_ACCUMULATE_BLOCKS, OFFSET_THRESHOLD_DB, ONSET_THRESHOLD_DB, SR, TOTAL_SIZE, compute_fft, rms_db};

// TODO: maybe move constants to mod.rs

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Backend { 
    PortAudio, 
    Cpal  // TODO: cpal? yay or nay? 
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

// TODO: trait fns instead of boxed closures, maybe
pub struct AudioEngine {
    on_onset: Box<dyn Fn() + Send + 'static>,
    on_offset: Box<dyn Fn() + Send + 'static>,
    on_fft: Box<dyn Fn(Vec<f32>) + Send + 'static>,
    onset_db: f32,
    offset_db: f32,
    device_index: usize,
    backend: Backend,
    stop: Arc<AtomicBool>,
}

impl AudioEngine {
    pub fn new(
        device_index: usize,
        backend: Backend,
        on_onset:  impl Fn() + Send + 'static,
        on_offset: impl Fn() + Send + 'static,
        on_fft:    impl Fn(Vec<f32>) + Send + 'static,
    ) -> Self {
        Self {
            on_onset:  Box::new(on_onset),
            on_offset: Box::new(on_offset),
            on_fft:    Box::new(on_fft),
            onset_db:  ONSET_THRESHOLD_DB,
            offset_db: OFFSET_THRESHOLD_DB,
            device_index, backend,
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn stop_flag(&self) -> Arc<AtomicBool> { Arc::clone(&self.stop) }  // TODO: ctrl+c handler

    pub fn run(self) {
        let (tx, rx) = bounded::<Vec<f32>>(512);
        let stop = Arc::clone(&self.stop);

        match self.backend {
            Backend::PortAudio => spawn_portaudio(self.device_index, tx, Arc::clone(&stop)),
            Backend::Cpal => spawn_cpal(self.device_index, tx, Arc::clone(&stop)),
        }

        process_loop(
            rx, stop,
            self.onset_db, self.offset_db,
            self.on_onset, self.on_offset, self.on_fft,
        );
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

        let mut stream = pa.open_non_blocking_stream(settings, callback)
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
        let device = host.input_devices()
            .expect("cannot enumerate devices")
            .nth(device_index)
            .expect("cpal device index out of range");

        let supported = device.default_input_config().expect("no input config");
        let channels = supported.channels() as usize;

        // Request matching block size; some backends may round it
        let config = cpal::StreamConfig {
            channels: supported.channels(),
            sample_rate: SR,
            buffer_size: cpal::BufferSize::Fixed(DEFAULT_BLOCK_SIZE),
        };

        let stream = device.build_input_stream(
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
        ).expect("failed to build cpal stream");

        stream.play().expect("failed to start cpal stream");

        while !stop.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        stream.pause().ok();
    });
}


fn process_loop(
    rx: Receiver<Vec<f32>>,
    stop: Arc<AtomicBool>,
    onset_db:  f32,
    offset_db: f32,
    on_onset:  Box<dyn Fn() + Send>,
    on_offset: Box<dyn Fn() + Send>,
    on_fft:    Box<dyn Fn(Vec<f32>) + Send>,
) {
    let mut note_on= false;
    let mut ring       = vec![0.0f32; TOTAL_SIZE];
    let mut write_pos = 0usize;
    let mut block_count = 0usize;

    while !stop.load(Ordering::Relaxed) {
        let block = match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(b) => b,
            Err(_) => continue,
        };

        let db = rms_db(&block);

        if !note_on && db >= onset_db {
            note_on = true;
            ring.iter_mut().for_each(|x| *x = 0.0);
            write_pos   = 0;
            block_count = 0;
            on_onset();
        } else if note_on && db < offset_db {
            note_on = false;
            on_offset();
        }

        if note_on {
            for &s in &block {
                ring[write_pos] = s;
                write_pos = (write_pos + 1) % TOTAL_SIZE;
            }

            block_count += 1;
            if block_count >= FFT_ACCUMULATE_BLOCKS {
                block_count = 0;
                
                let mut buf = Vec::with_capacity(TOTAL_SIZE);
                buf.extend_from_slice(&ring[write_pos..]);
                buf.extend_from_slice(&ring[..write_pos]);
                if let Some(fft_vec) = compute_fft(&buf) {
                    on_fft(fft_vec);
                }
            }
        }
    }
}