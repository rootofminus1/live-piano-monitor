import sounddevice as sd
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
import matplotlib
matplotlib.use("TkAgg")

DEVICE_ID = 13
dev = sd.query_devices(DEVICE_ID)
assert isinstance(dev, dict)
RATE = int(dev["default_samplerate"])

CHUNK = 1024
audio_buffer = np.zeros(CHUNK)

prev_magnitude = np.zeros(CHUNK // 2 + 1)
flux_history = []
HISTORY_SIZE = 50
onset_threshold_multiplier = 1.5

def audio_callback(indata, frames, time, status):
    global audio_buffer
    audio_buffer = indata[:, 0]

fig, (ax1, ax2) = plt.subplots(2, 1)


x = np.arange(0, CHUNK) / RATE
wave_line, = ax1.plot(x, np.zeros(CHUNK))
ax1.set_ylim(-1, 1)
ax1.set_title("live waveform")

flux_line, = ax2.plot([], [])
ax2.set_ylim(0, 200)
ax2.set_xlim(0, HISTORY_SIZE)
ax2.set_title("spectral flux")

def update_plot(frame):
    global prev_magnitude, flux_history

    wave_line.set_ydata(audio_buffer)

    windowed = audio_buffer * np.hanning(CHUNK)
    spectrum = np.fft.rfft(windowed)
    magnitude = np.abs(spectrum)

    diff = magnitude - prev_magnitude
    diff[diff < 0] = 0
    flux = np.sum(diff)

    prev_magnitude = magnitude

    flux_history.append(flux)
    if len(flux_history) > HISTORY_SIZE:
        flux_history.pop(0)

    if len(flux_history) >= 10:
        mean_flux = np.mean(flux_history)
        threshold = mean_flux * 1.5

        if flux > threshold and flux > 100:
            print("ONSET!")

    flux_line.set_data(range(len(flux_history)), flux_history)

    return wave_line, flux_line

stream = sd.InputStream(device=DEVICE_ID, callback=audio_callback, blocksize=CHUNK, samplerate=RATE, channels=1)

with stream:
    ani = FuncAnimation(fig, update_plot, interval=30, blit=True)
    plt.tight_layout()
    plt.show()