#!/bin/bash -ex

P1=31142
P2=31143
export CONFIG=${1}
RESULT=${2}
SEED=${3}
TEAM_SIZE=${4}

cargo build --release --features=read_config,disable_output
local_runner/codeball2018 \
    --duration 10000 \
    --team-size ${TEAM_SIZE} \
    --start-paused \
    --seed ${SEED} \
    --no-countdown \
    --noshow \
    --nitro true \
    --results-file ${RESULT} \
    --p1 helper \
    --p2-name dev \
    --p2 tcp-${P1} &
LOCAL_RUNNER_PID=$!

trap "kill -9 ${LOCAL_RUNNER_PID}" SIGHUP SIGINT SIGTERM

sleep 1
/usr/bin/time -v target/release/my-strategy 127.0.0.1 ${P1} 0000000000000000
