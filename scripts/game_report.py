#!/usr/bin/env python

import sys
import json
import numpy
import statistics
import matplotlib.pyplot

from itertools import islice, chain
from collections import defaultdict


def main():
    raw = [json.loads(v) for v in sys.stdin]
    constants = raw[0]
    tick_time = 1.0 / constants['TICKS_PER_SECOND']
    records = [fill(v, raw[1]) for v in islice(raw, 1, len(raw))]
    row(
        '',
        *sorted('%s-%s' % (records[0]['names'][v['player_index']], v['id']) for v in records[0]['robots']),
        *sorted(records[0]['names']),
        'ratio',
    )
    report_speeds(records)
    report_distances(records, tick_time)
    report_radius(records)
    report_hits(records)
    report_rounds(records)
    matplotlib.pyplot.show()


def report_speeds(records):
    speeds = split_by_keys(dict(get_speeds(v)) for v in records if v['reset_ticks'] is None)
    fig, ax = matplotlib.pyplot.subplots()
    ax.set_title('speeds distribution')
    bins = numpy.arange(0, max(max(v) for v in speeds.values()) + 1, 0.1)
    for k, v in speeds.items():
        ax.hist(v, bins=bins, label=k, histtype='step', linewidth=2)
    ax.grid(True)
    ax.legend()
    ax.set_xticks(numpy.arange(0, max(max(v) for v in speeds.values()) + 1, 1.0))
    names = sorted(records[0]['names'])
    speeds_by_players = [statistics.mean(chain(*[w for k, w in speeds.items() if v in k])) for v in names]
    row(
        'mean speed',
        *[statistics.mean(v) for _, v in sorted(speeds.items())],
        *speeds_by_players,
        speeds_by_players[0] / speeds_by_players[1]
    )


def report_distances(records, tick_time):
    distances = split_by_keys(get_distances(records, tick_time))
    fig, ax = matplotlib.pyplot.subplots()
    ax.set_title('distances')
    for k, v in distances.items():
        ax.plot(numpy.arange(0, len(v), 1), v, label=k, linewidth=2)
        ax.set_xticks(numpy.arange(0, len(v) + 1, 1000))
    for goal, player in get_goals(records):
        ax.axvline(goal, color='green' if player else 'red')
    ax.grid(True)
    ax.legend()
    names = sorted(records[0]['names'])
    distances_by_players = [sum([w[-1] for k, w in distances.items() if v in k]) for v in names]
    row(
        'distance',
        *[v[-1] for _, v in sorted(distances.items())],
        *distances_by_players,
        distances_by_players[0] / distances_by_players[1]
    )


def report_radius(records):
    radius = split_by_keys(dict(get_radius(v)) for v in records if v['reset_ticks'] is None)
    fig, ax = matplotlib.pyplot.subplots()
    ax.set_title('radius distribution')
    for k, v in radius.items():
        ax.hist(v, label=k, linewidth=2)
    ax.grid(True)
    ax.legend()
    names = sorted(records[0]['names'])
    radius_by_players = [statistics.mean(chain(*[w for k, w in radius.items() if v in k])) for v in names]
    row(
        'mean radius',
        *[statistics.mean(v) for _, v in sorted(radius.items())],
        *radius_by_players,
        radius_by_players[0] / radius_by_players[1]
    )
    row(
        'median radius',
        *[statistics.mean(v) for _, v in sorted(radius.items())],
        *radius_by_players,
        radius_by_players[0] / radius_by_players[1]
    )


def report_hits(records):
    hits = split_by_keys(get_hits(records))
    fig, ax = matplotlib.pyplot.subplots()
    ax.set_title('hits')
    for k, v in hits.items():
        ax.plot(numpy.arange(0, len(v), 1), v, label=k, linewidth=2)
        ax.set_xticks(numpy.arange(0, len(v) + 1, 1000))
    ax.grid(True)
    ax.legend()
    names = sorted(records[0]['names'])
    hits_by_players = [sum([w[-1] for k, w in hits.items() if v in k]) for v in names]
    row(
        'hits',
        *['-' for _ in records[0]['robots']],
        *hits_by_players,
        hits_by_players[0] / hits_by_players[1]
    )


def report_rounds(records):
    first = True
    score_0 = 0
    row('initial ball.y', 'ticks', 'winner')
    for record in records:
        if first:
            start = record
            first = False
        elif record['reset_ticks'] == 119:
            first = True
            player = record['names'][0] if score_0 < record['scores'][0] else record['names'][1]
            score_0 = record['scores'][0]
            row(start['ball']['position']['y'], record['current_tick'] - start['current_tick'], player)


def fill(record, first):
    record['names'] = first['names']
    return record


def row(*args):
    print(('{:>20}' * len(args)).format(*args))


def get_goals(records):
    score_0 = 0
    for record in records:
        if record['reset_ticks'] == 119:
            yield (record['current_tick'], score_0 < record['scores'][0])
            score_0 = record['scores'][0]


def get_speeds(record):
    for robot in record['robots']:
        velocity = numpy.array([robot['velocity']['x'], robot['velocity']['y'], robot['velocity']['z']])
        speed = numpy.linalg.norm(velocity)
        yield '%s-%s' % (record['names'][robot['player_index']], robot['id']), speed


def get_distances(records, tick_time):
    total = defaultdict(float)
    for i in range(1, len(records)):
        out = dict()
        prev = records[i - 1]
        current = records[i]
        prev_robots = sorted((v for v in prev['robots']), key=lambda v: v['id'])
        current_robots = sorted((v for v in current['robots']), key=lambda v: v['id'])
        for p, c in zip(prev_robots, current_robots):
            key = '%s-%s' % (prev['names'][p['player_index']], p['id'])
            if prev['reset_ticks'] is None and current['reset_ticks'] is None:
                prev_position = numpy.array([p['position']['x'], p['position']['y'], p['position']['z']])
                current_position = numpy.array([c['position']['x'], c['position']['y'], c['position']['z']])
                distance = numpy.linalg.norm(current_position - prev_position) * tick_time
                total[key] += distance
            out[key] = total[key]
        yield out


def get_radius(record):
    for robot in record['robots']:
        yield '%s-%s' % (record['names'][robot['player_index']], robot['id']), robot['radius']


def get_hits(records):
    hits = {v: 0 for v in records[0]['names']}
    for record in records:
        hits = {k: v for k, v in hits.items()}
        for hit in record['hits']:
            if hit['player_index'] is not None:
                hits[record['names'][hit['player_index']]] += 1
        yield hits


def split_by_keys(values):
    result = defaultdict(list)
    for value in values:
        for k, v in value.items():
            result[k].append(v)
    return result


if __name__ == '__main__':
    main()
