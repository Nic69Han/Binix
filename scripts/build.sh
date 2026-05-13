#!/usr/bin/env bash
set -e
cargo build --release --workspace
strip target/release/binix-app 2>/dev/null || true
echo "✅ Build successful: target/release/binix-app"
