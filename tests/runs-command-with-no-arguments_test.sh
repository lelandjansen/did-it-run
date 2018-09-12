#!/usr/bin/env bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"
CONTAINER_TAG=$1

result="$(docker run \
  --tty \
  $CONTAINER_TAG:latest \
  echo)"
code=$?
result="$(tr -dc \"[[:print:]]\" <<< $result)" # remove non-printable characters
if [ "$result" != "" ]; then
  exit 1
fi
exit "$code"
