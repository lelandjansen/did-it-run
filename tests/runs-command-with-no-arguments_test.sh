#!/usr/bin/env bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"
BINARY_NAME=$1

result="$($BINARY_NAME echo)"
code=$?
result="$(tr -dc \"[[:print:]]\" <<< $result)" # remove non-printable characters
if [ "$result" != "" ]; then
  exit 1
fi
exit "$code"
