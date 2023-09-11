#!/usr/bin/env bash

cd $( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

set -o nounset

export SQLX_OFFLINE=true

exec cargo build
