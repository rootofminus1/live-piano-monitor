import sounddevice as sd
import numpy as np

fs = 44100
duration = 5.0 
frequency = 440 

t = np.linspace(0, duration, int(fs * duration), endpoint=False)
wave = 0.5 * np.sin(2 * np.pi * frequency * t)

sd.play(wave, samplerate=fs)
sd.wait()
