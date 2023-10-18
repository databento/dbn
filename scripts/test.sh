#! /usr/bin/env bash
cargo --version
for i in $(seq 1 3); do
  if cargo test --all-features; then
    exit 0
  fi
done
exit 1
