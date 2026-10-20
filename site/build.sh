#!/bin/sh
# Builds the wasm the playground loads into site/pkg. Open site/index.html straight from disk or serve site/.
# Needs the wasm32-unknown-unknown target and wasm-bindgen-cli at the version in Cargo.lock.
set -e
cd "$(dirname "$0")/.."
cargo build -p mathra-wasm --release --target wasm32-unknown-unknown
rm -rf site/pkg
wasm-bindgen --target no-modules --no-typescript --out-dir site/pkg target/wasm32-unknown-unknown/release/mathra_wasm.wasm
# file:// pages cannot fetch the .wasm, but they can run a classic script: the same bytes as base64
printf 'var MATHRA_WASM_BASE64 = "%s";\n' "$(base64 < site/pkg/mathra_wasm_bg.wasm | tr -d '\n')" > site/pkg/mathra_wasm_bytes.js
