#!/bin/bash -ex

PLAYER_1_PORT=31231
PLAYER_2_PORT=31232

mkdir -p demo/

cargo build --release --features=enable_render

cp target/release/my-strategy demo/player2

cargo build --release

cp target/release/my-strategy demo/player1

local_runner/codeball2018 \
    --team-size 3 \
    --no-countdown \
    --nitro true \
    --p1 tcp-${PLAYER_1_PORT} \
    --p2 tcp-${PLAYER_2_PORT} \
    --p1-name 'Player 1' \
    --p2-name 'Player 2' \
    &
LOCAL_RUNNER_PID=$!

sleep 1
demo/player1 127.0.0.1 ${PLAYER_1_PORT} 0000000000000000 &
PLAYER_1_PID=$!

trap "kill -9 ${LOCAL_RUNNER_PID} ${PLAYER_1_PID}" SIGHUP SIGINT SIGTERM

sleep 1
demo/player2 127.0.0.1 ${PLAYER_2_PORT} 0000000000000000
