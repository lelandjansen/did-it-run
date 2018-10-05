#!/usr/bin/env bash
set -e
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null && pwd)"
CONTAINER_TAG="did-it-run"

cargo test --verbose --all

codecov_validate_output=$(curl \
  --data-binary \
  @$DIR/.codecov.yml \
  https://codecov.io/validate)
if [[ $codecov_validate_output != *"Valid!"* ]]; then
  echo $codecov_validate_output
  exit 1
fi

docker build \
  --tag $CONTAINER_TAG \
  $DIR

run-parts \
  --verbose \
  --regex ".*_test.*" \
  --arg=$CONTAINER_TAG \
  $DIR/tests
