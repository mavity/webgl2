# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/ast.rs | 0 | 1 | 1 | 0.00% 🟡 |
| src/decompiler/emitter.rs | 17 | 8 | 25 | 68.00% 🟡 |
| src/decompiler/lifter.rs | 17 | 21 | 38 | 44.74% 🟠 |
| src/decompiler/mod.rs | 8 | 6 | 14 | 57.14% 🟡 |
| src/decompiler/module.rs | 1 | 1 | 2 | 50.00% 🟡 |
| src/decompiler/parser.rs | 17 | 2 | 19 | 89.47% 🟢 |
| src/decompiler/simplifier.rs | 53 | 19 | 72 | 73.61% 🟡 |
| src/error.rs | 7 | 15 | 22 | 31.82% 🟠 |
| src/lib.rs | 315 | 210 | 525 | 60.00% 🟡 |
| src/naga_wasm_backend/backend.rs | 62 | 2 | 64 | 96.88% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 18 | 0 | 18 | 100.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 54 | 16 | 70 | 77.14% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 20 | 0 | 20 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 65 | 105 | 170 | 38.24% 🟠 |
| src/naga_wasm_backend/function_abi.rs | 22 | 0 | 22 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 4 | 1 | 5 | 80.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 2 | 2 | 4 | 50.00% 🟡 |
| src/naga_wasm_backend/types.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 26 | 18 | 44 | 59.09% 🟡 |
| src/webgl2_context/blend.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/webgl2_context/buffers.rs | 18 | 1 | 19 | 94.74% 🟢 |
| src/webgl2_context/drawing.rs | 26 | 3 | 29 | 89.66% 🟢 |
| src/webgl2_context/framebuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 113 | 12 | 125 | 90.40% 🟢 |
| src/webgl2_context/state.rs | 23 | 1 | 24 | 95.83% 🟢 |
| src/webgl2_context/textures.rs | 28 | 17 | 45 | 62.22% 🟡 |
| src/webgl2_context/types.rs | 15 | 1 | 16 | 93.75% 🟢 |
| src/webgl2_context/vaos.rs | 35 | 0 | 35 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgpu/backend.rs | 25 | 27 | 52 | 48.08% 🟠 |
| **Total** | **1049** | **490** | **1539** | **68.16% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 210/525 | [1040] `/// Get active attribute info.` | 60.00% 🟡 |
| src/naga_wasm_backend/expressions.rs | 105/170 | [1504] `for j in 0..count {` | 38.24% 🟠 |
| src/webgpu/backend.rs | 27/52 | [965] `}` | 48.08% 🟠 |
| src/decompiler/lifter.rs | 21/38 | [455] `fn unary_op(&mut self, op: UnaryOp) {` | 44.74% 🟠 |
