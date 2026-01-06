import sounddevice as sd
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
import librosa

DEVICE_ID = 1
samplerate = 44100
blocksize = 2048

audio_buffer = np.zeros(blocksize)
pitch_history_pyin = []
pitch_history_yin = []

def audio_callback(indata, frames, time, status):
    global audio_buffer
    audio_buffer[:] = indata[:, 0]

fig, axes = plt.subplots(1, 2, figsize=(14, 5))

line_wave, = axes[0].plot(np.zeros(blocksize))
axes[0].set_title("waveform")
axes[0].set_xlabel("time")
axes[0].set_ylabel("amplitude")
axes[0].set_ylim(-1, 1)
axes[0].grid(True, alpha=0.3)

line_pyin, = axes[1].plot([], [], color='green', marker='o', label='pYIN')
line_yin, = axes[1].plot([], [], color='orange', marker='s', label='YIN')
axes[1].set_title("pitch detection (pYIN vs YIN)")
axes[1].set_xlabel("frame")
axes[1].set_ylabel("freq (Hz)")
axes[1].set_yscale('log') 
axes[1].set_ylim(20, 4200)
axes[1].legend()
axes[1].grid(True, alpha=0.3)

def update_plot(frame):
    global audio_buffer, pitch_history_pyin, pitch_history_yin
    
    # just the waveform
    line_wave.set_ydata(audio_buffer)
    
    # for PYIN
    f0_pyin, _, _ = librosa.pyin(audio_buffer, fmin=27, fmax=4186, sr=samplerate)
    pitch_pyin = np.nanmean(f0_pyin[~np.isnan(f0_pyin)]) if np.any(~np.isnan(f0_pyin)) else 0
    # pitch_pyin = f0_pyin

    pitch_history_pyin.append(pitch_pyin)
    if len(pitch_history_pyin) > 50:
        pitch_history_pyin.pop(0)
    
    line_pyin.set_data(range(len(pitch_history_pyin)), pitch_history_pyin)
    
    # for YIN
    yin_result = librosa.yin(audio_buffer, fmin=27, fmax=4186, sr=samplerate)
    f0_yin = yin_result[0]
    pitch_yin = np.nanmean(f0_yin[~np.isnan(f0_yin)]) if np.any(~np.isnan(f0_yin)) else 0
    # pitch_yin = f0_yin

    pitch_history_yin.append(pitch_yin)
    if len(pitch_history_yin) > 50:
        pitch_history_yin.pop(0)
    
    line_yin.set_data(range(len(pitch_history_yin)), pitch_history_yin)

    
    axes[1].set_xlim(0, 50)
    
    return line_wave, line_pyin, line_yin

stream = sd.InputStream(device=DEVICE_ID, callback=audio_callback, blocksize=blocksize, samplerate=samplerate, channels=1)
with stream:
    ani = FuncAnimation(fig, update_plot, interval=100, blit=False)
    plt.show()