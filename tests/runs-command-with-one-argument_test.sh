#!/usr/bin/env bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"
BINARY_NAME=$1

arg1="foo"
result="$($BINARY_NAME echo $arg1)"
code=$?
result="$(tr -dc \"[[:print:]]\" <<< $result)" # remove non-printable characters
if [ "$result" != "$arg1" ]; then
  exit 1
fi
exit "$code"
