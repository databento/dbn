#! /usr/bin/env bash
set -e
cargo --version
cargo test --all-features
cargo run --example v4_compatibility --all-features
