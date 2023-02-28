#!/bin/sh

cargo build --release --target=wasm32-unknown-unknown
echo "Optimizing wasm..."
wasm-opt -O3 target/wasm32-unknown-unknown/release/synthesizer.wasm -o docs/synthesizer-opt.wasm
wasm-bindgen docs/synthesizer-opt.wasm --out-dir docs/pkg
wasm-bindgen target/wasm32-unknown-unknown/release/synthesizer.wasm --out-dir docs/pkg --target web
rm docs/synthesizer-opt.wasm
