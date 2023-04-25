#! /usr/bin/env bash
for i in $(seq 1 3); do
  if cargo test --features async,python; then
    break
  fi
done
