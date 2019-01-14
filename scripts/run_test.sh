#!/bin/bash -ex

ID=${1}
OTHER=${2}
NITRO=${3}
PORTS_SHIFT=${4}

if ! [[ "${OTHER}" ]] || ! [[ "${ID}" ]]; then
    echo "other is not set, usage: ${0} <id> <other> [nitro] [ports_shift]"
    exit 1
fi

if ! [[ "${NITRO}" ]]; then
    NITRO=false
fi

if ! [[ "${PORTS_SHIFT}" ]]; then
    PORTS_SHIFT=0
fi

VERSION=$(git rev-parse --short HEAD)
LOG_DIR=${PWD}/log/${OTHER}_vs_${VERSION}
RESULT=${LOG_DIR}/result.${ID}.txt

mkdir -p ${LOG_DIR}

PLAYER_1_PORT=$((31011 + ${PORTS_SHIFT}))
PLAYER_2_PORT=$((31012 + ${PORTS_SHIFT}))

cargo build --release

local_runner/codeball2018 \
    --team-size 2 \
    --start-paused \
    --no-countdown \
    --results-file ${RESULT} \
    --noshow \
    --nitro ${NITRO} \
    --p1-name ${OTHER} \
    --p2-name c \
    --p1 tcp-${PLAYER_1_PORT} \
    --p2 tcp-${PLAYER_2_PORT} &
LOCAL_RUNNER_PID=$!

sleep 1
bin/${OTHER} 127.0.0.1 ${PLAYER_1_PORT} 0000000000000000 &
PLAYER_1_PID=$!

trap "kill -9 ${LOCAL_RUNNER_PID} ${PLAYER_1_PID}" SIGHUP SIGINT SIGTERM

sleep 1
/usr/bin/time -v target/release/my-strategy 127.0.0.1 ${PLAYER_2_PORT} 0000000000000000
cat ${RESULT}

wait
