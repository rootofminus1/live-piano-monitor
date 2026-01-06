import sounddevice as sd
import numpy as np
import pyqtgraph as pg
from pyqtgraph.Qt import QtCore, QtWidgets

samplerate = 44100
blocksize = 1024

app = QtWidgets.QApplication([])
win = pg.GraphicsLayoutWidget(show=True, title="live waveform")
plot = win.addPlot()
curve = plot.plot(pen='y')
plot.setYRange(-1, 1)

def update():
    data, _ = sd.rec(blocksize, samplerate=samplerate, channels=1, dtype='float32', blocking=True, device=2), None
    curve.setData(data.flatten())

timer = QtCore.QTimer()
timer.timeout.connect(update)
timer.start(30)

app.exec()
