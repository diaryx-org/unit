#!/usr/bin/env bash
# The gallery for a browser, as a static site in one directory: what the
# Pages workflow publishes, and what any static server shows locally.
#
#     gallery/web/build.sh [out]        # default out: target/gallery-web
#     python3 -m http.server -d target/gallery-web
#
# Needs wasm-bindgen-cli at the version Cargo.lock has for wasm-bindgen, as
# the bindings and the crate must match.
set -euo pipefail
root=$(cd "$(dirname "$0")/../.." && pwd)
out=${1:-$root/target/gallery-web}

want=$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/version = |"/, ""); print; exit }' "$root/Cargo.lock")
have=$(wasm-bindgen --version | awk '{ print $2 }')
if [ "$want" != "$have" ]; then
  echo "wasm-bindgen $have is on PATH; Cargo.lock wants $want:" >&2
  echo "  cargo install --locked wasm-bindgen-cli --version $want" >&2
  exit 1
fi

cargo build --manifest-path "$root/Cargo.toml" -p unit-gallery --release \
  --target wasm32-unknown-unknown --no-default-features --features web

rm -rf "$out"
mkdir -p "$out"
wasm-bindgen --target web --no-typescript --out-dir "$out" \
  "$root/target/wasm32-unknown-unknown/release/unit-gallery.wasm"
cp "$root/gallery/web/index.html" "$out/"
# Pages runs Jekyll over a site unless told not to.
touch "$out/.nojekyll"
du -sh "$out"
