#!/usr/bin/env bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"
CONTAINER_TAG=$1

for code in `seq 0 5`; do
  (docker run \
    $CONTAINER_TAG:latest \
    tests/fixtures/exit-with-code.sh $code)
  result=$?
  if [[ "$code" -ne "$result" ]]; then
    exit 1
  fi 
done
