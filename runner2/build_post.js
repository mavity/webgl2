const fs = require('fs');
const path = require('path');

// move built wasm file from target to runner2/runner.wasm
const repoRoot = path.resolve(__dirname);
// Use package-local target (we pass --target-dir ./target during cargo build)
const builtPath = path.join(repoRoot, 'target', 'wasm32-unknown-unknown', 'release');
if (!fs.existsSync(builtPath)) {
  console.log('Rust build directory not found; did you run npm run build:rust?');
  process.exit(0);
}

const files = fs.readdirSync(builtPath).filter(f => f.endsWith('.wasm'));
if (files.length === 0) {
  console.log('No wasm files found in build directory.');
  process.exit(0);
}

// Prefer wasm files that contain 'runner2' (our crate name) or 'runner' so we don't pick unrelated
// wasm artifacts like webgl2.wasm accidentally.
// Prefer exact crate output name `runner2_runner.wasm` if present
let wasmFileName = files.find(f => f === 'runner2_runner.wasm') || files.find(f => f.includes('runner2')) || files.find(f => f.includes('runner')) || files[0];
const wasmFile = path.join(builtPath, wasmFileName);
const dest = path.join(repoRoot, 'runner.wasm');
fs.copyFileSync(wasmFile, dest);
console.log('Copied', wasmFile, '->', dest);
