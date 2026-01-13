# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/emitter.rs | 15 | 8 | 23 | 65.22% 🟡 |
| src/decompiler/lifter.rs | 17 | 21 | 38 | 44.74% 🟠 |
| src/decompiler/mod.rs | 9 | 5 | 14 | 64.29% 🟡 |
| src/decompiler/module.rs | 1 | 1 | 2 | 50.00% 🟡 |
| src/decompiler/parser.rs | 17 | 0 | 17 | 100.00% 🟢 |
| src/decompiler/simplifier.rs | 51 | 16 | 67 | 76.12% 🟡 |
| src/error.rs | 7 | 16 | 23 | 30.43% 🟠 |
| src/lib.rs | 290 | 203 | 493 | 58.82% 🟡 |
| src/naga_wasm_backend/backend.rs | 54 | 1 | 55 | 98.18% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 18 | 0 | 18 | 100.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 29 | 15 | 44 | 65.91% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 21 | 0 | 21 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 55 | 16 | 71 | 77.46% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 23 | 0 | 23 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 2 | 2 | 4 | 50.00% 🟡 |
| src/naga_wasm_backend/types.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 17 | 6 | 23 | 73.91% 🟡 |
| src/webgl2_context/blend.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/webgl2_context/buffers.rs | 16 | 1 | 17 | 94.12% 🟢 |
| src/webgl2_context/drawing.rs | 30 | 6 | 36 | 83.33% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 5 | 1 | 6 | 83.33% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 87 | 14 | 101 | 86.14% 🟢 |
| src/webgl2_context/state.rs | 15 | 1 | 16 | 93.75% 🟢 |
| src/webgl2_context/textures.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 8 | 1 | 9 | 88.89% 🟢 |
| src/webgl2_context/vaos.rs | 36 | 0 | 36 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 25 | 24 | 49 | 51.02% 🟡 |
| **Total** | **900** | **359** | **1259** | **71.49% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 203/493 | [1044] `) -> u32 {` | 58.82% 🟡 |
| src/webgpu/backend.rs | 24/49 | [923] `}` | 51.02% 🟡 |
| src/decompiler/lifter.rs | 21/38 | [455] `fn unary_op(&mut self, op: UnaryOp) {` | 44.74% 🟠 |
| src/decompiler/simplifier.rs | 16/67 | [112] `let get_const = |id: &Id| egraph[*id].data.constant;` | 76.12% 🟡 |
