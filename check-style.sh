#!/usr/bin/env bash
STATUS=0
sort --check CONTRIBUTORS || STATUS=1
sort --check rustfmt.toml || STATUS=1
# Fall back to an older nightly if the current version doesn't have rustfmt and
# clippy
cargo +nightly fmt --all -- --check || STATUS=1
! git \
  --no-pager \
  grep \
    --untracked \
    --ignore-case \
    --before-context 1 \
    --after-context 2 \
    --perl-regexp \
    -- \
      '.*(TODO|FIXME)(?!\(#\d+\)).*' \
      ':!*.rs' \
      ':!rustfmt.toml' \
      ':!check-style.sh' ||
  STATUS=1
cargo +nightly clippy -- -D warnings || STATUS=1
exit $STATUS
