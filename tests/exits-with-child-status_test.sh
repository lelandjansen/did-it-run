#!/usr/bin/env bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"
BINARY_NAME=$1
shift

for code in `seq 0 5`; do
  result="$($BINARY_NAME $@ $DIR/fixtures/exit-with-code.sh $code)"
  if [[ "$code" -ne "$result" ]]; then
    exit 1
  fi 
done
