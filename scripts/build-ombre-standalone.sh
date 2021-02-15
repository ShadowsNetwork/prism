#!/bin/bash
# Loading binary/specs variables

docker build -f docker/ombre-standalone.Dockerfile \
  -t shadowsnetwork/ombre-standalone:lasest .
