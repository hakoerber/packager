#!/usr/bin/env bash

set -o nounset
set -o errexit

if [[ ! -e "./pgdata" ]] ; then
    initdb --locale=C.UTF-8 --encoding=UTF8 -D './pgdata' --username postgres
    mkdir ./pgdata/run
fi

postgres -D ./pgdata -k run -h "" &
pg_pid="$!"

sleep 1

psql -h "${PWD}/pgdata/run/" -U postgres postgres <<SQLCMD
CREATE ROLE packager NOSUPERUSER NOCREATEDB NOCREATEROLE LOGIN CONNECTION LIMIT 10;
CREATE DATABASE packager WITH OWNER 'packager';
SQLCMD

kill "${pg_pid}"
wait
