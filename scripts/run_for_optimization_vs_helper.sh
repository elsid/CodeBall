#!/bin/bash -ex

PORT=${1}
export CONFIG=${2}
RESULT=${3}
SEED=${4}
TEAM_SIZE=${5}
NITRO=${6}

cargo build --release --features=read_config,disable_output
local_runner/codeball2018 \
    --duration 10000 \
    --team-size ${TEAM_SIZE} \
    --start-paused \
    --seed ${SEED} \
    --no-countdown \
    --noshow \
    --nitro ${NITRO} \
    --results-file ${RESULT} \
    --p1 helper \
    --p2-name dev \
    --p2 tcp-${PORT} &
LOCAL_RUNNER_PID=$!

trap "kill -9 ${LOCAL_RUNNER_PID}" SIGHUP SIGINT SIGTERM

sleep 1
/usr/bin/time -v target/release/my-strategy 127.0.0.1 ${PORT} 0000000000000000
