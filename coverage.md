# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/ast.rs | 0 | 1 | 1 | 0.00% 🟡 |
| src/decompiler/emitter.rs | 17 | 8 | 25 | 68.00% 🟡 |
| src/decompiler/lifter.rs | 17 | 21 | 38 | 44.74% 🟠 |
| src/decompiler/mod.rs | 8 | 6 | 14 | 57.14% 🟡 |
| src/decompiler/module.rs | 3 | 1 | 4 | 75.00% 🟡 |
| src/decompiler/parser.rs | 16 | 2 | 18 | 88.89% 🟢 |
| src/decompiler/simplifier.rs | 55 | 17 | 72 | 76.39% 🟡 |
| src/error.rs | 8 | 15 | 23 | 34.78% 🟠 |
| src/lib.rs | 314 | 211 | 525 | 59.81% 🟡 |
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
| src/wasm_gl_emu/device.rs | 14 | 0 | 14 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 27 | 18 | 45 | 60.00% 🟡 |
| src/wasm_gl_emu/transfer.rs | 13 | 6 | 19 | 68.42% 🟡 |
| src/webgl2_context/blend.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/webgl2_context/buffers.rs | 15 | 1 | 16 | 93.75% 🟢 |
| src/webgl2_context/drawing.rs | 19 | 0 | 19 | 100.00% 🟢 |
| src/webgl2_context/framebuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 111 | 12 | 123 | 90.24% 🟢 |
| src/webgl2_context/state.rs | 17 | 2 | 19 | 89.47% 🟢 |
| src/webgl2_context/textures.rs | 26 | 15 | 41 | 63.41% 🟡 |
| src/webgl2_context/types.rs | 19 | 1 | 20 | 95.00% 🟢 |
| src/webgl2_context/vaos.rs | 36 | 0 | 36 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 25 | 26 | 51 | 49.02% 🟠 |
| **Total** | **1064** | **489** | **1553** | **68.51% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 211/525 | [1040] `/// Get active attribute info.` | 59.81% 🟡 |
| src/naga_wasm_backend/expressions.rs | 105/170 | [1504] `for j in 0..count {` | 38.24% 🟠 |
| src/webgpu/backend.rs | 26/51 | [978] `}` | 49.02% 🟠 |
| src/decompiler/lifter.rs | 21/38 | [455] `fn unary_op(&mut self, op: UnaryOp) {` | 44.74% 🟠 |
