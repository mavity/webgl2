# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/ast.rs | 0 | 1 | 1 | 0.00% 🟡 |
| src/decompiler/emitter.rs | 17 | 8 | 25 | 68.00% 🟡 |
| src/decompiler/lifter.rs | 17 | 21 | 38 | 44.74% 🟠 |
| src/decompiler/mod.rs | 8 | 6 | 14 | 57.14% 🟡 |
| src/decompiler/module.rs | 1 | 1 | 2 | 50.00% 🟡 |
| src/decompiler/parser.rs | 16 | 2 | 18 | 88.89% 🟢 |
| src/decompiler/simplifier.rs | 55 | 17 | 72 | 76.39% 🟡 |
| src/error.rs | 9 | 14 | 23 | 39.13% 🟠 |
| src/lib.rs | 318 | 220 | 538 | 59.11% 🟡 |
| src/naga_wasm_backend/backend.rs | 62 | 2 | 64 | 96.88% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 18 | 0 | 18 | 100.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 54 | 16 | 70 | 77.14% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 20 | 0 | 20 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 67 | 107 | 174 | 38.51% 🟠 |
| src/naga_wasm_backend/function_abi.rs | 24 | 0 | 24 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 2 | 2 | 4 | 50.00% 🟡 |
| src/naga_wasm_backend/types.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/wasm_gl_emu/device.rs | 24 | 0 | 24 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 3 | 1 | 4 | 75.00% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 32 | 18 | 50 | 64.00% 🟡 |
| src/wasm_gl_emu/transfer.rs | 19 | 7 | 26 | 73.08% 🟡 |
| src/webgl2_context/blend.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/webgl2_context/buffers.rs | 23 | 1 | 24 | 95.83% 🟢 |
| src/webgl2_context/drawing.rs | 13 | 0 | 13 | 100.00% 🟢 |
| src/webgl2_context/framebuffers.rs | 12 | 0 | 12 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 6 | 1 | 7 | 85.71% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 111 | 12 | 123 | 90.24% 🟢 |
| src/webgl2_context/state.rs | 18 | 2 | 20 | 90.00% 🟢 |
| src/webgl2_context/textures.rs | 25 | 9 | 34 | 73.53% 🟡 |
| src/webgl2_context/types.rs | 16 | 0 | 16 | 100.00% 🟢 |
| src/webgl2_context/vaos.rs | 35 | 0 | 35 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgpu/backend.rs | 26 | 27 | 53 | 49.06% 🟠 |
| **Total** | **1094** | **496** | **1590** | **68.81% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 220/538 | [1047] `) -> u32 {` | 59.11% 🟡 |
| src/naga_wasm_backend/expressions.rs | 107/174 | [1504] `for j in 0..count {` | 38.51% 🟠 |
| src/webgpu/backend.rs | 27/53 | [984] `}` | 49.06% 🟠 |
| src/decompiler/lifter.rs | 21/38 | [455] `fn unary_op(&mut self, op: UnaryOp) {` | 44.74% 🟠 |
