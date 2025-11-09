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
  // Build args for cargo. Avoid passing '-C' via cargo's '--' which can be
  // unsupported on some cargo versions/environments. If link args are needed
  // they should be set via RUSTFLAGS or handled separately.
  const cargoArgs = ['build', '--workspace', '--target', target];

  // Use --release by default
  if (profile === 'release') cargoArgs.splice(1, 0, '--release');

  console.log('Running cargo to build WASM artifacts...');
  // Ensure the wasm target is installed (rustup-managed toolchains)
  try {
    const rustupCheck = spawnSync('rustup', ['target', 'list', '--installed'], { encoding: 'utf8' });
    if (rustupCheck.error) {
      if (rustupCheck.error.code === 'ENOENT') {
        console.error('rustup not found in PATH. Please install rustup (https://rustup.rs/) and ensure the wasm target is available:');
        console.error(`  rustup target add ${target}`);
        process.exit(1);
      }
    } else {
      const installed = rustupCheck.stdout || '';
      if (!installed.includes(target)) {
        console.log(`${target} not installed; attempting to install via rustup`);
        try {
          run('rustup', ['target', 'add', target]);
        } catch (e) {
          console.error('Failed to install wasm target via rustup:', e.message || e);
          console.error('Please run: rustup target add', target);
          process.exit(1);
        }
      } else {
        console.log(`${target} is already installed`);
      }
    }
  } catch (e) {
    // best effort; continue to cargo which will fail with a more specific error
    console.warn('Could not verify rustup target installation:', e && e.message ? e.message : e);
  }
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

  // Ensure destination: write wasm into repoRoot so the runtime can find the
  // single `webgl2.wasm` file next to `index.js`. We keep the copy out of git
  // by default (it's a build artifact).
  const outDir = repoRoot; // copy .wasm files directly into repo root

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

  console.log('\nDone. .wasm files copied to repo root (e.g. ./webgl2.wasm)');
  console.log('If you require wasm-bindgen output, run wasm-bindgen manually on the produced .wasm files.');
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
