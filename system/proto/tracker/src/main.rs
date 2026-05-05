mod capture;
mod piano;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use alg::{lars_processor::LarsProcessor, PitchProcessor};
use core::{generate_keys, ModelInfo};


fn main() {
    let (device_index, model_info, models_dir) = parse_args();

    let all_keys = generate_keys(&model_info.to_keyboard_spec());

    println!("tracker  |  model: {}  |  {} notes",model_info.name, all_keys.len(),);
    println!("Ctrl-C to stop\n");

    let mut processor = LarsProcessor::new(&model_info, &models_dir);

    let mut piano_height = None;
    piano::redraw(&all_keys, &[], &mut piano_height);

    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = Arc::clone(&stop);
        ctrlc::set_handler(move || stop.store(true, Ordering::Relaxed)).ok();
    }

    let (tx, rx) = crossbeam::channel::bounded::<Vec<f32>>(512);
    capture::start_capture(device_index, tx, Arc::clone(&stop));

    while !stop.load(Ordering::Relaxed) {
        let block = match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(b) => b,
            Err(_) => continue,
        };

        if let Some(pitches) = processor.process_block(&block) {
            piano::redraw(&all_keys, &pitches, &mut piano_height);
        }
    }

    println!("\nSTOPPED.");
}

fn parse_args() -> (usize, ModelInfo, PathBuf) {
    let args: Vec<String> = std::env::args().collect();

    let mut device_index: Option<usize> = None;
    let mut settings_path = PathBuf::from("settings.json");
    let mut model_index: Option<usize> = None;
    let mut models_dir_override: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--list-devices" => {
                capture::list_input_devices();
                std::process::exit(0);
            }
            "--device" => {
                i += 1;
                device_index = Some(args[i].parse().expect("invalid device index"));
            }
            "--settings" => {
                i += 1;
                settings_path = PathBuf::from(&args[i]);
            }
            "--model" => {
                i += 1;
                model_index = Some(args[i].parse().expect("invalid model index"));
            }
            "--models-dir" => {
                i += 1;
                models_dir_override = Some(PathBuf::from(&args[i]));
            }
            _ => {}
        }
        i += 1;
    }

    let device_index = device_index.unwrap_or_else(|| {
        capture::list_input_devices();
        eprintln!("error: --device <index> is required");
        std::process::exit(1);
    });

    let settings_str = std::fs::read_to_string(&settings_path).unwrap_or_else(|_| {
        eprintln!("error: could not read '{}'", settings_path.display());
        std::process::exit(1);
    });

    #[derive(serde::Deserialize)]
    struct Settings {
        models: Vec<ModelInfo>,
        active_model_index: Option<usize>,
        models_dir: Option<String>,
    }

    let settings: Settings = serde_json::from_str(&settings_str).unwrap_or_else(|e| {
        eprintln!("error parsing settings.json: {e}");
        std::process::exit(1);
    });

    let idx = model_index.or(settings.active_model_index).unwrap_or(0);

    let info = settings
        .models
        .get(idx)
        .unwrap_or_else(|| {
            eprintln!("error: no model at index {idx}");
            std::process::exit(1);
        })
        .clone();

    let models_dir = models_dir_override
        .or_else(|| settings.models_dir.map(PathBuf::from))
        .or_else(|| std::env::var("PIANO_MODELS_DIR").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(".models"));

    (device_index, info, models_dir)
}