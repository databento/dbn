#! /usr/bin/env bash
set -e

# `cargo doc` does not have a `--deny warnings` flag like clippy, workaround from:
# https://github.com/rust-lang/cargo/issues/8424#issuecomment-1070988443
export RUSTDOCFLAGS='-D warnings'

cargo clippy --all-features -- --deny warnings
cargo doc --all-features
