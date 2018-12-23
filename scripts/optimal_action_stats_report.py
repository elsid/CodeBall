#!/usr/bin/env python

import sys
import json
import statistics
import numpy
import matplotlib.pyplot


def row(*args):
    print(('{:>20}' * len(args)).format(*args))


def main():
    raw = [json.loads(v) for v in sys.stdin]
    records = [v for v in raw if isinstance(v, dict)]
    result = [v for v in raw if isinstance(v, list)]
    values = {k: [v.get(k) for v in records if v.get(k) is not None] for k in records[0]}
    values['time_to_score'] = [v for v in values['time_to_score'] if v < 2]
    values['jump_simulation'] = [v for v in values['jump_simulation'] if v]
    row('', 'n', 'q95', 'min', 'max', 'mean', 'median', 'stdev')
    for k, v in values.items():
        row(
            k,
            len(v),
            numpy.quantile(v, 0.95),
            min(v),
            max(v),
            statistics.mean(v),
            statistics.median(v),
            statistics.stdev(v),
        )
    print()
    if result:
        row(*sorted(result[0][0].keys()))
        for player in result[0]:
            row(*[player[k] for k in sorted(result[0][0].keys())])
    print()
    for k, v in values.items():
        matplotlib.pyplot.figure(k)
        matplotlib.pyplot.hist(v, bins='auto')
        matplotlib.pyplot.grid(True)
    matplotlib.pyplot.show()


if __name__ == '__main__':
    main()
