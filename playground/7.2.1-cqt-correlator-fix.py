import sounddevice as sd
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
import librosa
import soundfile as sf

DEVICE_ID = 1
samplerate = 44100
blocksize = 2048

n_bins = 88  # 88 because 88 piano keys
fmin = 27.5  # lowest piano key freq (damnit low freqs are quite a pain)
bins_per_octave = 12


template_audio, sr = sf.read("frame_751.wav")

# the cqt template
template_cqt = librosa.cqt(
    template_audio,
    sr=samplerate,
    hop_length=blocksize,
    n_bins=n_bins,
    bins_per_octave=bins_per_octave
)
template_mag = np.abs(template_cqt)

template_vec = np.mean(template_mag, axis=1)
template_vec /= (np.linalg.norm(template_vec) + 1e-12)  # 1e-12 silly fix for division by 0 



audio_buffer = np.zeros(blocksize)
dot_history = np.zeros(400)

def audio_callback(indata, frames, time, status):
    global audio_buffer
    audio_buffer = indata[:, 0].copy()


fig, (ax_wave, ax_cqt, ax_dot) = plt.subplots(3, 1, figsize=(12, 12))

x = np.arange(0, blocksize) / samplerate
line_wave, = ax_wave.plot(x, np.zeros(blocksize), color='blue')
ax_wave.set_ylim(-1, 1)
ax_wave.set_xlim(0, blocksize / samplerate)
ax_wave.set_title("live waveform")
ax_wave.set_xlabel("time (s)")
ax_wave.set_ylabel("amplitude")
ax_wave.grid(True, alpha=0.3)

cqt_freqs = librosa.cqt_frequencies(n_bins=n_bins, fmin=27.5)
line_live_cqt, = ax_cqt.semilogx(cqt_freqs, np.zeros(n_bins), color='orange', label='live', linewidth=1.5)
line_tpl_cqt, = ax_cqt.semilogx(cqt_freqs, template_vec, color='blue', label='template', linewidth=1.5, alpha=0.7)
ax_cqt.set_xlim(20, 4186)
ax_cqt.set_ylim(0, 1)
ax_cqt.set_xlabel("frequency (Hz)")
ax_cqt.set_ylabel("magnitude")
ax_cqt.set_title("CQT")
ax_cqt.legend()
ax_cqt.grid(True, which='both', alpha=0.3)

line_dot, = ax_dot.plot([], [], color='orange', linewidth=1)
ax_dot.set_ylim(-1, 1)
ax_dot.set_xlim(0, len(dot_history))
ax_dot.set_title("correlation")
ax_dot.set_xlabel("time")
ax_dot.set_ylabel("dot product")
ax_dot.grid(True, alpha=0.3)


def update_plot(frame):
    global audio_buffer, dot_history

    # for the waveform
    line_wave.set_ydata(audio_buffer)

    # for the CQT
    live_cqt = librosa.cqt(
        audio_buffer,
        sr=samplerate,
        hop_length=blocksize,
        n_bins=n_bins,
        bins_per_octave=12
    )
    live_mag = np.abs(live_cqt)
    live_vec = live_mag.mean(axis=1) if live_mag.ndim > 1 else live_mag
    live_vec /= (np.linalg.norm(live_vec) + 1e-12)

    line_live_cqt.set_ydata(live_vec)

    # for the dot product
    score = np.dot(live_vec, template_vec)

    dot_history[:-1] = dot_history[1:]
    dot_history[-1] = score
    line_dot.set_data(np.arange(len(dot_history)), dot_history)

    return line_wave, line_live_cqt, line_tpl_cqt, line_dot



stream = sd.InputStream(
    device=DEVICE_ID,
    callback=audio_callback,
    blocksize=blocksize,
    samplerate=samplerate,
    channels=1
)

with stream:
    ani = FuncAnimation(fig, update_plot, interval=30, blit=True)
    plt.show()