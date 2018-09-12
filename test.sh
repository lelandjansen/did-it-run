#!/usr/bin/env bash
set -e
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"
CONTAINER_TAG="did-it-run"

cargo test --verbose --all

docker build \
  --tag $CONTAINER_TAG \
  $DIR

run-parts \
  --verbose \
  --regex ".*_test.*" \
  --arg=$CONTAINER_TAG \
  $DIR/tests
