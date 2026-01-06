import sounddevice as sd
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
import librosa

DEVICE_ID = 2
samplerate = 44100
blocksize = 2048

spectrum_buffer = np.zeros(blocksize // 2 + 1)
mel_fb = librosa.filters.mel(sr=samplerate, n_fft=blocksize, n_mels=128, fmin=20, fmax=20000)

def audio_callback(indata, frames, time, status):
    global spectrum_buffer
    spectrum_buffer[:] = np.abs(np.fft.rfft(indata[:, 0]))

fig, axes = plt.subplots(2, 2, figsize=(14, 10))
x = np.fft.rfftfreq(blocksize, 1 / samplerate)

# 1. Normal (absolute magnitude)
line1, = axes[0, 0].semilogx(x, np.zeros_like(x), color='blue')
axes[0, 0].set_xlim(20, 20000)
axes[0, 0].set_ylim(0, 0.1)
axes[0, 0].set_title("FFT (Normal / Absolute)")
axes[0, 0].set_xlabel("Frequency (Hz)")
axes[0, 0].set_ylabel("Magnitude")
axes[0, 0].grid(True, which='both', alpha=0.3)

# 2. Normalized (0-1 by max)
line2, = axes[0, 1].semilogx(x, np.zeros_like(x), color='green')
axes[0, 1].set_xlim(20, 20000)
axes[0, 1].set_ylim(0, 1)
axes[0, 1].set_title("FFT (Normalized)")
axes[0, 1].set_xlabel("Frequency (Hz)")
axes[0, 1].set_ylabel("Magnitude (normalized)")
axes[0, 1].grid(True, which='both', alpha=0.3)

# 3. Log (dB)
line3, = axes[1, 0].semilogx(x, np.zeros_like(x), color='red')
axes[1, 0].set_xlim(20, 20000)
axes[1, 0].set_ylim(-80, 0)
axes[1, 0].set_title("FFT (Log / dB)")
axes[1, 0].set_xlabel("Frequency (Hz)")
axes[1, 0].set_ylabel("Magnitude (dB)")
axes[1, 0].grid(True, which='both', alpha=0.3)

# 4. Mel spectrum
mel_freqs = librosa.mel_frequencies(n_mels=128, fmin=20, fmax=20000)
line4, = axes[1, 1].semilogx(mel_freqs, np.zeros(128), color='purple')
axes[1, 1].set_xlim(20, 20000)
axes[1, 1].set_ylim(-80, 0)
axes[1, 1].set_title("Mel Spectrum (dB)")
axes[1, 1].set_xlabel("Frequency (Hz)")
axes[1, 1].set_ylabel("Magnitude (dB)")
axes[1, 1].grid(True, which='both', alpha=0.3)

def update_plot(frame):
    global spectrum_buffer
    
    line1.set_ydata(spectrum_buffer)
    
    maxv = np.max(spectrum_buffer)
    normalized = spectrum_buffer / maxv if maxv > 0 else spectrum_buffer
    line2.set_ydata(normalized)
    
    spectrum_db = 20 * np.log10(np.maximum(spectrum_buffer, 1e-10))
    line3.set_ydata(spectrum_db)
    
    mel_spec = mel_fb @ spectrum_buffer
    mel_db = 20 * np.log10(np.maximum(mel_spec, 1e-10))
    line4.set_ydata(mel_db)
    
    return line1, line2, line3, line4

stream = sd.InputStream(device=DEVICE_ID, callback=audio_callback, blocksize=blocksize, samplerate=samplerate, channels=1)
with stream:
    ani = FuncAnimation(fig, update_plot, interval=30, blit=True)
    plt.show()