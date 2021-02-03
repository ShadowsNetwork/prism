#!/usr/bin/env bash

VERSION=$1

if [[ -z "$1" ]] ; then
    echo "Usage: ./scripts/docker-hub-publish.sh VERSION"
    exit 1
fi

docker build . -t shadowsnetwork/shadows-parachain-testnet:$1 -t shadowsnetwork/shadows-parachain-testnet:latest
docker push shadowsnetwork/shadows-parachain-testnet:$1
docker push shadowsnetwork/shadows-parachain-testnet:latest
