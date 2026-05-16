#!/usr/bin/env bash
# Build civium-core as a WebAssembly module for the browser.
# Output: website/src/www/wasm/civium_core{.js,_bg.wasm,.d.ts}
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../website/src/www/wasm"

echo "Building civium-core WASM..."
wasm-pack build \
  --target web \
  --out-dir "$OUT_DIR" \
  "$SCRIPT_DIR/civium-core" \
  -- --features wasm

echo "Done. Output in $OUT_DIR"
ls -lh "$OUT_DIR"
