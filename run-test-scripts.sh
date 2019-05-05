#!/usr/bin/env bash
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null && pwd)"
BINARY_NAME="diditrun"
ARGS="--no-desktop"

for filename in $(find "$DIR/tests" -type f -name "*_test.sh"); do
  echo "$filename"
  bash -c "$filename $BINARY_NAME $ARGS"
done
