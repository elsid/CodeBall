#!/usr/bin/env python3

import sys
import statistics
import numpy
import matplotlib.pyplot


def main():
    records = list(collect_data(sys.argv[1:]))
    seeds = [v['seed'] for v in records]
    players = dict(
        first=dict(places=[], scores=[]),
        second=dict(places=[], scores=[]),
    )
    for r in records:
        for k, p in players.items():
            p['places'].append(r[k]['place'])
            p['scores'].append(r[k]['score'])
    stats = dict(
        _1=[sum(w == 1 for w in v['places']) for v in players.values()],
        _2=[sum(w == 2 for w in v['places']) for v in players.values()],
        total_score=[sum(v['scores']) for v in players.values()],
        min_score=[min(v['scores']) for v in players.values()],
        max_score=[max(v['scores']) for v in players.values()],
        mean_score=[statistics.mean(v['scores']) for v in players.values()],
        median_score=[statistics.median(v['scores']) for v in players.values()],
        stdev_score=[statistics.stdev(v['scores']) for v in players.values()],
        q95_score=[numpy.quantile(v['scores'], 0.95) for v in players.values()],
    )
    row('', *(list(players.keys()) + ['ratio (second/first)']))
    for k, v in stats.items():
        row(k, *(v + ratio(v)))
    print()
    print(*seeds)
    fig, ax = matplotlib.pyplot.subplots()
    bins = numpy.arange(0, max(max(v['scores']) for v in players.values()))
    for k, v in players.items():
        ax.hist(v['scores'], bins=bins, label=k, alpha=0.5)
        ax.set_xticks(bins)
        ax.grid(True)
        ax.legend()
    matplotlib.pyplot.show()


def ratio(values):
    return [values[1] / values[0] if values[0] else float('inf')]


def row(*args):
    print(('{:>25}' * len(args)).format(*args))


def collect_data(paths):
    for path in paths:
        content = read_result(path)
        if content:
            yield parse_result(read_result(path))


def read_result(path):
    with open(path) as f:
        return f.read()


def parse_result(content):
    first, second, seed = content.split('\n')[:3]
    return dict(first=parse_record(first), second=parse_record(second), seed=seed)


def parse_record(content):
    place, score, status = content.split(':')[:3]
    return dict(place=int(place), score=int(score), status=status.strip())


if __name__ == '__main__':
    main()
