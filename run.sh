#!/usr/bin/env bash

set -o nounset
set -o errexit
set -o xtrace

baseargs=(
    --database-url "postgresql://packager@postgres/packager?host=$PWD/pgdata/run"
)

cargobuildargs=(
    --all-features
)

cargoargs=(
    --color=always
)

./migrate.sh
cargo run "${cargoargs[@]}" "${cargobuildargs[@]}" -- "${baseargs[@]}" admin user create --username hannes --fullname "Hannes Körber" || true

serveargs=(
    --enable-opentelemetry false
    --enable-tokio-console false
    --enable-prometheus false
    serve
    --bind 127.0.0.1
    --disable-auth-and-assume-user hannes
)

cargo "${cargoargs[@]}" watch --why --clear --ignore pgdata -- cargo "${cargoargs[@]}" run "${cargobuildargs[@]}"  -- "${baseargs[@]}" "${serveargs[@]}" 2>&1
