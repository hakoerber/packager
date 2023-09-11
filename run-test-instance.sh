#!/usr/bin/env bash

cd $( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

set -o nounset

port="${1}"

db="$(mktemp)"

exec ./target/debug/packager --port "${port}" --database-url "sqlite://${db}"
