import sounddevice as sd
import numpy as np
import matplotlib.pyplot as plt

DEVICE_ID = 1
RATE = 44100
CHUNK = 1024

fig, ax = plt.subplots()
x = np.arange(0, CHUNK) / RATE
line, = ax.plot(x, np.zeros(CHUNK))
ax.set_ylim(-1, 1)
ax.set_xlim(0, CHUNK / RATE)
ax.set_title("live waveform")
ax.set_xlabel("time (s)")
ax.set_ylabel("amplitude")
ax.grid(True, alpha=0.3)

with sd.InputStream(device=DEVICE_ID, blocksize=CHUNK, samplerate=RATE, channels=1) as stream:
    try:
        while plt.fignum_exists(fig.number):
            data, overflow = stream.read(CHUNK)
            if overflow:
                print("Overflow")
            
            audio_buffer = data[:, 0]
            line.set_ydata(audio_buffer)
            
            plt.pause(0.02)
    except KeyboardInterrupt:
        pass