# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/ast.rs | 1 | 1 | 2 | 50.00% 🟡 |
| src/decompiler/emitter.rs | 17 | 8 | 25 | 68.00% 🟡 |
| src/decompiler/lifter.rs | 17 | 21 | 38 | 44.74% 🟠 |
| src/decompiler/mod.rs | 8 | 6 | 14 | 57.14% 🟡 |
| src/decompiler/module.rs | 1 | 1 | 2 | 50.00% 🟡 |
| src/decompiler/parser.rs | 16 | 2 | 18 | 88.89% 🟢 |
| src/decompiler/simplifier.rs | 55 | 17 | 72 | 76.39% 🟡 |
| src/error.rs | 8 | 15 | 23 | 34.78% 🟠 |
| src/lib.rs | 318 | 219 | 537 | 59.22% 🟡 |
| src/naga_wasm_backend/backend.rs | 62 | 2 | 64 | 96.88% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 18 | 0 | 18 | 100.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 54 | 16 | 70 | 77.14% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 20 | 0 | 20 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 65 | 105 | 170 | 38.24% 🟠 |
| src/naga_wasm_backend/function_abi.rs | 23 | 0 | 23 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 2 | 2 | 4 | 50.00% 🟡 |
| src/naga_wasm_backend/types.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/wasm_gl_emu/device.rs | 23 | 0 | 23 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 2 | 2 | 4 | 50.00% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 31 | 18 | 49 | 63.27% 🟡 |
| src/wasm_gl_emu/transfer.rs | 14 | 11 | 25 | 56.00% 🟡 |
| src/webgl2_context/blend.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/webgl2_context/buffers.rs | 23 | 1 | 24 | 95.83% 🟢 |
| src/webgl2_context/drawing.rs | 13 | 0 | 13 | 100.00% 🟢 |
| src/webgl2_context/framebuffers.rs | 12 | 0 | 12 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 111 | 12 | 123 | 90.24% 🟢 |
| src/webgl2_context/state.rs | 18 | 2 | 20 | 90.00% 🟢 |
| src/webgl2_context/textures.rs | 28 | 11 | 39 | 71.79% 🟡 |
| src/webgl2_context/types.rs | 16 | 1 | 17 | 94.12% 🟢 |
| src/webgl2_context/vaos.rs | 35 | 0 | 35 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 25 | 28 | 53 | 47.17% 🟠 |
| **Total** | **1084** | **502** | **1586** | **68.35% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 219/537 | [1047] `) -> u32 {` | 59.22% 🟡 |
| src/naga_wasm_backend/expressions.rs | 105/170 | [1504] `for j in 0..count {` | 38.24% 🟠 |
| src/webgpu/backend.rs | 28/53 | [982] `}` | 47.17% 🟠 |
| src/decompiler/lifter.rs | 21/38 | [455] `fn unary_op(&mut self, op: UnaryOp) {` | 44.74% 🟠 |
