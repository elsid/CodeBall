#!/usr/bin/env python3

import sys
import json
import statistics
import matplotlib.pyplot

data = [json.loads(line) for line in sys.stdin]
keys = set(w['name'] for v in data for w in v)

for key in keys:
    values = [w['duration'] for v in data for w in v if w['name'] == key]
    figure, axis = matplotlib.pyplot.subplots()
    figure.canvas.set_window_title(key)
    axis.hist(values, bins='auto')
    axis.axvline(statistics.mean(values), color='green')
    axis.axvline(statistics.median(values), color='red')
    axis.grid(True)

matplotlib.pyplot.show()
