#!/usr/bin/env bash

set -o nounset
set -o errexit
set -o xtrace

baseargs=(
    --database-url "postgresql://packager@postgres/packager?host=$PWD/pgdata/run"
)

cargobuildargs=(
)

cargoargs=(
    --color=always
)

cargo run "${cargoargs[@]}" "${cargobuildargs[@]}" -- "${baseargs[@]}" migrate
