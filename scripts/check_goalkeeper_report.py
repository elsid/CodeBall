#!/usr/bin/env python

import sys
import json
import matplotlib.pyplot
import math
import numpy

from collections import defaultdict
from matplotlib.collections import LineCollection


def main():
    if len(sys.argv) > 1:
        with open(sys.argv[1]) as f:
            other_records = [json.loads(v) for v in f]
    else:
        other_records = None
    records = [json.loads(v) for v in sys.stdin]
    if other_records:
        show_metric(('stdin', records), ('arg', other_records))
    else:
        show_metric(('stdin', records))
    if other_records:
        show_distributions_diff(records, other_records)
    else:
        show_distributions(records)
    if other_records:
        show_maps_diff(records, other_records)
    else:
        show_maps(records)
    matplotlib.pyplot.show()


def show_metric(*args):
    row('source', 'simulations', 'misses', 'hits', 'hits rate', 'safes', 'goals', 'goals rate', 'safes rate', 'safes/goals')
    for n, records in args:
        misses = sum(1 for v in records if is_miss(v))
        hits = sum(1 for v in records if is_hit(v))
        safes = sum(1 for v in records if is_safe(v))
        goals = sum(1 for v in records if is_goal(v))
        hits_rate = safe_div(hits, misses)
        goals_rate = safe_div(goals, hits)
        safes_rate = safe_div(safes, hits)
        safes_goals_ratio = safe_div(safes, goals)
        row(n, len(records), misses, hits, hits_rate, safes, goals, goals_rate, safes_rate, safes_goals_ratio)


def show_distributions(records):
    goals = dict()
    without_goalkeeper = list()
    with_goalkeeper = list()
    for record in records:
        if is_hit(record):
            update_values('without_goalkeeper', record['parameters'], goals)
            without_goalkeeper.append(record['empty']['tick'])
        if is_goal(record):
            update_values('with_goalkeeper', record['parameters'], goals)
            with_goalkeeper.append(record['goalkeeper']['tick'])
    for name, values in goals.items():
        figure, axis = matplotlib.pyplot.subplots()
        title = '%s goals' % name
        figure.canvas.set_window_title(title)
        axis.set_title(title)
        bins = numpy.linspace(
            min(min(v) for v in values.values()),
            max(max(v) for v in values.values()),
            num=10,
        )
        for k, v in values.items():
            axis.hist(v, label=k, alpha=0.5, bins=bins)
            axis.grid(True)
    figure, axis = matplotlib.pyplot.subplots()
    title = 'ticks'
    figure.canvas.set_window_title(title)
    axis.set_title(title)
    ticks = dict(
        without_goalkeeper=without_goalkeeper,
        with_goalkeeper=with_goalkeeper,
    )
    bins = numpy.linspace(
        min(min(v) for v in ticks.values()),
        max(max(v) for v in ticks.values()),
        num=10,
    )
    for k, v in ticks.items():
        axis.hist(v, label=k, alpha=0.5, bins=bins)
    axis.grid(True)


def show_distributions_diff(records, other_records):
    goals = dict()
    without_goalkeeper = list()
    with_goalkeeper_before = list()
    with_goalkeeper_after = list()
    other_records_index = {v['id']: v for v in other_records}
    for record in records:
        other_record = other_records_index[record['id']]
        if is_hit(record):
            update_values('without_goalkeeper', record['parameters'], goals)
            without_goalkeeper.append(record['empty']['tick'])
        if is_goal(record):
            update_values('with_goalkeeper_before', record['parameters'], goals)
            with_goalkeeper_before.append(record['goalkeeper']['tick'])
        if is_goal(other_record):
            update_values('with_goalkeeper_after', other_record['parameters'], goals)
            with_goalkeeper_after.append(other_record['goalkeeper']['tick'])
    for name, values in goals.items():
        figure, axis = matplotlib.pyplot.subplots()
        title = '%s goals' % name
        figure.canvas.set_window_title(title)
        axis.set_title(title)
        bins = numpy.linspace(
            min(min(v) for v in values.values()),
            max(max(v) for v in values.values()),
            num=10,
        )
        del values['without_goalkeeper']
        for k, v in values.items():
            axis.hist(v, label=k, alpha=0.5, bins=bins)
            axis.grid(True)
    figure, axis = matplotlib.pyplot.subplots()
    title = 'ticks'
    figure.canvas.set_window_title(title)
    axis.set_title(title)
    ticks = dict(
        without_goalkeeper=without_goalkeeper,
        with_goalkeeper_before=with_goalkeeper_before,
        with_goalkeeper_after=with_goalkeeper_after,
    )
    bins = numpy.linspace(
        min(min(v) for v in ticks.values()),
        max(max(v) for v in ticks.values()),
        num=10,
    )
    del ticks['without_goalkeeper']
    for k, v in ticks.items():
        axis.hist(v, label=k, alpha=0.5, bins=bins)
    axis.grid(True)


