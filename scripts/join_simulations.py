#!/usr/bin/env python

import sys
import json


def main():
    left = sys.argv[1]
    right = sys.argv[2]
    with open(left) as left_f:
        with open(right) as right_f:
            left_records = dict(make_record(v) for v in left_f)
            right_records = dict(make_record(v) for v in right_f)
            for left_record_id, left_record in left_records.items():
                right_record = right_records[left_record_id]
                for k in left_record.keys():
                    if k in right_record:
                        if left_record[k] is None:
                            left_record[k] = right_record[k]
                        else:
                            right_record[k] = left_record[k]
            for v in right_records.values():
                print(json.dumps(v))


def make_record(line):
    record = json.loads(line)
    return record['id'], record


if __name__ == '__main__':
    main()
