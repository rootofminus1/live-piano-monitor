import sounddevice as sd
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
import sounddevice as sd
import matplotlib
matplotlib.use("TkAgg")

print(matplotlib.get_backend())

DEVICE_ID = 13
dev = sd.query_devices(DEVICE_ID)
assert isinstance(dev, dict)
RATE = int(dev["default_samplerate"])



# RATE = 44100  # CD quality standard, also to capture up to 20k we should use 2x that, so around 40k
CHUNK = 1024  # powers of 2 are fast for FFT 
# higher CHUNK (2048, 4096) = more latency, more frequency detail
# lower CHUNK (256, 512) = less latency, less frequency detail 

audio_buffer = np.zeros(CHUNK)

def audio_callback(indata, frames, time, status):
    global audio_buffer
    audio_buffer = indata[:, 0]

fig, ax = plt.subplots()
x = np.arange(0, CHUNK) / RATE
line, = ax.plot(x, np.zeros(CHUNK))
ax.set_ylim(-1, 1)
ax.set_xlim(0, CHUNK / RATE)
ax.set_title("live waveform")
ax.set_xlabel("time (s)")
ax.set_ylabel("amplitude")
ax.grid(True, alpha=0.3)


def update_plot(frame):
    line.set_ydata(audio_buffer)
    return line,

stream = sd.InputStream(device=DEVICE_ID, callback=audio_callback, blocksize=CHUNK, samplerate=RATE, channels=1)
with stream:
    ani = FuncAnimation(fig, update_plot, interval=30, blit=True)
    plt.show()
