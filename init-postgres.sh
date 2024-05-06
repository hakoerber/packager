#!/usr/bin/env bash

set -o nounset

if [[ ! -e "./pgdata" ]] ; then
    initdb --locale=C.UTF-8 --encoding=UTF8 -D './pgdata' --user postgres
    mkdir ./pgdata/run
fi

postgres -D ./pgdata -k run -h "" 
