#!/usr/bin/env bash
cp ../../target/release/shadows-node bin/shadows-node
docker build -t shadowsnetwork/shadows-parachain-devnet:1.0.0 -t shadowsnetwork/shadows-parachain-devnet:latest .