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
| src/decompiler/simplifier.rs | 56 | 18 | 74 | 75.68% 🟡 |
| src/error.rs | 8 | 15 | 23 | 34.78% 🟠 |
| src/lib.rs | 308 | 196 | 504 | 61.11% 🟡 |
| src/naga_wasm_backend/backend.rs | 56 | 2 | 58 | 96.55% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 17 | 0 | 17 | 100.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 29 | 15 | 44 | 65.91% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 21 | 0 | 21 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 55 | 16 | 71 | 77.46% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 24 | 0 | 24 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 2 | 2 | 4 | 50.00% 🟡 |
| src/naga_wasm_backend/types.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 35 | 13 | 48 | 72.92% 🟡 |
| src/webgl2_context/blend.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/webgl2_context/buffers.rs | 18 | 1 | 19 | 94.74% 🟢 |
| src/webgl2_context/drawing.rs | 32 | 6 | 38 | 84.21% 🟢 |
| src/webgl2_context/framebuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 5 | 1 | 6 | 83.33% 🟢 |
| src/webgl2_context/renderbuffers.rs | 12 | 0 | 12 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 109 | 14 | 123 | 88.62% 🟢 |
| src/webgl2_context/state.rs | 21 | 1 | 22 | 95.45% 🟢 |
| src/webgl2_context/textures.rs | 23 | 9 | 32 | 71.88% 🟡 |
| src/webgl2_context/types.rs | 10 | 1 | 11 | 90.91% 🟢 |
| src/webgl2_context/vaos.rs | 36 | 0 | 36 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 25 | 25 | 50 | 50.00% 🟡 |
| **Total** | **996** | **375** | **1371** | **72.65% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 196/504 | [1040] `// ---- GLSL Decompiler Support (docs/11.b-decompile-theo...` | 61.11% 🟡 |
| src/webgpu/backend.rs | 25/50 | [943] `}` | 50.00% 🟡 |
| src/decompiler/lifter.rs | 21/38 | [455] `fn unary_op(&mut self, op: UnaryOp) {` | 44.74% 🟠 |
| src/decompiler/simplifier.rs | 18/74 | [115] `let get_const = |id: &Id| egraph[*id].data.constant;` | 75.68% 🟡 |
