# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/emitter.rs | 16 | 7 | 23 | 69.57% 🟡 |
| src/decompiler/lifter.rs | 17 | 21 | 38 | 44.74% 🟠 |
| src/decompiler/mod.rs | 9 | 5 | 14 | 64.29% 🟡 |
| src/decompiler/module.rs | 1 | 1 | 2 | 50.00% 🟡 |
| src/decompiler/parser.rs | 17 | 0 | 17 | 100.00% 🟢 |
| src/decompiler/simplifier.rs | 50 | 17 | 67 | 74.63% 🟡 |
| src/error.rs | 6 | 18 | 24 | 25.00% 🟠 |
| src/lib.rs | 285 | 203 | 488 | 58.40% 🟡 |
| src/naga_wasm_backend/backend.rs | 54 | 1 | 55 | 98.18% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 5 | 12 | 17 | 29.41% 🟠 |
| src/naga_wasm_backend/control_flow.rs | 29 | 15 | 44 | 65.91% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 21 | 0 | 21 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 48 | 23 | 71 | 67.61% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 11 | 11 | 22 | 50.00% 🟡 |
| src/naga_wasm_backend/functions/prep.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 2 | 2 | 4 | 50.00% 🟡 |
| src/naga_wasm_backend/types.rs | 6 | 5 | 11 | 54.55% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 14 | 0 | 14 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 16 | 1 | 17 | 94.12% 🟢 |
| src/webgl2_context/drawing.rs | 30 | 6 | 36 | 83.33% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 7 | 1 | 8 | 87.50% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 87 | 15 | 102 | 85.29% 🟢 |
| src/webgl2_context/state.rs | 13 | 3 | 16 | 81.25% 🟢 |
| src/webgl2_context/textures.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 8 | 1 | 9 | 88.89% 🟢 |
| src/webgl2_context/vaos.rs | 35 | 1 | 36 | 97.22% 🟢 |
| src/webgpu/adapter.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/webgpu/backend.rs | 25 | 24 | 49 | 51.02% 🟡 |
| **Total** | **851** | **394** | **1245** | **68.35% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 203/488 | [1042] `}` | 58.40% 🟡 |
| src/webgpu/backend.rs | 24/49 | [919] `}` | 51.02% 🟡 |
| src/naga_wasm_backend/expressions.rs | 23/71 | [888] `let var = &ctx.module.global_variables[*handle];` | 67.61% 🟡 |
| src/decompiler/lifter.rs | 21/38 | [455] `fn unary_op(&mut self, op: UnaryOp) {` | 44.74% 🟠 |
