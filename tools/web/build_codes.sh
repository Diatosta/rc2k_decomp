#!/bin/bash
# The saves page's save code module (`tools/web/codes/`): the game's own save code functions as a
# small WebAssembly module for the browser, built for `wasm32-unknown-unknown` (no Emscripten).
# Output: target/web_codes/wasm32-unknown-unknown/release/rc2k_web_codes.wasm.
set -e
cd "$(dirname "$0")/../.."
rustup target list --installed | grep -qx wasm32-unknown-unknown || rustup target add wasm32-unknown-unknown
# Small is better than fast here: the module runs a QR code at a time.
CARGO_TARGET_DIR="${CODES_TARGET_DIR:-target/web_codes}" RUSTFLAGS="-C opt-level=s -C strip=symbols" \
  cargo build --release --target wasm32-unknown-unknown -p rc2k_web_codes "$@"
