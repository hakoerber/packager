#!/usr/bin/env bash

set -o nounset
set -o errexit
set -o xtrace

baseargs=(
    --database-url "postgresql://postgres@postgres/packager?host=$PWD/pgdata/run"
)

cargoargs=(
    --all-features
)

cargo run "${cargoargs[@]}" -- "${baseargs[@]}" migrate
cargo run "${cargoargs[@]}" -- "${baseargs[@]}" admin user create --username hannes --fullname "Hannes Körber" || true

serveargs=(
 --enable-opentelemetry true
 --enable-tokio-console true
 --enable-prometheus true
 --prometheus-bind 0.0.0.0
 --prometheus-port 3001
 serve
 --bind 127.0.0.1
 --disable-auth-and-assume-user hannes
)

cargo watch --why --ignore pgdata -- cargo run "${cargoargs[@]}" -- "${baseargs[@]}" "${serveargs[@]}"
