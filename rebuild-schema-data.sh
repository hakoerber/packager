#!/usr/bin/env bash

set -o nounset
set -o errexit
set -o xtrace

if [[ -n "$1" ]] ; then
    export DATABASE_URL="postgresql://postgres@sock/packager?host=$1"
else
    echo err
    exit 1
    # db="$(mktemp)"

    # export DATABASE_URL="postgresql://postgres@postgres?host=$1"
    # export DATABASE_URL="sqlite://${db}"
fi

echo $DATABASE_URL

cargo sqlx database create
cargo sqlx migrate run
cargo sqlx prepare -- --color=always
