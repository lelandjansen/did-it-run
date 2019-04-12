#!/usr/bin/env bash
set -e
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
mkdir -p /usr/local/share/ca-certificates/test
cp "$DIR/test.crt" /usr/local/share/ca-certificates/test/
update-ca-certificates
