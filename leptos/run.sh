#!/usr/bin/env bash

set -o nounset

DATABASE_URL="postgresql://packager@postgres/packager?host=$PWD/../pgdata/run" cargo --color=always leptos watch "${@}"
