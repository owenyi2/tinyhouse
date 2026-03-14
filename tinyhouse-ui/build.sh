#!/bin/bash
set -e

crate=tinyhouse_ui

cargo build --release --target wasm32-unknown-unknown --target-dir target
wasm-bindgen --target web --out-dir web target/wasm32-unknown-unknown/release/$crate.wasm
cp index.html web/index.html
cd web
python3 -m http.server 



