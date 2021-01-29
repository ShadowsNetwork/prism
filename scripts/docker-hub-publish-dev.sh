#!/usr/bin/env bash

VERSION=$(git rev-parse --short HEAD)

docker build . -t shadows/shadows-node:$VERSION --no-cache
docker push shadows/shadows-node:$VERSION
