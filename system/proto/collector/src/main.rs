mod capture;

use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use alg::{
    data::TrainingData,
    dsp::{compute_fft, rms_db, FFT_ACCUMULATE_BLOCKS, OFFSET_THRESHOLD_DB, ONSET_THRESHOLD_DB, TOTAL_SIZE},
};
use core::{generate_keys, ModelInfo};



fn main() {
    let (device_index, datafile, model_info) = parse_args();

    let keys = generate_keys(&model_info.to_keyboard_spec());
    let n_notes = keys.len();

    println!("\n{}", "═".repeat(55));
    println!("collector  |  model: {}  |  {} notes", model_info.name, n_notes);
    println!("Enter  confirm and advance");
    println!("r  re-record current note");
    println!("b  go back");
    println!("q  quit without saving");
    println!("{}\n", "═".repeat(55));

    let data: Arc<Mutex<Vec<Vec<Vec<f32>>>>> = Arc::new(Mutex::new(vec![vec![]; n_notes]));
    let cur_idx: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let note_on: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let stop = Arc::new(AtomicBool::new(false));

    let (tx, rx) = crossbeam::channel::bounded::<Vec<f32>>(512);
    capture::start_capture(device_index, tx, Arc::clone(&stop));

    
    {
        let data = Arc::clone(&data);
        let cur_idx = Arc::clone(&cur_idx);
        let note_on = Arc::clone(&note_on);
        let stop = Arc::clone(&stop);

        std::thread::spawn(move || {
            // TODO: abstract this away into some common lib (same for the capture)
            let mut ring = vec![0.0f32; TOTAL_SIZE];
            let mut write_pos = 0usize;
            let mut block_count = 0usize;
            let mut is_on = false;

            while !stop.load(Ordering::Relaxed) {
                let block = match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(b) => b,
                    Err(_) => continue,
                };

                let db = rms_db(&block);

                if !is_on && db >= ONSET_THRESHOLD_DB {
                    is_on = true;
                    *note_on.lock().unwrap() = true;
                    ring.iter_mut().for_each(|x| *x = 0.0);
                    write_pos = 0;
                    block_count = 0;
                } else if is_on && db < OFFSET_THRESHOLD_DB {
                    is_on = false;
                    *note_on.lock().unwrap() = false;
                }

                if !is_on { continue; }

                for &s in &block {
                    ring[write_pos] = s;
                    write_pos = (write_pos + 1) % TOTAL_SIZE;
                }

                block_count += 1;
                if block_count < FFT_ACCUMULATE_BLOCKS { continue; }
                block_count = 0;

                let mut buf = Vec::with_capacity(TOTAL_SIZE);
                buf.extend_from_slice(&ring[write_pos..]);
                buf.extend_from_slice(&ring[..write_pos]);

                if let Some(fft) = compute_fft(&buf) {
                    if *note_on.lock().unwrap() {
                        let idx = *cur_idx.lock().unwrap();
                        if idx < n_notes {
                            data.lock().unwrap()[idx].push(fft);
                        }
                    }
                }
            }
        });
    }

    let mut idx = 0usize;
    while idx < n_notes {
        let frames_before = data.lock().unwrap()[idx].len();
        print!("{:6} [{}/{}] ({} frames) > ", keys[idx], idx + 1, n_notes, frames_before);
        io::stdout().flush().ok();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();

        match input.trim() {
            "q" => {
                stop.store(true, Ordering::Relaxed);
                println!("quitting");
                return;
            }
            "b" => {
                if idx > 0 {
                    idx -= 1;
                    data.lock().unwrap()[idx].clear();
                    *cur_idx.lock().unwrap() = idx;
                    println!("back to {}", keys[idx]);
                } else {
                    println!("already at first note");
                }
                continue;
            }
            "r" => {
                data.lock().unwrap()[idx].clear();
                println!("re-recording");
                continue;
            }
            _ => {}
        }

        let new_frames = data.lock().unwrap()[idx].len() - frames_before;
        if new_frames == 0 {
            println!("no audio detected, try again");
        } else {
            println!("{} frames", new_frames);
            *cur_idx.lock().unwrap() = idx + 1;
            idx += 1;
        }
    }

    stop.store(true, Ordering::Relaxed);

    let collected = data.lock().unwrap().iter().filter(|d| !d.is_empty()).count();
    if collected == 0 {
        println!("nothing to save");
        return;
    }

    print!("\nsave {}/{} notes to '{}'? (y/n) ", collected, n_notes, datafile.display());
    io::stdout().flush().ok();
    let mut answer = String::new();
    io::stdin().read_line(&mut answer).ok();
    if answer.trim().to_lowercase() != "y" {
        println!("discarded");
        return;
    }


    match (TrainingData { notes: data.lock().unwrap().clone() }).save(&datafile) {
        Ok(_) => println!("saved to {}", datafile.display()),
        Err(e) => eprintln!("save error: {e}"),
    }

}

fn parse_args() -> (usize, PathBuf, ModelInfo) {
    let args: Vec<String> = std::env::args().collect();

    let mut device_index: Option<usize> = None;
    let mut settings_path = PathBuf::from("settings.json");
    let mut model_index: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--list-devices" => { capture::list_input_devices(); std::process::exit(0); }
            "--device" => { i += 1; device_index = Some(args[i].parse().expect("invalid device index")); }
            "--settings" => { i += 1; settings_path = PathBuf::from(&args[i]); }
            "--model" => { i += 1; model_index = Some(args[i].parse().expect("invalid model index")); }
            _ => {}
        }
        i += 1;
    }

    let device_index = device_index.unwrap_or_else(|| {
        capture::list_input_devices();
        eprintln!("error: --device <index> is required");
        std::process::exit(1);
    });

    #[derive(serde::Deserialize)]
    struct Settings {
        models: Vec<ModelInfo>,
        active_model_index: Option<usize>,
        models_dir: Option<String>,
    }

    let settings: Settings = std::fs::read_to_string(&settings_path)
        .map_err(|_| format!("could not read '{}'", settings_path.display()))
        .and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string()))
        .unwrap_or_else(|e| { eprintln!("error: {e}"); std::process::exit(1); });

    let idx = model_index.or(settings.active_model_index).unwrap_or(0);

    let info = settings.models.get(idx).unwrap_or_else(|| {
        eprintln!("error: no model at index {idx}");
        std::process::exit(1);
    }).clone();

    let models_dir = settings.models_dir
        .map(PathBuf::from)
        .or_else(|| std::env::var("PIANO_MODELS_DIR").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(".models"));

    (device_index, models_dir.join(&info.filename), info)
}