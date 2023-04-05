#! /usr/bin/env bash
set -e

cargo clippy --all-features -- --deny warnings
# `cargo doc` does not have a `--deny warnings` flag like clippy, workaround from:
# https://github.com/rust-lang/cargo/issues/8424#issuecomment-1070988443
RUSTDOCFLAGS='--deny warnings' cargo doc --all-features
