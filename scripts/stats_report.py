#!/usr/bin/env python

import sys
import json
import statistics
import numpy
import matplotlib.pyplot


def main():
    raw = [json.loads(v) for v in sys.stdin]
    records = [v for v in raw if isinstance(v, dict)]
    result = [v for v in raw if isinstance(v, list)]
    values = {k: [v.get(k) for v in records if v.get(k) is not None] for k in records[0]}
    values['time_to_score'] = [v for v in values['time_to_score']]
    values['jump_simulation'] = [v for v in values['jump_simulation']]
    row('', 'n', 'sum', 'q95', 'min', 'max', 'mean', 'median', 'stdev')
    for k, v in values.items():
        if not v or not isinstance(v[0], (int, float)):
            continue
        row(
            k,
            len(v),
            sum(v),
            numpy.quantile(v, 0.95) if v else float('NaN'),
            min(v) if v else float('NaN'),
            max(v) if v else float('NaN'),
            statistics.mean(v) if v else float('NaN'),
            statistics.median(v) if v else float('NaN'),
            statistics.stdev(v) if v else float('NaN'),
        )
    print()
    if result:
        row(*sorted(result[0][0].keys()))
        for player in result[0]:
            row(*[player[k] for k in sorted(result[0][0].keys())])
    print()
    for k, v in values.items():
        if not v or not isinstance(v[0], (int, float)):
            continue
        fig, ax = matplotlib.pyplot.subplots()
        fig.canvas.set_window_title(k)
        if v and k in ('iteration', 'step', 'total_iterations'):
            ax.hist(v, bins=numpy.arange(min(v), max(v) + 1, 1))
            ax.set_xticks(numpy.arange(min(v), max(v) + 1, 1))
        else:
            ax.hist(v, bins='auto')
        ax.grid(True)
    matplotlib.pyplot.show()


def row(*args):
    print(('{:>20}' * len(args)).format(*args))


if __name__ == '__main__':
    main()
