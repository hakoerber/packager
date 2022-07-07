#!/usr/bin/env bash

docker run \
    --rm \
    --publish 4444:4444 \
    --env SE_OPTS="--session-timeout 36000" \
    --shm-size="2g" \
    --net=host \
    --name docker-selenium \
    selenium/standalone-firefox:4.3.0-20220706
