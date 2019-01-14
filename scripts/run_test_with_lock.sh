#!/bin/bash -ex

ID=$(date +%s)

until flock -n ${PWD}/log/${ID}.lock scripts/run_test.sh ${ID} ${@}; do
    echo "${ID} is busy"
    sleep 1
    ID=$(date +%s)
done

rm ${PWD}/log/${ID}.lock
