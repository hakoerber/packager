#!/usr/bin/env bash

if [[ -n "$1" ]] ; then
    export DATABASE_URL="sqlite://${1}"
else
    db="$(mktemp)"

    export DATABASE_URL="sqlite://${db}"
fi

cargo sqlx database create
cargo sqlx migrate run
cargo sqlx prepare -- --color=always
