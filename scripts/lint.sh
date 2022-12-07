#! /usr/bin/env bash
set -e
cargo clippy --all-features -- --deny warnings
cargo fmt --check # fails if anything is misformatted
cargo doc --all-features
