#!/usr/bin/env bash
set -e
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
if [ "$(uname -s)" == "Linux" ]; then
  mkdir -p /usr/local/share/ca-certificates/test
  cp "$DIR/test.crt" /usr/local/share/ca-certificates/test/
  update-ca-certificates
elif [ "$(uname)" == "Darwin" ]; then
  security add-trusted-cert \
    -d \
    -r trustRoot \
    -k /Library/Keychains/System.keychain \
    "$DIR/test.crt"
elif [ "$(uname)" == "MSYS*" ]; then
  certutil -addstore -f "ROOT" "$DIR/test.crt"
fi
