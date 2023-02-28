#!/bin/sh

cargo +nightly build --release --target=wasm32-unknown-unknown
echo "Optimizing wasm..."
wasm-opt -O3 target/wasm32-unknown-unknown/release/synthesizer.wasm -o docs/synthesizer-opt.wasm
wasm-bindgen docs/synthesizer-opt.wasm --out-dir docs/pkg
wasm-bindgen target/wasm32-unknown-unknown/release/synthesizer.wasm --out-dir docs/pkg --target no-modules
rm docs/synthesizer-opt.wasm


echo '
const init = wasm_bindgen;
const {synthesize} = init;
const prom = init();
self.addEventListener("message", async (e) => {
  const {inps, outs, tests} = e.data;
  await prom;
  const result = synthesize(inps, outs, tests);

  self.postMessage(result);
});
' >> docs/pkg/synthesizer.js
