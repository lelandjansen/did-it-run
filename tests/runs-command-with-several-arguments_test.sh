#!/usr/bin/env bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null && pwd )"
BINARY_NAME=$1
shift

arg1="foo"
arg2="bar"
arg3="baz"
result="$($BINARY_NAME $@ echo $arg1 $arg2 $arg3)"
code=$?
result="$(tr -dc \"[[:print:]]\" <<< $result)" # remove non-printable characters
if [ "$result" != "$arg1 $arg2 $arg3" ]; then
  exit 1
fi
exit $((code))
