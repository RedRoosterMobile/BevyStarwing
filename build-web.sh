#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

# The CLI must match the wasm-bindgen version in Cargo.lock.
version=$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/"/, "", $3); print $3; exit }' Cargo.lock)
if ! command -v wasm-bindgen >/dev/null || [[ "$(wasm-bindgen --version)" != "wasm-bindgen $version" ]]; then
  cargo install wasm-bindgen-cli --version "$version" --locked
fi
rustup target add wasm32-unknown-unknown
cargo build --profile web-release --locked --target wasm32-unknown-unknown --features web
mkdir -p dist
wasm-bindgen --target web --no-typescript --remove-name-section --remove-producers-section \
  --out-dir dist --out-name starwing \
  target/wasm32-unknown-unknown/web-release/starwing.wasm
if command -v wasm-opt >/dev/null; then
  wasm-opt -Os --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
    dist/starwing_bg.wasm -o dist/starwing_bg.optimized.wasm
  mv dist/starwing_bg.optimized.wasm dist/starwing_bg.wasm
else
  printf '\nOptional: install Binaryen (brew install binaryen) for additional wasm-opt compression.\n'
fi
cp web/index.html dist/index.html
# All game assets are embedded; nothing else needs to be uploaded.
rm -f starwing-web.zip
(cd dist && zip -q ../starwing-web.zip index.html starwing.js starwing_bg.wasm)
printf '\nUpload the contents of dist/ into any web subfolder (not the ZIP itself).\n'
