#!/bin/bash
cargo fix --workspace --allow-staged --allow-dirty
cargo clippy --all
cargo test --all --all-features -- --nocapture