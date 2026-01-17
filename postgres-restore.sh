#!/usr/bin/env bash

set -o nounset

./psql.sh < "${1}"
