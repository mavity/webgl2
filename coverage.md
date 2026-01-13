# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/emitter.rs | 15 | 7 | 22 | 68.18% 🟡 |
| src/decompiler/lifter.rs | 18 | 21 | 39 | 46.15% 🟠 |
| src/decompiler/mod.rs | 9 | 5 | 14 | 64.29% 🟡 |
| src/decompiler/module.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/decompiler/parser.rs | 17 | 0 | 17 | 100.00% 🟢 |
| src/decompiler/simplifier.rs | 51 | 16 | 67 | 76.12% 🟡 |
| src/error.rs | 8 | 15 | 23 | 34.78% 🟠 |
| src/lib.rs | 300 | 203 | 503 | 59.64% 🟡 |
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
| src/wasm_gl_emu/framebuffer.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 34 | 13 | 47 | 72.34% 🟡 |
| src/webgl2_context/blend.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/webgl2_context/buffers.rs | 16 | 1 | 17 | 94.12% 🟢 |
| src/webgl2_context/drawing.rs | 32 | 6 | 38 | 84.21% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 87 | 14 | 101 | 86.14% 🟢 |
| src/webgl2_context/state.rs | 21 | 1 | 22 | 95.45% 🟢 |
| src/webgl2_context/textures.rs | 21 | 9 | 30 | 70.00% 🟡 |
| src/webgl2_context/types.rs | 10 | 1 | 11 | 90.91% 🟢 |
| src/webgl2_context/vaos.rs | 36 | 0 | 36 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 25 | 24 | 49 | 51.02% 🟡 |
| **Total** | **953** | **372** | **1325** | **71.92% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 203/503 | [1043] `#[no_mangle]` | 59.64% 🟡 |
| src/webgpu/backend.rs | 24/49 | [943] `}` | 51.02% 🟡 |
| src/decompiler/lifter.rs | 21/39 | [455] `fn unary_op(&mut self, op: UnaryOp) {` | 46.15% 🟠 |
| src/decompiler/simplifier.rs | 16/67 | [112] `let get_const = |id: &Id| egraph[*id].data.constant;` | 76.12% 🟡 |
