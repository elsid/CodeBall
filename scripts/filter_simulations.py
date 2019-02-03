#!/usr/bin/env python

import sys
import json

from check_goalkeeper_report import is_miss, is_hit, is_safe, is_goal


def main():
    filter_f = dict(miss=is_miss, hit=is_hit, safe=is_safe, goal=is_goal)[sys.argv[1]]
    for line in sys.stdin:
        record = json.loads(line)
        if filter_f(record):
            sys.stdout.write(line)


if __name__ == '__main__':
    main()
