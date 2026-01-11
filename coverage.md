# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/error.rs | 4 | 17 | 21 | 19.05% 🔴 |
| src/lib.rs | 251 | 221 | 472 | 53.18% 🟡 |
| src/naga_wasm_backend/backend.rs | 50 | 2 | 52 | 96.15% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 5 | 9 | 14 | 35.71% 🟠 |
| src/naga_wasm_backend/control_flow.rs | 30 | 15 | 45 | 66.67% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 21 | 0 | 21 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 48 | 24 | 72 | 66.67% 🟡 |
| src/naga_wasm_backend/frame_allocator.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/naga_wasm_backend/function_abi.rs | 14 | 12 | 26 | 53.85% 🟡 |
| src/naga_wasm_backend/output_layout.rs | 2 | 3 | 5 | 40.00% 🟡 |
| src/naga_wasm_backend/types.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 14 | 0 | 14 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 16 | 1 | 17 | 94.12% 🟢 |
| src/webgl2_context/drawing.rs | 30 | 6 | 36 | 83.33% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 6 | 1 | 7 | 85.71% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 87 | 15 | 102 | 85.29% 🟢 |
| src/webgl2_context/state.rs | 14 | 1 | 15 | 93.33% 🟢 |
| src/webgl2_context/textures.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 9 | 1 | 10 | 90.00% 🟢 |
| src/webgl2_context/vaos.rs | 37 | 0 | 37 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 24 | 26 | 50 | 48.00% 🟠 |
| **Total** | **699** | **354** | **1053** | **66.38% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 221/472 | [1041] `) -> u32 {` | 53.18% 🟡 |
| src/webgpu/backend.rs | 26/50 | [919] `}` | 48.00% 🟠 |
| src/naga_wasm_backend/expressions.rs | 24/72 | [915] `let var = &ctx.module.global_variables[*handle];` | 66.67% 🟡 |
| src/error.rs | 17/21 | [52] `pub fn set_error(source: ErrorSource, code: u32, msg: imp...` | 19.05% 🔴 |
