#!/usr/bin/env bash

VERSION=$1

if [[ -z "$1" ]] ; then
    echo "Usage: ./scripts/docker-hub-publish.sh VERSION"
    exit 1
fi

docker build . -t shadows/shadows-node:$1 -t shadows/shadows-node:latest
docker push shadows/shadows-node:$1
docker push shadows/shadows-node:latest
