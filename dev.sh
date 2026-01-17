#!/usr/bin/env bash

set -o nounset
set -o errexit
set -o xtrace

baseargs=(
    --database-url "postgresql://postgres@postgres/packager?host=$(pwd)/pgdata/run"
)

cargobuildargs=(
    --all-features
)

cargoargs=(
    --color=always
)

serveargs=(
    --enable-opentelemetry true
    --enable-tokio-console true
    --enable-prometheus true
    --prometheus-bind 127.0.0.1
    --prometheus-port 3001
    serve
    --bind 127.0.0.1
    --disable-auth-and-assume-user hannes
)

RUSTFLAGS="--cfg tokio_unstable" cargo "${cargoargs[@]}" watch --why --clear --ignore pgdata -- cargo "${cargoargs[@]}" run "${cargobuildargs[@]}"  -- "${baseargs[@]}" "${serveargs[@]}"
