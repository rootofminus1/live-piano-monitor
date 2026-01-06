import sounddevice as sd
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
import librosa

DEVICE_ID = 1

samplerate = 44100
blocksize = 2048
n_mels = 128

spectrum_buffer = np.zeros(blocksize // 2 + 1)
mel_fb = librosa.filters.mel(sr=samplerate, n_fft=blocksize, n_mels=n_mels, fmin=20, fmax=20000)

def audio_callback(indata, frames, time, status):
    global spectrum_buffer
    spectrum_buffer[:] = np.abs(np.fft.rfft(indata[:, 0]))

fig, ax = plt.subplots()
mel_freqs = librosa.mel_frequencies(n_mels=n_mels, fmin=20, fmax=20000)
line, = ax.semilogx(mel_freqs, np.zeros(n_mels), color='purple')
ax.set_xlim(20, 20000)
ax.set_ylim(-100, 10)
ax.set_title("mel spec")
ax.set_xlabel("frequency (Hz)")
ax.set_ylabel("magnitude (dB)")
ax.grid(True, which='both', alpha=0.3)

def update_plot(frame):
    global spectrum_buffer

    mel_spec = mel_fb @ spectrum_buffer  # applying the whole mel transfromation

    mel_db = 20 * np.log10(np.maximum(mel_spec, 1e-10))  # to decibels
    
    line.set_ydata(mel_db)
    return line,

stream = sd.InputStream(device=DEVICE_ID, callback=audio_callback, blocksize=blocksize, samplerate=samplerate, channels=1)
with stream:
    ani = FuncAnimation(fig, update_plot, interval=30, blit=True)
    plt.show()