#!/usr/bin/env node
// build-wasm.js
// Cross-platform Node build script that builds Rust workspace for wasm32
// and copies resulting .wasm files into runners/wasm/.

const { spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

function run(cmd, args, opts = {}) {
  const res = spawnSync(cmd, args, { stdio: 'inherit', shell: false, ...opts });
  if (res.error) throw res.error;
  if (res.status !== 0) throw new Error(`${cmd} ${args.join(' ')} failed with status ${res.status}`);
}

function copyFileSync(src, dest) {
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  fs.copyFileSync(src, dest);
  console.log(`copied ${src} -> ${dest}`);
}

async function main() {
  const repoRoot = path.resolve(__dirname, '..');
  const target = process.env.RUST_WASM_TARGET || 'wasm32-unknown-unknown';
  const profile = process.env.RUST_PROFILE || 'release';
  const cargoArgs = ['build', '--workspace', '--target', target, '--' , '-C', 'link-args='];

  // Use --release by default
  if (profile === 'release') cargoArgs.splice(1, 0, '--release');

  console.log('Running cargo to build WASM artifacts...');
  try {
    // we run in repoRoot
    run('cargo', cargoArgs, { cwd: repoRoot });
  } catch (e) {
    console.error('cargo build failed:', e.message || e);
    process.exit(1);
  }

  const builtDir = path.join(repoRoot, 'target', target, profile);
  if (!fs.existsSync(builtDir)) {
    console.error('Expected build output at', builtDir, 'but it does not exist.');
    process.exit(1);
  }

  // Ensure destination
  const outDir = path.join(repoRoot, 'runners', 'wasm');
  fs.mkdirSync(outDir, { recursive: true });

  // Copy all .wasm files from builtDir into runners/wasm, preserving filename
  const files = fs.readdirSync(builtDir).filter(f => f.endsWith('.wasm'));
  if (files.length === 0) {
    console.warn('No .wasm files found in', builtDir);
  }
  for (const f of files) {
    const src = path.join(builtDir, f);
    const dest = path.join(outDir, f);
    copyFileSync(src, dest);
  }

  console.log('\nDone. .wasm files copied to runners/wasm/');
  console.log('If you require wasm-bindgen output, run wasm-bindgen manually on the produced .wasm files.');
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
