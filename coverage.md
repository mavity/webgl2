# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/emitter.rs | 16 | 7 | 23 | 69.57% 🟡 |
| src/decompiler/lifter.rs | 18 | 21 | 39 | 46.15% 🟠 |
| src/decompiler/mod.rs | 9 | 5 | 14 | 64.29% 🟡 |
| src/decompiler/module.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/decompiler/parser.rs | 17 | 0 | 17 | 100.00% 🟢 |
| src/decompiler/simplifier.rs | 50 | 17 | 67 | 74.63% 🟡 |
| src/error.rs | 6 | 17 | 23 | 26.09% 🟠 |
| src/lib.rs | 299 | 198 | 497 | 60.16% 🟡 |
| src/naga_wasm_backend/backend.rs | 54 | 1 | 55 | 98.18% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 17 | 0 | 17 | 100.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 29 | 15 | 44 | 65.91% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 21 | 0 | 21 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 55 | 16 | 71 | 77.46% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 23 | 0 | 23 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 2 | 2 | 4 | 50.00% 🟡 |
| src/naga_wasm_backend/types.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 34 | 13 | 47 | 72.34% 🟡 |
| src/webgl2_context/blend.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/webgl2_context/buffers.rs | 16 | 1 | 17 | 94.12% 🟢 |
| src/webgl2_context/drawing.rs | 30 | 6 | 36 | 83.33% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 87 | 14 | 101 | 86.14% 🟢 |
| src/webgl2_context/state.rs | 20 | 1 | 21 | 95.24% 🟢 |
| src/webgl2_context/textures.rs | 10 | 1 | 11 | 90.91% 🟢 |
| src/webgl2_context/types.rs | 9 | 1 | 10 | 90.00% 🟢 |
| src/webgl2_context/vaos.rs | 36 | 0 | 36 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 25 | 24 | 49 | 51.02% 🟡 |
| **Total** | **934** | **362** | **1296** | **72.07% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 198/497 | [1037] `) -> u32 {` | 60.16% 🟡 |
| src/webgpu/backend.rs | 24/49 | [943] `}` | 51.02% 🟡 |
| src/decompiler/lifter.rs | 21/39 | [455] `fn unary_op(&mut self, op: UnaryOp) {` | 46.15% 🟠 |
| src/decompiler/simplifier.rs | 17/67 | [90] `#[derive(Debug, Clone, PartialEq)]` | 74.63% 🟡 |
