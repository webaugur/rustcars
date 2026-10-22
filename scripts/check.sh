#!/usr/bin/env bash
# Local, CI-free check: formatting, clippy for all three build modes, tests.
# Usage: scripts/check.sh   (needs: rustup target add wasm32-unknown-unknown)
set -euo pipefail
cd "$(dirname "$0")/.."
cargo fmt --check
cargo clippy --all-targets -- -D warnings                                   # desktop (default)
cargo clippy --features web --target wasm32-unknown-unknown -- -D warnings  # browser build
cargo clippy --features web-server --all-targets -- -D warnings             # optional server
cargo test
cargo test --features web-server --bin rustcars-web
# Default build must not pull web deps:
if cargo tree -e normal | grep -E '^\S*.*(axum|tokio|wasm-bindgen-futures) ' >/dev/null; then
  echo "default build pulls web dependencies" >&2; exit 1
fi
echo "all checks passed"
