import sounddevice as sd
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation

DEVICE_ID = 1
samplerate = 44100
blocksize = 1024

# spectrum_buffer = np.zeros(blocksize)
spectrum_buffer = np.zeros(blocksize // 2 + 1)

def audio_callback(indata, frames, time, status):
    global spectrum_buffer
    spectrum_buffer[:] = np.abs(np.fft.rfft(indata[:, 0]))

fig, ax = plt.subplots()
x = np.fft.rfftfreq(blocksize, 1 / samplerate)
line, = ax.semilogx(x, np.zeros_like(x))
ax.set_xlim(20, 20000)
ax.set_ylim(0, 1)
ax.set_xlabel("frequency (Hz)")
ax.set_ylabel("magnitude (absolute)")
ax.grid(True, which='both', alpha=0.3)

def update_plot(frame):
    line.set_ydata(spectrum_buffer)
    return line,

stream = sd.InputStream(device=DEVICE_ID, callback=audio_callback, blocksize=blocksize, samplerate=samplerate, channels=1)
with stream:
    ani = FuncAnimation(fig, update_plot, interval=30, blit=True)
    plt.show()