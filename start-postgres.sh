#!/usr/bin/env bash

set -o nounset

postgres -D ./pgdata -k run -c listen_addresses=''
