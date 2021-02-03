#!/usr/bin/env bash

set -e

# cargo clean
WASM_BUILD_TYPE=release cargo run -- build-spec --chain shadows-latest > ./resources/shadows.json
WASM_BUILD_TYPE=release cargo run -- build-spec --chain ./resources/shadows.json --raw > ./resources/shadows-dist.json
