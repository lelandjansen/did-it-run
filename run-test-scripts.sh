#!/usr/bin/env bash
set -e
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null && pwd)"
BINARY_NAME="diditrun"

run-parts \
  --verbose \
  --regex ".*_test.*" \
  --arg="$BINARY_NAME" \
  "$DIR/tests"
