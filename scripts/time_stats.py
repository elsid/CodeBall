#!/usr/bin/env python3

import sys
import statistics
import matplotlib.pyplot

data = [tuple(float(v) for v in line.split(' ')) for line in sys.stdin]
times = ('times', [v[0] for v in data])
ticks = ('ticks', [v[1] for v in data])
ticks_per_ms = ('ticks per ms', [v[1] / v[0] for v in data])

for title, values in (times, ticks, ticks_per_ms):
    figure, axis = matplotlib.pyplot.subplots()
    figure.canvas.set_window_title(title)
    axis.hist(values, bins='auto')
    axis.axvline(statistics.mean(values), color='green')
    axis.axvline(statistics.median(values), color='red')
    axis.grid(True)
    print(title, sum(values), statistics.mean(values), statistics.median(values))

matplotlib.pyplot.show()
