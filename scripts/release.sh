#!/bin/bash -ex

VERSION=$(date +%Y-%m-%d_%H-%M-%S)-$(git rev-parse --short HEAD)
SRC=${PWD}
DIR=${SRC}/release/${VERSION}

mkdir -p release
mkdir ${DIR}
mkdir ${DIR}/src

cp src/action.rs ${DIR}/src
cp src/arena.rs ${DIR}/src
cp src/ball.rs ${DIR}/src
cp src/common.rs ${DIR}/src
cp src/entity.rs ${DIR}/src
cp src/my_strategy_impl.rs ${DIR}/src
cp src/my_strategy.rs ${DIR}/src
cp src/optimal_action.rs ${DIR}/src
cp src/plane.rs ${DIR}/src
cp src/random.rs ${DIR}/src
cp src/render.rs ${DIR}/src
cp src/robot.rs ${DIR}/src
cp src/rules.rs ${DIR}/src
cp src/simulator.rs ${DIR}/src
cp src/sphere.rs ${DIR}/src
cp src/vec2.rs ${DIR}/src
cp src/vec3.rs ${DIR}/src
cp src/world.rs ${DIR}/src

cd ${DIR}/src

zip ${SRC}/release/${VERSION}.zip *.rs

cd ..

cp -r ${SRC}/src/tests ${DIR}/src

cp ${SRC}/raic/packages/rust/Cargo.lock ${DIR}
cp ${SRC}/raic/packages/rust/Cargo.toml ${DIR}
cp ${SRC}/raic/packages/rust/src/main.rs ${DIR}/src
cp ${SRC}/raic/packages/rust/src/remote_process_client.rs ${DIR}/src
cp ${SRC}/raic/packages/rust/src/strategy.rs ${DIR}/src
cp -r ${SRC}/raic/packages/rust/src/model ${DIR}/src

cargo build --frozen
cargo test