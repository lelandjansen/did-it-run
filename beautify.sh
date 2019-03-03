#!/usr/bin/env bash
sort CONTRIBUTORS --output=CONTRIBUTORS
sort rustfmt.toml --output=rustfmt.toml
cargo +nightly fmt --all ||
  cargo +nightly-2019-02-26 fmt --all
