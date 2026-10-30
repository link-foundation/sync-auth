#!/usr/bin/env bash
# Before the scoped allowances, Rust 1.99 reports four double_must_use errors.
# The same command is the regression check after the fix.
set -euo pipefail
cd "$(dirname "$0")/.."
CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 RUSTFLAGS=-Dwarnings \
  cargo clippy --all-targets --all-features
