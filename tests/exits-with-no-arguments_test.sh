#!/usr/bin/env bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"
CONTAINER_TAG=$1

result="$(docker run \
  --tty \
  $CONTAINER_TAG:latest \
  2>&1)"
code=$?
result="$(tr -dc \"[[:print:]]\" <<< $result)" # remove non-printable characters
if [[ "$result" != *"required arguments were not provided"* ]]; then
  exit 1
fi
if [ "$code" -eq "0" ]; then
  exit 1
fi
