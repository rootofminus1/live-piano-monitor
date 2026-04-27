use std::sync::OnceLock;
use rustfft::{FftPlanner, num_complex::Complex};

// TODO: move consts and remove unused ones

pub const SR: u32 = 44100;
pub const TOTAL_SIZE: usize = 4096;  // TUNABLE (liveliness vs accuracy)
pub const CROP_SIZE: usize = 256;
pub const NONZERO_COEFS: usize = 6;
pub const ONSET_THRESHOLD_DB: f32 = -30.0;
pub const OFFSET_THRESHOLD_DB: f32 = -40.0;  // TUNABLE (offset recognition)

// value 10 means 1024 at 44100 below matches python, might also be tunable 
pub const FFT_ACCUMULATE_BLOCKS: usize = 10;  // TUNABLE (liveliness vs accuracy)

// might be changeable for better results - to be tested
pub const DEFAULT_BLOCK_SIZE: u32 = 1024;

static HANNING: OnceLock<Vec<f32>> = OnceLock::new();

pub fn hanning_window() -> &'static [f32] {
    HANNING.get_or_init(|| {
        (0..TOTAL_SIZE)
            .map(|i| {
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (TOTAL_SIZE - 1) as f32).cos())
            })
            .collect()
    })
}

static FFT_PLAN: OnceLock<std::sync::Arc<dyn rustfft::Fft<f32>>> = OnceLock::new();

fn fft_plan() -> &'static std::sync::Arc<dyn rustfft::Fft<f32>> {
    FFT_PLAN.get_or_init(|| {
        let mut planner = FftPlanner::<f32>::new();
        planner.plan_fft_forward(TOTAL_SIZE)
    })
}

pub fn compute_fft(ring_buf: &[f32]) -> Option<Vec<f32>> {
    debug_assert!(ring_buf.len() >= TOTAL_SIZE);
    let win = hanning_window();
    
    let mut buf: Vec<Complex<f32>> = ring_buf[..TOTAL_SIZE]
        .iter()
        .zip(win.iter())
        .map(|(&s, &w)| Complex { re: s * w, im: 0.0 })
        .collect();

    fft_plan().process(&mut buf);

    let mag: Vec<f32> = buf[..CROP_SIZE].iter().map(|c| c.norm()).collect();
    let norm: f32 = mag.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm < 1e-12 { return None; }

    Some(mag.iter().map(|x| x / norm).collect())
}

pub fn rms_db(block: &[f32]) -> f32 {
    let rms = (block.iter().map(|x| x * x).sum::<f32>() / block.len() as f32 + 1e-12).sqrt();
    20.0 * rms.log10()
}
