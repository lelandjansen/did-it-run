#!/usr/bin/env bash
set -e
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
openssl genrsa \
  -out "$DIR/test.key" \
  4096
openssl req \
  -config $DIR/csr.conf \
  -new \
  -key "$DIR/test.key" \
  -out "$DIR/test.csr"
openssl x509 \
  -req \
  -days 36500 \
  -in "$DIR/test.csr" \
  -signkey "$DIR/test.key" \
  -out "$DIR/test.crt"
