#!/usr/bin/env bash

set -o nounset
set -o pipefail

rustup target add x86_64-unknown-linux-musl

cargo build --target x86_64-unknown-linux-musl --release

docker build -t packager:latest .
docker tag packager:latest packager:$(git rev-parse HEAD)
docker tag packager:latest registry.hkoerber.de/packager:$(git rev-parse HEAD)
