#!/usr/bin/env bash

docker run \
  --rm \
  --name \
  packager-jaeger \
  -p 4317:4317 \
  -p 4318:4318 \
  -p16686:16686 \
  jaegertracing/all-in-one:latest \
  "${@}"
