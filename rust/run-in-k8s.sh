#!/usr/bin/env bash

set -o nounset
set -o errexit

pushd ~/projects/mycloud/ > /dev/null

./kubectl.sh exec \
    "deployment/$(./kubectl.sh get deployment --output=jsonpath={.items..metadata.name} -l app=packager)" \
    -c packager \
    -- \
    packager \
        --database-url /var/lib/packager/db/db.sqlite \
        "${@}"
