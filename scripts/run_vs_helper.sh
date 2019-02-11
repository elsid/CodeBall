#!/bin/bash -ex

PORT=31231

cargo build --release --features=enable_render

local_runner/codeball2018 \
    --team-size 3 \
    --start-paused \
    --no-countdown \
    --nitro true \
    --p1 tcp-${PORT} \
    --p1-name dev \
    --p2-name helper \
    &
LOCAL_RUNNER_PID=$!

trap "kill -9 ${LOCAL_RUNNER_PID}" SIGHUP SIGINT SIGTERM

sleep 1
target/release/my-strategy 127.0.0.1 ${PORT} 0000000000000000
