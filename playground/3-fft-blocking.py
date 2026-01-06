import sounddevice as sd
import numpy as np
import matplotlib.pyplot as plt

DEVICE_ID = 1

samplerate = 44100
blocksize = 2048

fig, ax = plt.subplots()
x = np.fft.rfftfreq(blocksize, 1 / samplerate)
line, = ax.semilogx(x, np.zeros_like(x))
ax.set_xlim(20, 20000)
ax.set_ylim(0, 1)
ax.set_xlabel("frequency (Hz)")
ax.set_ylabel("magnitude (absolute)")
ax.grid(True, which='both', alpha=0.3)

with sd.InputStream(device=DEVICE_ID, channels=1, blocksize=blocksize, samplerate=samplerate) as stream:
    try:
        while plt.fignum_exists(fig.number):
            data, overflow = stream.read(blocksize)
            if overflow:
                print("overflew")
            spectrum = np.abs(np.fft.rfft(data[:, 0]))

            line.set_ydata(spectrum)
            plt.pause(0.02)
    except KeyboardInterrupt:
        pass