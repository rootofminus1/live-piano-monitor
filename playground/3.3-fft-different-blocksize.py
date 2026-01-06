import sounddevice as sd
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation

DEVICE_ID = 1
samplerate = 44100
blocksize1 = 1024
blocksize2 = 4096

spectrum_buffer1 = np.zeros(blocksize1 // 2 + 1)
spectrum_buffer2 = np.zeros(blocksize2 // 2 + 1)

audio_accumulator = np.zeros(blocksize2)
accumulator_idx = 0

def audio_callback(indata, frames, time, status):
    global spectrum_buffer1, spectrum_buffer2, audio_accumulator, accumulator_idx
    
    spectrum_buffer1[:] = np.abs(np.fft.rfft(indata[:blocksize1, 0]))
    
    chunk_size = len(indata)
    remaining = blocksize2 - accumulator_idx
    
    if chunk_size <= remaining:
        audio_accumulator[accumulator_idx:accumulator_idx + chunk_size] = indata[:, 0]
        accumulator_idx += chunk_size
    else:
        audio_accumulator[accumulator_idx:] = indata[:remaining, 0]
        spectrum_buffer2[:] = np.abs(np.fft.rfft(audio_accumulator))
        accumulator_idx = 0

fig, axes = plt.subplots(1, 2, figsize=(14, 5))

x1 = np.fft.rfftfreq(blocksize1, 1 / samplerate)
line1, = axes[0].semilogx(x1, np.zeros_like(x1))
axes[0].set_xlim(20, 20000)
axes[0].set_ylim(0, 1)
axes[0].set_title("FFT (blocksize 1024, ~23ms)")
axes[0].set_xlabel("frequency (Hz)")
axes[0].set_ylabel("magnitude (absolute)")
axes[0].grid(True, which='both', alpha=0.3)

x2 = np.fft.rfftfreq(blocksize2, 1 / samplerate)
line2, = axes[1].semilogx(x2, np.zeros_like(x2))
axes[1].set_xlim(20, 20000)
axes[1].set_ylim(0, 1)
axes[1].set_title("FFT (blocksize 4096, ~93ms)")
axes[1].set_xlabel("frequency (Hz)")
axes[1].set_ylabel("magnitude (absolute)")
axes[1].grid(True, which='both', alpha=0.3)

def update_plot(frame):
    line1.set_ydata(spectrum_buffer1)
    line2.set_ydata(spectrum_buffer2)
    return line1, line2

stream = sd.InputStream(device=DEVICE_ID, callback=audio_callback, blocksize=blocksize1, samplerate=samplerate, channels=1)
with stream:
    ani = FuncAnimation(fig, update_plot, interval=23, blit=True)
    plt.show()