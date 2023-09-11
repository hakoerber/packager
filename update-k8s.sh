#!/usr/bin/env bash

set -o nounset
set -o pipefail
set -o errexit

id=$(pwgen -s 25 1)
url=registry.hkoerber.de/packager:$id 

echo "NEW URL: " $url

./build-container.sh

docker tag packager:latest $url
docker push $url

pushd ~/projects/mycloud/

./kubectl.sh set image \
  deployment/$(./kubectl.sh get deployment --output=jsonpath={.items..metadata.name} -l app=packager) \
  packager=$url