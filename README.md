# WebGL2 Development Platform: VISION

A **Rust + WASM** based toolkit for debugging GLSL shaders and generating ergonomic WebGL2 bindings.

![WebGL2 Singing Dog Logo](./webgl2.png)

## 🎯 Quick Start

```bash
# Build the project
npm run build

# Run node tests
npm run tests
```

### Build via npm

This repository is both a Rust workspace and an npm package. Run the npm build helper to build the Rust workspace for the WASM target and copy produced .wasm files into `runners/wasm/`:

```bash
# from repository root
npm run build
```

## 🚀 Project Overview and Goals

The project aims to create a **Composite WebGL2 Development Platform** built with **Rust and WASM**. The primary objective is to significantly improve the WebGL2 developer experience by introducing standard software engineering practices—specifically, **robust debugging and streamlined resource management**—into the WebGL/GLSL workflow, which is currently hindered by hairy API, lack of debugging, incomprehensible errors and opaque GPU execution.

| Key Goals | Target Block | Value Proposition |
| :--- | :--- | :--- |
| **GPU Debugging** | Block 1 (Emulator) | Enable **step-through debugging**, breakpoints, and variable inspection for GLSL code. |
| **Unit Testing** | Block 1 (Emulator) | Provide a stable, deterministic environment for **automated testing** of graphics logic. |
| **Tech Stack** | Both | Utilize **Rust for safety and WASM for high-performance cross-platform execution** in the browser. |

-----

## 🏗️ Architecture

This project uses **Naga** (from the wgpu/WebGPU ecosystem) as the shader IR, rather than building a custom IR from scratch. This significantly reduces complexity while providing a proven, well-maintained foundation.

## 🔧 Development Status

**Current Phase: Phase 0 - Foundation Setup** ✅

- [x] Workspace structure created
- [x] Core crate skeletons implemented
- [x] Basic WASM backend (emits empty functions)
- [x] Runtime structure (wasmtime integration)
- [x] CLI tool with compile/validate/codegen/run commands
- [ ] DWARF debug information generation (in progress)
- [ ] Browser DevTools integration validation

## 📚 Documentation

- [`docs/1-plan.md`](docs/1-plan.md) - Original project proposal and plan
- [`docs/1.1-ir-wasm.md`](docs/1.1-ir-wasm.md) - Naga IR → WASM architecture (recommended reading)

## 🧪 Testing

```bash
# Run Rust tests
cargo test

# Run JS tests
npm test

# Test with a simple shader (prototype, experimental)
cargo run --bin webgl2 -- compile tests/fixtures/simple.vert --debug -o output.wasm
cargo run --bin webgl2 -- run output.wasm
```

## 📄 License

MIT
