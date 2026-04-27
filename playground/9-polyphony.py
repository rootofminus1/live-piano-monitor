import numpy as np
import sounddevice as sd
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
import matplotlib
import scipy.signal
from scipy.interpolate import interp1d

matplotlib.use("Qt5Agg")
print(matplotlib.get_backend())



DEVICE_ID = 13
dev = sd.query_devices(DEVICE_ID)
assert isinstance(dev, dict)
samplerate = int(dev["default_samplerate"])
blocksize = 4096

MIN_FREQ = 60
MAX_FREQ = 16000
ALPHA = 1.1

window = np.hanning(blocksize)
freqs = np.fft.rfftfreq(blocksize, 1 / samplerate)

# log(0) = no
freqs = freqs[1:]

spectrum_buffer = np.zeros(blocksize // 2 + 1)




def bandpass_mask(freqs):
    return (freqs >= MIN_FREQ) & (freqs <= MAX_FREQ)

def find_peaks(spectrum):
    height = np.percentile(spectrum, 75)
    peaks, _ = scipy.signal.find_peaks(spectrum, height=height)
    # print(peaks)
    return peaks

def harmonics_from_frequency(freq, max_freq):
    harmonics = []
    multiple = 2
    while freq * multiple <= max_freq:
        harmonics.append(freq * multiple)
        multiple += 1
    return harmonics

def notes_reduce(freqs, spectrum, fundamental_idx):
    fundamental_freq = freqs[fundamental_idx]
    max_freq = freqs[-1]

    harmonic_freqs = harmonics_from_frequency(fundamental_freq, max_freq)

    harmonic_indices = [
        np.argmin(np.abs(freqs - hf)) for hf in harmonic_freqs
    ]

    # print(harmonic_freqs)

    points_x = []
    points_y = []
    last_amp = np.inf

    for idx in harmonic_indices:
        amp = spectrum[idx]
        if amp < last_amp:
            points_x.append(freqs[idx])
            points_y.append(amp)
            last_amp = amp

    if len(points_x) < 2:
        return spectrum

    points_x.append(max_freq)
    points_y.append(0)

    spline = interp1d(
        points_x,
        points_y,
        kind="linear",
        fill_value=0,
        bounds_error=False
    )

    # for idx in harmonic_indices:
    #     reduction = spline(freqs[idx])
    #     spectrum[idx] = max(0, spectrum[idx] - reduction)

    bandwidth = 3

    for idx in harmonic_indices:
        for b in range(-bandwidth, bandwidth + 1):
            j = idx + b
            if 0 <= j < len(spectrum):
                reduction = spline(freqs[j])
                spectrum[j] = max(0, spectrum[j] - reduction)


    return spectrum

def rake_pitch_detection(freqs, spectrum):
    print("rake")
    detected = []

    print("bandpass")
    mask = bandpass_mask(freqs)
    freqs_masked = freqs[mask]
    spectrum_masked = spectrum[mask]

    if np.max(spectrum_masked) == 0:
        return detected

    spectrum_masked = spectrum_masked / np.max(spectrum_masked)

    print("find_peaks")
    peaks = list(find_peaks(spectrum_masked))

    print(peaks)

    print("loop")

    max_iterations = 10
    iteration = 0

    while peaks and iteration < max_iterations:
        print("loooooping")
        # peaks.sort(key=lambda i: freqs_masked[i])
        peaks.sort(key=lambda i: spectrum_masked[i], reverse=True)
        idx = peaks.pop(0)

        remaining = [spectrum_masked[i] for i in peaks]
        mean_amp = np.mean(remaining) if remaining else 0
        threshold = ALPHA * mean_amp

        if spectrum_masked[idx] < threshold:
            # peaks.remove(idx)
            print("STUCK ON:", freqs_masked[idx], spectrum_masked[idx], threshold)
            break

        detected.append(freqs_masked[idx])

        spectrum_masked = notes_reduce(freqs_masked, spectrum_masked, idx)
        peaks = list(find_peaks(spectrum_masked))

        iteration += 1
    print("AFTER LOOP")

    return detected





def audio_callback(indata, frames, time, status):
    global spectrum_buffer
    if status:
        print(status)
    spectrum_buffer[:] = np.abs(
        np.fft.rfft(indata[:, 0] * window)
    )



fig, ax = plt.subplots()

line, = ax.semilogx(freqs, np.zeros_like(freqs))

ax.set_xlim(20, 20000)
ax.set_ylim(0, 1)
ax.set_xlabel("frequency (Hz)")
ax.set_ylabel("magnitude (normalized)")
ax.grid(True, which="both", alpha=0.3)

max_pitches = 10
pitch_lines = [
    ax.axvline(1, color="red", alpha=0.7, visible=False)
    for _ in range(max_pitches)
]





def update_plot(frame):
    print("spec")
    spectrum = spectrum_buffer.copy()

    print("spec")
    spectrum = spectrum[1:]

    print("spec")
    if np.max(spectrum) > 0:
        spectrum = spectrum / np.max(spectrum)

    print("spec")
    line.set_ydata(spectrum)

    print("BEFORE RAKE")
    detected = rake_pitch_detection(freqs, spectrum.copy())
    print("AFTER RAKE")

    for pl in pitch_lines:
        pl.set_visible(False)

    for pl, f in zip(pitch_lines, detected[:max_pitches]):
        pl.set_xdata([f, f])
        pl.set_visible(True)

    return [line, *pitch_lines]


with sd.InputStream(
    device=DEVICE_ID,
    callback=audio_callback,
    blocksize=blocksize,
    samplerate=samplerate,
    channels=1
):
    ani = FuncAnimation(
        fig,
        update_plot,
        interval=30,
        blit=False,
    )

    plt.tight_layout()
    plt.show()