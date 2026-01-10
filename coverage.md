# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/error.rs | 3 | 18 | 21 | 14.29% 🔴 |
| src/lib.rs | 252 | 227 | 479 | 52.61% 🟡 |
| src/naga_wasm_backend/backend.rs | 44 | 1 | 45 | 97.78% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 30 | 14 | 44 | 68.18% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 21 | 0 | 21 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 43 | 23 | 66 | 65.15% 🟡 |
| src/naga_wasm_backend/output_layout.rs | 2 | 2 | 4 | 50.00% 🟡 |
| src/naga_wasm_backend/types.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 14 | 0 | 14 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 16 | 1 | 17 | 94.12% 🟢 |
| src/webgl2_context/drawing.rs | 31 | 6 | 37 | 83.78% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 6 | 2 | 8 | 75.00% 🟡 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 87 | 15 | 102 | 85.29% 🟢 |
| src/webgl2_context/state.rs | 14 | 1 | 15 | 93.33% 🟢 |
| src/webgl2_context/textures.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 9 | 1 | 10 | 90.00% 🟢 |
| src/webgl2_context/vaos.rs | 37 | 0 | 37 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 24 | 26 | 50 | 48.00% 🟠 |
| **Total** | **671** | **337** | **1008** | **66.57% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 227/479 | [1033] `};` | 52.61% 🟡 |
| src/webgpu/backend.rs | 26/50 | [919] `}` | 48.00% 🟠 |
| src/naga_wasm_backend/expressions.rs | 23/66 | [941] `let var = &ctx.module.global_variables[*handle];` | 65.15% 🟡 |
| src/error.rs | 18/21 | [52] `pub fn set_error(source: ErrorSource, code: u32, msg: imp...` | 14.29% 🔴 |
