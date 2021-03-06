#!/usr/bin/env python

import sys
import json
import statistics
import numpy
import matplotlib.pyplot

from collections import Counter, defaultdict


def main():
    raw = [json.loads(v) for v in sys.stdin]
    records = [v for v in raw if isinstance(v, dict)]
    by_ticks = defaultdict(dict)
    for record in records:
        for k, v in record.items():
            if isinstance(v, list):
                record[k] = ','.join(v)
    for record in records:
        by_ticks[record['current_tick']][record['robot_id']] = record
    values = {k: [v.get(k) for v in records if v.get(k) is not None] for k in records[0]}
    values['time_to_score'] = [v for v in values['time_to_score']]
    values['transitions'] = [w for v in values['path'] for w in v.split(',')]
    row('', 'n', 'sum', 'q95', 'min', 'max', 'mean', 'median', 'stdev')
    for k, v in values.items():
        if not v:
            row(k, len(v))
        elif isinstance(v[0], (int, float)):
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
        elif isinstance(v[0], str):
            count = Counter(v)
            row(
                k,
                len(count),
                '-',
                '-',
                '%s (%s)' % min(count.items(), key=lambda v: v[1]),
                '%s (%s)' % max(count.items(), key=lambda v: v[1]),
            )
    print()
    for k, v in values.items():
        if not v or k in ('game_micro_ticks_limit', 'game_micro_ticks'):
            continue
        elif isinstance(v[0], (int, float)):
            fig, ax = matplotlib.pyplot.subplots()
            fig.canvas.set_window_title(k)
            if v and k in ('iteration', 'step', 'total_iterations'):
                n = max(v) - min(v)
                step = 1
                while n > 50:
                    n /= 2
                    step *= 2
                ax.hist(v, bins=numpy.arange(min(v), max(v) + step, step))
                ax.set_xticks(numpy.arange(min(v), max(v) + step, step))
            else:
                ax.hist(v, bins='auto')
            ax.grid(True)
        elif isinstance(v[0], str):
            count = sorted(Counter(v).items(), key=lambda v: -v[1])
            fig, ax = matplotlib.pyplot.subplots()
            fig.subplots_adjust(left=0.3)
            fig.canvas.set_window_title(k)
            x_coordinates = numpy.arange(len(count))
            ax.barh(x_coordinates, [v[1] for v in count], align='center')
            ax.yaxis.set_major_locator(matplotlib.pyplot.FixedLocator(x_coordinates))
            ax.yaxis.set_major_formatter(matplotlib.pyplot.FixedFormatter(['%s (%s)' % (v[0], v[1]) for v in count]))
            ax.grid(True)
    fig, ax = matplotlib.pyplot.subplots()
    fig.canvas.set_window_title('game_micro_ticks')
    ticks = numpy.arange(max(values['current_tick']))
    for k in ('game_micro_ticks', 'game_micro_ticks_limit'):
        y = list()
        for tick in ticks:
            current = [v[k] for v in by_ticks[tick].values()]
            if current and (not y or max(current) > y[-1]):
                y.append(max(current))
            else:
                y.append(y[-1])
        ax.plot(ticks, y)
    ax.grid(True)
    matplotlib.pyplot.show()


def row(*args):
    print(('{:>25}' * len(args)).format(*args))


if __name__ == '__main__':
    main()
