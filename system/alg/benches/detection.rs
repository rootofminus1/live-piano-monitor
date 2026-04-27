use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ndarray::{Array1, Array2};
use sklears_core::traits::Fit;
use sklears_linear::Lars;
use alg::dsp::{compute_fft, CROP_SIZE, TOTAL_SIZE, NONZERO_COEFS};

// apparently these numbers give the best fair statistical structure 
const LCG_MULT: u64 = 6364136223846793005;
const LCG_INC: u64 = 1442695040888963407;
const LCG_SHIFT: u32 = 33;


#[inline]
fn lcg_next_u64(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(LCG_MULT).wrapping_add(LCG_INC);
    *state
}

#[inline]
fn lcg_next_f32(state: &mut u64) -> f32 {
    let x = lcg_next_u64(state);
    ((x >> LCG_SHIFT) as f32 / u32::MAX as f32) * 2.0 - 1.0
}

#[inline]
fn lcg_next_f64(state: &mut u64) -> f64 {
    let x = lcg_next_u64(state);
    ((x >> LCG_SHIFT) as f64 / u32::MAX as f64) * 2.0 - 1.0
}


fn make_random_signal(seed: u64) -> Vec<f32> {
    let mut state = seed;
    (0..TOTAL_SIZE).map(|_| lcg_next_f32(&mut state)).collect()
}


fn bench_fft(c: &mut Criterion) {
    let signal = make_random_signal(42);

    c.bench_function("fft_4096", |b| {
        b.iter(|| {
            let buf = signal.clone();
            black_box(compute_fft(black_box(&buf)))
        })
    });
}

fn bench_lars(c: &mut Criterion) {
    const N_ATOMS: usize = 61;

    let mut state = 99u64;

    let mut dict = Array2::<f64>::zeros((CROP_SIZE, N_ATOMS));
    
    for col in 0..N_ATOMS {
        let mut norm = 0.0;

        for row in 0..CROP_SIZE {
            let v = lcg_next_f64(&mut state);
            dict[(row, col)] = v;
            norm += v * v;
        }

        let norm = norm.sqrt().max(1e-12);
        
        for row in 0..CROP_SIZE {
            dict[(row, col)] /= norm;
        }
    }

    let x: Array1<f64> = {
        let raw: Vec<f64> = (0..CROP_SIZE).map(|_| lcg_next_f64(&mut state)).collect();
        let norm = raw.iter().map(|v| v * v).sum::<f64>().sqrt().max(1e-12);
        Array1::from_iter(raw.iter().map(|v| v / norm))
    };

    c.bench_function("lars_fit", |b| {
        b.iter(|| {
            let model = Lars::new()
                .n_nonzero_coefs(black_box(NONZERO_COEFS))
                .fit(black_box(&dict), black_box(&x))
                .expect("LARS failed");
            black_box(model.coef().to_owned())
        })
    });
}

criterion_group!(benches, bench_fft, bench_lars);
criterion_main!(benches);