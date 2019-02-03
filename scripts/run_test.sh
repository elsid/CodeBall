#!/bin/bash -ex

ID=${1}
OTHER=${2}
PREFIX=
SUFFIX=

if ! [[ "${OTHER}" ]] || ! [[ "${ID}" ]]; then
    echo "other is not set, usage: ${0} <id> <other>"
    exit 1
fi

if [[ ${DEBUG} ]]; then
    PREFIX=debug_
fi

if ! [[ "${NITRO}" ]]; then
    NITRO=true
else
    SUFFIX=_nitro
fi

if ! [[ "${TEAM_SIZE}" ]]; then
    TEAM_SIZE=3
fi

if ! [[ "${DURATION}" ]]; then
    DURATION=18000
fi

SUFFIX=${SUFFIX}_${TEAM_SIZE}_${DURATION}

if ! [[ "${PORTS_SHIFT}" ]]; then
    PORTS_SHIFT=0
fi

if [[ "${USE_BIN}" ]]; then
    VERSION=$(echo ${USE_BIN} | sed 's;.*/;;')
else
    VERSION=$(git rev-parse --short HEAD)
fi

LOG_DIR=${PWD}/log/${PREFIX}${OTHER}_vs_${VERSION}${SUFFIX}
RESULT=${LOG_DIR}/result.${ID}.txt

mkdir -p ${LOG_DIR}

PLAYER_1_PORT=$((31011 + ${PORTS_SHIFT}))
PLAYER_2_PORT=$((31012 + ${PORTS_SHIFT}))

if ! [[ "${USE_BIN}" ]]; then
    cargo build --release
fi

local_runner/codeball2018 \
    --team-size ${TEAM_SIZE} \
    --duration ${DURATION} \
    --start-paused \
    --no-countdown \
    --results-file ${RESULT} \
    --noshow \
    --nitro ${NITRO} \
    --p1-name ${OTHER} \
    --p2-name ${VERSION} \
    --p1 tcp-${PLAYER_1_PORT} \
    --p2 tcp-${PLAYER_2_PORT} &
LOCAL_RUNNER_PID=$!

sleep 1
bin/${OTHER} 127.0.0.1 ${PLAYER_1_PORT} 0000000000000000 &
PLAYER_1_PID=$!

trap "kill -9 ${LOCAL_RUNNER_PID} ${PLAYER_1_PID}" SIGHUP SIGINT SIGTERM

sleep 1
if [[ "${USE_BIN}" ]]; then
    /usr/bin/time -v ${USE_BIN} 127.0.0.1 ${PLAYER_2_PORT} 0000000000000000
else
    /usr/bin/time -v target/release/my-strategy 127.0.0.1 ${PLAYER_2_PORT} 0000000000000000
fi
cat ${RESULT}

wait
