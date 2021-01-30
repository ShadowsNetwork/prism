#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

if [ -z $CI ] ; then
   rustup update nightly-2020-10-05
   rustup update stable
fi

rustup target add wasm32-unknown-unknown --toolchain nightly-2020-10-05
rustup default nightly-2020-10-05
