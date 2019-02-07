#!/bin/bash -ex

P1=31140
P2=31141
export CONFIG=${1}
RESULT=${2}
SEED=${3}
TEAM_SIZE=${4}

cargo build --release --features=read_config,disable_output
local_runner/codeball2018 \
    --duration 100000 \
    --team-size ${TEAM_SIZE} \
    --start-paused \
    --seed ${SEED} \
    --no-countdown \
    --noshow \
    --nitro true \
    --results-file ${RESULT} \
    --p1-name v42 \
    --p2-name dev \
    --p1 tcp-${P1} \
    --p2 tcp-${P2} &
LOCAL_RUNNER_PID=$!

sleep 1
(bin/v42 127.0.0.1 ${P1} 0000000000000000 &> /dev/null &)
OTHER_PID=$!

trap "kill -9 ${LOCAL_RUNNER_PID} ${OTHER_PID}" SIGHUP SIGINT SIGTERM

sleep 1
/usr/bin/time -v target/release/my-strategy 127.0.0.1 ${P2} 0000000000000000
