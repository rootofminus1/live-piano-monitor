import sounddevice as sd
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
import librosa

import matplotlib
matplotlib.use("Qt5Agg")
print(matplotlib.get_backend())

DEVICE_ID = 16

dev = sd.query_devices(DEVICE_ID)
assert isinstance(dev, dict)
RATE = int(dev["default_samplerate"])
samplerate = RATE # 44100
blocksize = 2048


# DEVICE_ID = 1
# samplerate = 44100
# blocksize = 1024

n_bins = 88  # 88 because 88 piano keys
fmin = 27.5  # lowest piano key freq (damnit low freqs are quite a pain)

cqt_buffer = np.zeros(n_bins)
fft_buffer = np.zeros(blocksize // 2 + 1)

def audio_callback(indata, frames, time, status):
    global cqt_buffer, fft_buffer
    # CQT
    cqt = np.abs(librosa.cqt(indata[:, 0], sr=samplerate, fmin=fmin, n_bins=n_bins))
    cqt_buffer[:] = np.mean(cqt, axis=1)
    # FFT
    fft_buffer[:] = np.abs(np.fft.rfft(indata[:, 0]))

fig, (ax_fft, ax_cqt) = plt.subplots(1, 2, figsize=(14, 5))

fft_freqs = np.fft.rfftfreq(blocksize, 1 / samplerate)
line_fft, = ax_fft.semilogx(fft_freqs, np.zeros_like(fft_freqs), color='blue')
ax_fft.set_xlim(20, 20000)
ax_fft.set_ylim(0, 1)
ax_fft.set_xlabel("frequency (Hz)")
ax_fft.set_ylabel("magnitude")
ax_fft.set_title("FFT")
ax_fft.grid(True, which='both', alpha=0.3)


cqt_freqs = librosa.cqt_frequencies(n_bins=n_bins, fmin=fmin)
line_cqt, = ax_cqt.semilogx(cqt_freqs, np.zeros_like(cqt_freqs), color='orange')
ax_cqt.set_xlim(20, 4186)
ax_cqt.set_ylim(0, 1)
ax_cqt.set_xlabel("frequency (Hz)")
ax_cqt.set_ylabel("magnitude")
ax_cqt.set_title("CQT")
ax_cqt.grid(True, which='both', alpha=0.3)

def update_plot(frame):
    # line_fft.set_ydata(fft_buffer / np.max(fft_buffer + 1e-10))
    # line_cqt.set_ydata(cqt_buffer / np.max(cqt_buffer + 1e-10))
    line_fft.set_ydata(fft_buffer)
    line_cqt.set_ydata(cqt_buffer)
    return line_fft, line_cqt

stream = sd.InputStream(device=DEVICE_ID, callback=audio_callback, blocksize=blocksize, samplerate=samplerate, channels=1)
with stream:
    ani = FuncAnimation(fig, update_plot, interval=30, blit=True)
    plt.show()