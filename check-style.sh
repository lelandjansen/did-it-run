#!/usr/bin/env bash
STATUS=0
sort --check CONTRIBUTORS || STATUS=1
sort --check rustfmt.toml || STATUS=1
cargo +nightly fmt --all -- --check || STATUS=1
cargo +nightly clippy -- -D warnings || STATUS=1
exit $STATUS
