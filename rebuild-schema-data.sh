#!/usr/bin/env bash

set -o nounset
set -o errexit
set -o xtrace

export DATABASE_URL="postgresql://packager@postgres/packager?host=$PWD/pgdata/run"

cargo sqlx database create
cargo sqlx migrate run
cargo sqlx prepare -- --color=always
