#!/usr/bin/env bash

set -o nounset

pg_dump -h "$PWD/pgdata/run/" --exclude-table=_sqlx_migrations -U postgres packager --data-only
