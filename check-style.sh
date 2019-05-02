#!/usr/bin/env bash
STATUS=0
sort --check CONTRIBUTORS || STATUS=1
sort --check rustfmt.toml || STATUS=1
if [ -z "$RUSTUP_TOOLCHAIN" ]; then
  cargo +nightly fmt --all -- --check || STATUS=1
else
  cargo fmt --all -- --check || STATUS=1
fi
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
if [ -z "$RUSTUP_TOOLCHAIN" ]; then
  cargo +nightly clippy -- -D warnings || STATUS=1
else
  cargo clippy -- -D warnings || STATUS=1
fi
exit $STATUS
