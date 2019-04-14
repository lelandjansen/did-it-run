#!/usr/bin/env bash
BINARY_NAME=$1

result="$($BINARY_NAME 2>&1)"
if [[ "$result" == *"required arguments were not provided"* ]]; then
  exit 0
fi
exit 1
