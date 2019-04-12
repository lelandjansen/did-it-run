#!/usr/bin/env bash
BINARY_NAME=$1

result=$("$BINARY_NAME" 2>&1)
code=$?
result="$(tr -dc \"[[:print:]]\" <<< $result)" # remove non-printable characters
if [[ "$result" != *"required arguments were not provided"* ]]; then
  exit 1
fi
if [ "$code" -eq "0" ]; then
  exit 1
fi
