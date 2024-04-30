#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

docker run --add-host host.docker.internal:host-gateway --rm --name packager-prometheus -p 9090:9090 -v $SCRIPT_DIR/prometheus.yml:/etc/prometheus/prometheus.yml docker.io/prom/prometheus "${@}"
