# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/ast.rs | 0 | 1 | 1 | 0.00% 🟡 |
| src/decompiler/emitter.rs | 17 | 9 | 26 | 65.38% 🟡 |
| src/decompiler/lifter.rs | 17 | 22 | 39 | 43.59% 🟠 |
| src/decompiler/mod.rs | 8 | 6 | 14 | 57.14% 🟡 |
| src/decompiler/module.rs | 1 | 1 | 2 | 50.00% 🟡 |
| src/decompiler/parser.rs | 16 | 2 | 18 | 88.89% 🟢 |
| src/decompiler/simplifier.rs | 55 | 18 | 73 | 75.34% 🟡 |
| src/error.rs | 6 | 16 | 22 | 27.27% 🟠 |
| src/lib.rs | 313 | 216 | 529 | 59.17% 🟡 |
| src/naga_wasm_backend/backend.rs | 60 | 2 | 62 | 96.77% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 18 | 0 | 18 | 100.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 54 | 16 | 70 | 77.14% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 20 | 0 | 20 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 64 | 106 | 170 | 37.65% 🟠 |
| src/naga_wasm_backend/function_abi.rs | 24 | 0 | 24 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 4 | 1 | 5 | 80.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 2 | 2 | 4 | 50.00% 🟡 |
| src/naga_wasm_backend/types.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 30 | 13 | 43 | 69.77% 🟡 |
| src/webgl2_context/blend.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/webgl2_context/buffers.rs | 18 | 1 | 19 | 94.74% 🟢 |
| src/webgl2_context/drawing.rs | 26 | 3 | 29 | 89.66% 🟢 |
| src/webgl2_context/framebuffers.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 6 | 1 | 7 | 85.71% 🟢 |
| src/webgl2_context/renderbuffers.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 111 | 12 | 123 | 90.24% 🟢 |
| src/webgl2_context/state.rs | 23 | 2 | 25 | 92.00% 🟢 |
| src/webgl2_context/textures.rs | 24 | 11 | 35 | 68.57% 🟡 |
| src/webgl2_context/types.rs | 12 | 1 | 13 | 92.31% 🟢 |
| src/webgl2_context/vaos.rs | 36 | 0 | 36 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/webgpu/backend.rs | 25 | 27 | 52 | 48.08% 🟠 |
| **Total** | **1039** | **491** | **1530** | **67.91% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 216/529 | [1039] `)` | 59.17% 🟡 |
| src/naga_wasm_backend/expressions.rs | 106/170 | [1492] `for j in 0..count {` | 37.65% 🟠 |
| src/webgpu/backend.rs | 27/52 | [965] `}` | 48.08% 🟠 |
| src/decompiler/lifter.rs | 22/39 | [455] `fn unary_op(&mut self, op: UnaryOp) {` | 43.59% 🟠 |
