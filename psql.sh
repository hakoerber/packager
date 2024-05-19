#!/usr/bin/env bash

psql -h $PWD/pgdata/run/ -U postgres packager
