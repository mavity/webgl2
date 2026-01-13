# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/emitter.rs | 15 | 7 | 22 | 68.18% 🟡 |
| src/decompiler/lifter.rs | 18 | 21 | 39 | 46.15% 🟠 |
| src/decompiler/mod.rs | 8 | 5 | 13 | 61.54% 🟡 |
| src/decompiler/module.rs | 4 | 1 | 5 | 80.00% 🟢 |
| src/decompiler/parser.rs | 16 | 0 | 16 | 100.00% 🟢 |
| src/decompiler/simplifier.rs | 49 | 17 | 66 | 74.24% 🟡 |
| src/error.rs | 6 | 17 | 23 | 26.09% 🟠 |
| src/lib.rs | 285 | 203 | 488 | 58.40% 🟡 |
| src/naga_wasm_backend/backend.rs | 54 | 1 | 55 | 98.18% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 5 | 4 | 9 | 55.56% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 29 | 15 | 44 | 65.91% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 21 | 0 | 21 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 48 | 23 | 71 | 67.61% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 12 | 11 | 23 | 52.17% 🟡 |
| src/naga_wasm_backend/functions/prep.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 2 | 2 | 4 | 50.00% 🟡 |
| src/naga_wasm_backend/types.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 14 | 0 | 14 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 16 | 1 | 17 | 94.12% 🟢 |
| src/webgl2_context/drawing.rs | 31 | 6 | 37 | 83.78% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 86 | 15 | 101 | 85.15% 🟢 |
| src/webgl2_context/state.rs | 12 | 3 | 15 | 80.00% 🟢 |
| src/webgl2_context/textures.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 9 | 1 | 10 | 90.00% 🟢 |
| src/webgl2_context/vaos.rs | 35 | 1 | 36 | 97.22% 🟢 |
| src/webgpu/adapter.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/webgpu/backend.rs | 25 | 25 | 50 | 50.00% 🟡 |
| **Total** | **851** | **380** | **1231** | **69.13% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 203/488 | [1042] `}` | 58.40% 🟡 |
| src/webgpu/backend.rs | 25/50 | [919] `}` | 50.00% 🟡 |
| src/naga_wasm_backend/expressions.rs | 23/71 | [888] `let var = &ctx.module.global_variables[*handle];` | 67.61% 🟡 |
| src/decompiler/lifter.rs | 21/39 | [455] `fn unary_op(&mut self, op: UnaryOp) {` | 46.15% 🟠 |
