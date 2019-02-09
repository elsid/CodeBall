#!/usr/bin/env python3

import numpy
import sys
import json
import subprocess
import results_stats
import scipy.optimize
import os.path
import time


def to_int(v):
    return int(round(v))


def to_unsigned_int(v):
    return max(int(round(v)), 0)


def to_int_list(v):
    return [to_int(w) for w in v]


OPTIONS_TYPES = dict(
    unsigned_int=to_unsigned_int,
    int=to_int,
    float=float,
)
DATA_DIR = 'optimization'


def main():
    script = sys.argv[1]
    port = sys.argv[2]
    config = read_json(sys.argv[3])
    options_index = read_json(sys.argv[4])
    team_size = int(sys.argv[5])
    nitro = sys.argv[6]
    seeds = sys.argv[7:]
    initial = make_initial(config, options_index)
    optimization_id = int(time.time())
    simulation_id = [1]
    os.makedirs(DATA_DIR, exist_ok=True)

    def f(x):
        for k, v in options_index.items():
            config[k] = OPTIONS_TYPES[v['type']](x[v['index']])
        return run_simulations(script, port, config, options_index, team_size, nitro, seeds, optimization_id, simulation_id)

    result = scipy.optimize.minimize(
        f,
        numpy.array(initial),
        method='Powell',
        options=dict(disp=True),
    )
    print(result)
    for k, v in options_index.items():
        config[k] = OPTIONS_TYPES[v['type']](result.x[v['index']])
    print(json.dumps(config))


def run_simulations(script, port, config, options_index, team_size, nitro, seeds, optimization_id, simulation_id):
    score_diff = sum(run_simulation(script, port, config, options_index, team_size, nitro, v, optimization_id, simulation_id) for v in seeds) / len(seeds)
    print('all', score_diff, {k: config[k] for k in options_index.keys()})
    return score_diff


def run_simulation(script, port, config, options_index, team_size, nitro, seed, optimization_id, simulation_id):
    config_path = os.path.abspath(os.path.join(DATA_DIR, 'config.%s.%s.json' % (optimization_id, simulation_id[0])))
    result_path = os.path.abspath(os.path.join(DATA_DIR, 'result.%s.%s.txt' % (optimization_id, simulation_id[0])))
    with open(config_path, 'w') as f:
        json.dump(config, f)
    simulation = subprocess.Popen([
        script,
        port,
        config_path,
        result_path,
        seed,
        str(team_size),
        nitro,
    ])
    simulation.wait()
    simulation_id[0] += 1
    result = results_stats.parse_result(results_stats.read_result(result_path))
    score_diff = result['first']['score'] - result['second']['score']
    print('single', score_diff, {k: config[k] for k in options_index.keys()}, result)
    return score_diff


def read_json(path):
    with open(path) as f:
        return json.load(f)


def make_initial(config, options_index):
    initial = [0] * len(options_index)
    for k, v in options_index.items():
        initial[v['index']] = config[k]
    return initial


if __name__ == '__main__':
    main()
