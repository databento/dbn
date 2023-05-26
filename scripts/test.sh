#! /usr/bin/env bash
for i in $(seq 1 3); do
  if cargo test --all-features; then
    exit 0
  fi
done
exit 1