def show_maps(records):
    maps = (
        ('misses', 'gray', is_miss),
        ('hits', 'black', is_hit),
        ('safes', 'green', is_safe),
        ('goals', 'red', is_goal),
    )
    for name, color, predicate in maps:
        for f in (generate_map, generate_velocity_map):
            f(name, color, records, predicate, 'x', 'z')
            f(name, color, records, predicate, 'x', 'y')


def show_maps_diff(records, other_records):
    maps = (
        ('misses', 'gray', is_miss, merge),
        ('hits', 'black', is_hit, merge),
        ('safes removed', 'green', is_safe, diff),
        ('safes added', 'green', is_safe, lambda a, b, p: diff(b, a, p)),
        ('goals left', 'red', is_goal, intersect),
    )
    for name, color, predicate, combine in maps:
        combination = combine(records, other_records, predicate)
        for f in (generate_map, generate_velocity_map):
            f(name, color, combination, predicate, 'x', 'z')
            f(name, color, combination, predicate, 'x', 'y')


def merge(a, b, predicate):
    a_index = {v['id'] for v in a if predicate(v)}
    result = a
    for v in b:
        if predicate(v) and v['id'] not in a_index:
            result.append(v)
    return result


def diff(a, b, predicate):
    a_index = {v['id'] for v in a if predicate(v)}
    result = list()
    for v in b:
        if predicate(v) and v['id'] not in a_index:
            result.append(v)
    return result


def intersect(a, b, predicate):
    b_index = {v['id'] for v in b if predicate(v)}
    result = list()
    for v in a:
        if predicate(v) and v['id'] in b_index:
            result.append(v)
    return result


def is_miss(record):
    return not is_hit(record)


def is_hit(record):
    return record['empty']['score'] < 0


def is_safe(record):
    return is_hit(record) and not is_goal(record)


def is_goal(record):
    return record['goalkeeper'] and record['goalkeeper']['score'] < 0


def generate_map(name, color, records, predicate, axis_x, axis_y):
    figure, axis = matplotlib.pyplot.subplots()
    title = '%s map by %s, %s' % (name, axis_x, axis_y)
    figure.canvas.set_window_title(title)
    axis.set_title(title)
    x = list()
    y = list()
    colors = list()
    lines = list()
    for record in records:
        if predicate(record):
            ball_x = record['parameters']['ball_position'][axis_x]
            ball_y = record['parameters']['ball_position'][axis_y]
            velocity_x = record['parameters']['ball_velocity'][axis_x]
            velocity_y = record['parameters']['ball_velocity'][axis_y]
            speed = record['parameters']['speed'] / 50
            to_x = ball_x + velocity_x * speed
            to_y = ball_y + velocity_y * speed
            x.append(ball_x)
            y.append(ball_y)
            colors.append(color)
            lines.append([(ball_x, ball_y), (to_x, to_y)])
    axis.scatter(x, y, c=colors)
    axis.add_collection(LineCollection(lines, colors=colors))
    axis.grid(True)


def generate_velocity_map(name, color, records, predicate, axis_x, axis_y):
    figure, axis = matplotlib.pyplot.subplots()
    title = '%s velocity map by %s, %s' % (name, axis_x, axis_y)
    figure.canvas.set_window_title(title)
    axis.set_title(title)
    x_values = list()
    y_values = list()
    colors = list()
    areas = list()
    for record in records:
        if predicate(record):
            x_values.append(record['parameters']['ball_velocity'][axis_x])
            y_values.append(record['parameters']['ball_velocity'][axis_y])
            colors.append(color)
            areas.append(math.log(record['parameters']['speed']))
    axis.scatter(x_values, y_values, c=colors, s=areas)
    axis.grid(True)


def update_values(kind, parameters, values):
    for name, parameter in parameters.items():
        if isinstance(parameter, dict):
            for k, v in parameter.items():
                if '_'.join([name, k]) not in values:
                    values['_'.join([name, k])] = defaultdict(list)
                values['_'.join([name, k])][kind].append(v)
        else:
            if name not in values:
                values[name] = defaultdict(list)
            values[name][kind].append(parameter)


def update_bounds(name, parameter, bounds):
    if name not in bounds:
        bounds[name] = dict(min=parameter, max=parameter)
    else:
        bounds[name]['min'] = min(parameter, bounds[name]['min'])
        bounds[name]['max'] = max(parameter, bounds[name]['max'])


def row(*args):
    print(('{:>22}' * len(args)).format(*args))


def safe_div(dividend, divisor):
    return dividend / divisor if divisor != 0 else float('inf')


if __name__ == '__main__':
    main()
