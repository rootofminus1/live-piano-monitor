import numpy as np
import sounddevice as sd
import matplotlib
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from scipy.interpolate import interp1d
import scipy.signal


matplotlib.use("Qt5Agg")

DEVICE_ID = 13       
BLOCKSIZE = 4096
MIN_FREQ = 60.0
MAX_FREQ = 16000.0
ALPHA = 1.5  # to be changed, varies from lab to real life
MAX_NOTES = 3



dev = sd.query_devices(DEVICE_ID, kind="input")
assert isinstance(dev, dict)
SAMPLERATE = int(dev["default_samplerate"])

hanning = np.hanning(BLOCKSIZE)
ALL_FREQS = np.fft.rfftfreq(BLOCKSIZE, 1.0 / SAMPLERATE)[1:]


_spectrum_buffer = np.zeros(len(ALL_FREQS))


def bandpass(freqs: np.ndarray, spectrum: np.ndarray):
    mask = (freqs >= MIN_FREQ) & (freqs <= MAX_FREQ)
    return freqs[mask], spectrum[mask]


def find_peaks(spectrum: np.ndarray) -> list[int]:
    if np.max(spectrum) == 0:
        return []
        
    height = 0.05 * np.max(spectrum)
    peaks, _ = scipy.signal.find_peaks(spectrum, height=height, distance=3)
    return list(peaks)


def harmonic_indices(fundamental_idx: int, freqs: np.ndarray) -> list[int]:
    f0 = freqs[fundamental_idx]
    indices = []
    n = 2

    while f0 * n <= MAX_FREQ:
        target = f0 * n
        idx = int(np.argmin(np.abs(freqs - target)))
        indices.append(idx)
        n += 1

    return indices



def notes_reduce(freqs: np.ndarray, spectrum: np.ndarray, fundamental_idx: int) -> np.ndarray:
    print(freqs, spectrum)

    spectrum = spectrum.copy()
    h_indices = harmonic_indices(fundamental_idx, freqs)

    if not h_indices:
        return spectrum

    points_x = []
    points_y = []
    last_amp = np.inf

    for idx in h_indices:
        amp = spectrum[idx]
        if amp < last_amp:
            points_x.append(freqs[idx])
            points_y.append(amp)
            last_amp = amp

    if not points_x:
        return spectrum

    points_x.append(MAX_FREQ)
    points_y.append(0.0)

    spline = interp1d(
        points_x, points_y,
        kind="linear",
        bounds_error=False,
        fill_value=0.0,
    )

    for idx in h_indices:
        reduction = float(spline(freqs[idx]))
        spectrum[idx] = max(0.0, spectrum[idx] - reduction)

    return spectrum


def rake_pitch_detection(freqs: np.ndarray, spectrum: np.ndarray) -> list[float]:
    print("rake")
    detected: list[float] = []

    freqs, spectrum = bandpass(freqs, spectrum)

    if len(spectrum) == 0 or np.max(spectrum) == 0:
        return detected

    spectrum = spectrum / np.max(spectrum)
    peaks = find_peaks(spectrum)

    while peaks and len(detected) < MAX_NOTES:
        print("peaks")
        peaks.sort()
        candidate_idx = peaks.pop(0)

        remaining_amps = [spectrum[i] for i in peaks]
        mu = np.mean(remaining_amps) if remaining_amps else 0.0
        threshold = ALPHA * mu

        if spectrum[candidate_idx] < threshold:
            continue

        detected.append(float(freqs[candidate_idx]))

        spectrum = notes_reduce(freqs, spectrum, candidate_idx)

        peak_val = np.max(spectrum)
        if peak_val > 0:
            spectrum = spectrum / peak_val

        peaks = find_peaks(spectrum)

    return detected


def audio_callback(indata, frames, time, status):
    if status:
        print("audio status:", status)
    global _spectrum_buffer
    windowed = indata[:, 0] * hanning
    fft_out = np.fft.rfft(windowed)
    _spectrum_buffer[:] = np.abs(fft_out[1:])



LABEL_COLORS = ["red", "darkorange", "green"]

fig, ax = plt.subplots(figsize=(13, 5))

(spectrum_line,) = ax.semilogx(ALL_FREQS, np.zeros_like(ALL_FREQS), color="steelblue", lw=1.2, label="Spectrum")
ax.set_xlim(MIN_FREQ, MAX_FREQ)
ax.set_xlabel("frequency (Hz)")
ax.set_ylabel("magnitude (normalised)")
ax.grid(True, which="both", alpha=0.25)



pitch_lines = [
    ax.axvline(100, color=LABEL_COLORS[k], alpha=0.85, lw=2.0, visible=False)
    for k in range(MAX_NOTES)
]
pitch_labels = [
    ax.text(
        100, 1.02, "",
        color=LABEL_COLORS[k],
        fontsize=9, fontweight="bold",
        ha="center", va="bottom",
        visible=False,
    )
    for k in range(MAX_NOTES)
]

ax.legend(loc="upper right", fontsize=8)


def freq_to_note_name(freq: float) -> str:
    if freq <= 0:
        return ""

    midi = 69 + 12 * np.log2(freq / 440.0)
    note = int(round(midi))
    names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"]
    octave = note // 12 - 1
    name = names[note % 12]
    return f"{name}{octave}"


def update_plot(frame):
    spectrum = _spectrum_buffer.copy()

    if np.max(spectrum) > 0:
        display_spectrum = spectrum / np.max(spectrum)
    else:
        display_spectrum = spectrum

    spectrum_line.set_ydata(display_spectrum)

    detected = rake_pitch_detection(ALL_FREQS, spectrum.copy())

    for pl, lbl in zip(pitch_lines, pitch_labels):
        pl.set_visible(False)
        lbl.set_visible(False)

    for pl, lbl, f in zip(pitch_lines, pitch_labels, detected[:MAX_NOTES]):
        note_name = freq_to_note_name(f)

        pl.set_xdata([f, f])
        pl.set_visible(True)

        lbl.set_x(f)
        lbl.set_text(f"{note_name}\n{f:.0f} Hz")
        lbl.set_visible(True)

    return [spectrum_line, *pitch_lines, *pitch_labels]


if __name__ == "__main__":
    print(f"audio device: {dev['name']}")
    print(f"sample rate: {SAMPLERATE} Hz  |  block size: {BLOCKSIZE}  |  alpha = {ALPHA}")

    with sd.InputStream(
        device=DEVICE_ID,
        callback=audio_callback,
        blocksize=BLOCKSIZE,
        samplerate=SAMPLERATE,
        channels=1,
    ):
        ani = FuncAnimation(
            fig,
            update_plot,
            interval=40, # 25 fps? should be?
            blit=False,
            cache_frame_data=False,
        )
        plt.tight_layout()
        plt.show()