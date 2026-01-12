# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/error.rs | 3 | 18 | 21 | 14.29% 🔴 |
| src/lib.rs | 268 | 204 | 472 | 56.78% 🟡 |
| src/naga_wasm_backend/backend.rs | 54 | 2 | 56 | 96.43% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 5 | 4 | 9 | 55.56% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 29 | 15 | 44 | 65.91% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 21 | 0 | 21 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 47 | 23 | 70 | 67.14% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 14 | 12 | 26 | 53.85% 🟡 |
| src/naga_wasm_backend/output_layout.rs | 2 | 3 | 5 | 40.00% 🟡 |
| src/naga_wasm_backend/types.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 14 | 0 | 14 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 16 | 1 | 17 | 94.12% 🟢 |
| src/webgl2_context/drawing.rs | 30 | 6 | 36 | 83.33% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 6 | 1 | 7 | 85.71% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 86 | 16 | 102 | 84.31% 🟢 |
| src/webgl2_context/state.rs | 12 | 3 | 15 | 80.00% 🟢 |
| src/webgl2_context/textures.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 9 | 1 | 10 | 90.00% 🟢 |
| src/webgl2_context/vaos.rs | 36 | 1 | 37 | 97.30% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 24 | 26 | 50 | 48.00% 🟠 |
| **Total** | **712** | **336** | **1048** | **67.94% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 204/472 | [1041] `) -> u32 {` | 56.78% 🟡 |
| src/webgpu/backend.rs | 26/50 | [919] `}` | 48.00% 🟠 |
| src/naga_wasm_backend/expressions.rs | 23/70 | [886] `let var = &ctx.module.global_variables[*handle];` | 67.14% 🟡 |
| src/error.rs | 18/21 | [52] `pub fn set_error(source: ErrorSource, code: u32, msg: imp...` | 14.29% 🔴 |
