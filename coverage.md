# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/ast.rs | 0 | 1 | 1 | 0.00% 🟡 |
| src/decompiler/emitter.rs | 17 | 8 | 25 | 68.00% 🟡 |
| src/decompiler/lifter.rs | 17 | 21 | 38 | 44.74% 🟠 |
| src/decompiler/mod.rs | 8 | 6 | 14 | 57.14% 🟡 |
| src/decompiler/module.rs | 3 | 1 | 4 | 75.00% 🟡 |
| src/decompiler/parser.rs | 15 | 2 | 17 | 88.24% 🟢 |
| src/decompiler/simplifier.rs | 56 | 17 | 73 | 76.71% 🟡 |
| src/error.rs | 8 | 17 | 25 | 32.00% 🟠 |
| src/lib.rs | 326 | 215 | 541 | 60.26% 🟡 |
| src/naga_wasm_backend/backend.rs | 62 | 2 | 64 | 96.88% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 18 | 0 | 18 | 100.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 54 | 16 | 70 | 77.14% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 20 | 0 | 20 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 67 | 107 | 174 | 38.51% 🟠 |
| src/naga_wasm_backend/function_abi.rs | 25 | 0 | 25 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 4 | 2 | 6 | 66.67% 🟡 |
| src/naga_wasm_backend/types.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/wasm_gl_emu/device.rs | 24 | 0 | 24 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 3 | 1 | 4 | 75.00% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 32 | 18 | 50 | 64.00% 🟡 |
| src/wasm_gl_emu/transfer.rs | 19 | 7 | 26 | 73.08% 🟡 |
| src/webgl2_context/blend.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/webgl2_context/buffers.rs | 23 | 1 | 24 | 95.83% 🟢 |
| src/webgl2_context/drawing.rs | 13 | 0 | 13 | 100.00% 🟢 |
| src/webgl2_context/framebuffers.rs | 12 | 0 | 12 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 114 | 12 | 126 | 90.48% 🟢 |
| src/webgl2_context/state.rs | 18 | 2 | 20 | 90.00% 🟢 |
| src/webgl2_context/textures.rs | 25 | 9 | 34 | 73.53% 🟡 |
| src/webgl2_context/types.rs | 16 | 0 | 16 | 100.00% 🟢 |
| src/webgl2_context/vaos.rs | 35 | 0 | 35 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 69 | 26 | 95 | 72.63% 🟡 |
| src/webgpu/command.rs | 3 | 1 | 4 | 75.00% 🟡 |
| **Total** | **1156** | **493** | **1649** | **70.10% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 215/541 | [1033] `/// Use a program.` | 60.26% 🟡 |
| src/naga_wasm_backend/expressions.rs | 107/174 | [1504] `for j in 0..count {` | 38.51% 🟠 |
| src/webgpu/backend.rs | 26/95 | [1404] `}` | 72.63% 🟡 |
| src/decompiler/lifter.rs | 21/38 | [455] `fn unary_op(&mut self, op: UnaryOp) {` | 44.74% 🟠 |
