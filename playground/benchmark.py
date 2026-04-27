import numpy as np
import timeit
from sklearn.linear_model import Lars

TOTAL_SIZE = 4096
CROP_SIZE = 256
N_ATOMS = 61 
NONZERO_COEFS = 6
RUNS = 1000


rng = np.random.default_rng(42)

signal = rng.standard_normal(TOTAL_SIZE).astype(np.float32)
window = np.hanning(TOTAL_SIZE).astype(np.float32)

def bench_fft():
    windowed = signal * window
    spectrum = np.fft.rfft(windowed)
    mag = np.abs(spectrum[:CROP_SIZE])
    norm = np.linalg.norm(mag)
    if norm > 1e-12:
        mag /= norm


fft_time = timeit.timeit(bench_fft, number=RUNS)
print(f"FFT ({RUNS} runs): total={fft_time*1000:.2f}ms  avg={fft_time/RUNS*1000:.4f}ms")

dictionary = rng.standard_normal((CROP_SIZE, N_ATOMS)).astype(np.float64)

norms = np.linalg.norm(dictionary, axis=0, keepdims=True)
norms[norms < 1e-12] = 1.0
dictionary /= norms

x = rng.standard_normal(CROP_SIZE).astype(np.float64)
x /= np.linalg.norm(x)

def bench_lars():
    model = Lars(n_nonzero_coefs=NONZERO_COEFS, fit_intercept=False)
    model.fit(dictionary, x)


lars_time = timeit.timeit(bench_lars, number=RUNS)
print(f"LARS ({RUNS} runs): total={lars_time*1000:.2f}ms  avg={lars_time/RUNS*1000:.4f}ms")