#!/usr/bin/env bash
set -euo pipefail

echo "Setting up wasm test environment..."

if ! command -v wasm-bindgen-test-runner >/dev/null 2>&1; then
  echo "wasm-bindgen-test-runner not found. Installing wasm-bindgen-cli (provides runner)..."
  cargo install --locked wasm-bindgen-cli || true
else
  echo "wasm-bindgen-test-runner is already installed."
fi

echo "Ensuring wasm32 target is installed..."
rustup target add wasm32-unknown-unknown || true

echo "Setup complete. Run 'cargo test --target wasm32-unknown-unknown -v' or simply 'cargo test -v' (repo default target may use wasm)."
