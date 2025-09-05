#!/usr/bin/env bash
# 
set -o nounset

pg_dump -h "$PWD/pgdata/run/" -U postgres packager #--data-only --inserts
