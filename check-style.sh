#!/usr/bin/env bash
STATUS=0
sort --check CONTRIBUTORS || STATUS=1
sort --check rustfmt.toml || STATUS=1
# Fall back to an older nightly if the current version doesn't have rustfmt and
# clippy
cargo +nightly fmt --all -- --check ||
  cargo +nightly-2019-02-26 fmt --all -- --check ||
  STATUS=1
cargo +nightly clippy -- -D warnings ||
  cargo +nightly-2019-02-26 clippy -- -D warnings ||
  STATUS=1
exit $STATUS
