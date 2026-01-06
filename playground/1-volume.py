import sounddevice as sd
import numpy as np

DEVICE_ID = 2

def callback(indata, frames, time, status):
    volume_norm = np.linalg.norm(indata) * 10
    print("|" * int(volume_norm))

with sd.InputStream(callback=callback, device=DEVICE_ID):
    sd.sleep(50000)
