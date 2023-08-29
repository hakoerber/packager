#!/usr/bin/env bash

sleep 100
set -o nounset

port="${1}"

db="$(mktemp)"

export SQLX_OFFLINE=true
export DATABASE_URL="sqlite://${db}"

cargo run -- --port "${port}"
