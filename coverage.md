# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/emitter.rs | 16 | 7 | 23 | 69.57% 🟡 |
| src/decompiler/lifter.rs | 18 | 21 | 39 | 46.15% 🟠 |
| src/decompiler/mod.rs | 8 | 5 | 13 | 61.54% 🟡 |
| src/decompiler/module.rs | 4 | 1 | 5 | 80.00% 🟢 |
| src/decompiler/parser.rs | 16 | 0 | 16 | 100.00% 🟢 |
| src/decompiler/simplifier.rs | 51 | 16 | 67 | 76.12% 🟡 |
| src/error.rs | 6 | 17 | 23 | 26.09% 🟠 |
| src/lib.rs | 290 | 199 | 489 | 59.30% 🟡 |
| src/naga_wasm_backend/backend.rs | 54 | 2 | 56 | 96.43% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 5 | 4 | 9 | 55.56% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 29 | 15 | 44 | 65.91% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 21 | 0 | 21 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 47 | 23 | 70 | 67.14% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 13 | 11 | 24 | 54.17% 🟡 |
| src/naga_wasm_backend/mod.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 2 | 2 | 4 | 50.00% 🟡 |
| src/naga_wasm_backend/types.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 14 | 0 | 14 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 16 | 1 | 17 | 94.12% 🟢 |
| src/webgl2_context/drawing.rs | 30 | 6 | 36 | 83.33% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 7 | 1 | 8 | 87.50% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 87 | 14 | 101 | 86.14% 🟢 |
| src/webgl2_context/state.rs | 14 | 1 | 15 | 93.33% 🟢 |
| src/webgl2_context/textures.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 9 | 1 | 10 | 90.00% 🟢 |
| src/webgl2_context/vaos.rs | 36 | 0 | 36 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 25 | 25 | 50 | 50.00% 🟡 |
| **Total** | **855** | **372** | **1227** | **69.68% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 199/489 | [1042] `}` | 59.30% 🟡 |
| src/webgpu/backend.rs | 25/50 | [919] `}` | 50.00% 🟡 |
| src/naga_wasm_backend/expressions.rs | 23/70 | [886] `let var = &ctx.module.global_variables[*handle];` | 67.14% 🟡 |
| src/decompiler/lifter.rs | 21/39 | [454] `fn unary_op(&mut self, op: UnaryOp) {` | 46.15% 🟠 |
