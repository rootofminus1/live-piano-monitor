import sounddevice as sd
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from scipy import signal
import soundfile as sf

DEVICE_ID = 1
samplerate = 44100
blocksize = 1024

reference_audio, sr_ref = sf.read("frame_751.wav")

reference_norm = reference_audio / (np.max(np.abs(reference_audio)) + 1e-10)

audio_buffer = np.zeros(blocksize)
dot_product_history = np.zeros(300)

def audio_callback(indata, frames, time, status):
    global audio_buffer
    audio_buffer = indata[:, 0]

fig, (ax_wave, ax_dot) = plt.subplots(2, 1, figsize=(14, 10))

x = np.arange(0, blocksize) / samplerate
line_wave, = ax_wave.plot(x, np.zeros(blocksize), color='blue')
ax_wave.set_ylim(-1, 1)
ax_wave.set_xlim(0, blocksize / samplerate)
ax_wave.set_title("live waveform")
ax_wave.set_xlabel("time (s)")
ax_wave.set_ylabel("amplitude")
ax_wave.grid(True, alpha=0.3)

line_dot, = ax_dot.plot([], [], color='orange', linewidth=1)
ax_dot.set_title("dot product over time")
ax_dot.set_xlabel("time")
ax_dot.set_ylabel("dot product")
ax_dot.set_ylim(-1, 1)
ax_dot.grid(True, alpha=0.3)

def update_plot(frame):
    global audio_buffer, dot_product_history
    
    line_wave.set_ydata(audio_buffer)
    
    live_norm = audio_buffer / (np.max(np.abs(audio_buffer)) + 1e-10)
    
    ref_to_use = reference_norm[:len(live_norm)]
    dot_prod = np.dot(live_norm, ref_to_use) / len(live_norm)

    dot_product_history = np.roll(dot_product_history, -1)
    dot_product_history[-1] = dot_prod
    
    line_dot.set_data(np.arange(len(dot_product_history)), dot_product_history)
    ax_dot.set_xlim(0, len(dot_product_history))
    
    return line_wave, line_dot

stream = sd.InputStream(device=DEVICE_ID, callback=audio_callback, blocksize=blocksize, samplerate=samplerate, channels=1)
with stream:
    ani = FuncAnimation(fig, update_plot, interval=30, blit=True)
    plt.show()