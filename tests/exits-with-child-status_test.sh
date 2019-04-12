#!/usr/bin/env bash
BINARY_NAME=$1

for code in `seq 0 5`; do
  $("$BINARY_NAME" tests/fixtures/exit-with-code.sh "$code")
  result=$?
  if [[ "$code" -ne "$result" ]]; then
    exit 1
  fi 
done
