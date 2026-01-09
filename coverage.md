# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/error.rs | 3 | 4 | 7 | 42.86% 🟠 |
| src/lib.rs | 111 | 27 | 138 | 80.43% 🟢 |
| src/naga_wasm_backend/backend.rs | 44 | 1 | 45 | 97.78% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 30 | 14 | 44 | 68.18% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 20 | 0 | 20 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 43 | 23 | 66 | 65.15% 🟡 |
| src/naga_wasm_backend/output_layout.rs | 3 | 2 | 5 | 60.00% 🟡 |
| src/naga_wasm_backend/types.rs | 6 | 1 | 7 | 85.71% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 14 | 0 | 14 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 18 | 3 | 21 | 85.71% 🟢 |
| src/webgl2_context/drawing.rs | 30 | 6 | 36 | 83.33% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 6 | 1 | 7 | 85.71% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 84 | 15 | 99 | 84.85% 🟢 |
| src/webgl2_context/state.rs | 14 | 1 | 15 | 93.33% 🟢 |
| src/webgl2_context/textures.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 9 | 1 | 10 | 90.00% 🟢 |
| src/webgl2_context/vaos.rs | 37 | 0 | 37 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 24 | 26 | 50 | 48.00% 🟠 |
| **Total** | **527** | **125** | **652** | **80.83% 🟢** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 27/138 | [516] `pub extern "C" fn wasm_ctx_uniform1f(ctx: u32, location: ...` | 80.43% 🟢 |
| src/webgpu/backend.rs | 26/50 | [919] `}` | 48.00% 🟠 |
| src/naga_wasm_backend/expressions.rs | 23/66 | [941] `let var = &ctx.module.global_variables[*handle];` | 65.15% 🟡 |
| src/webgl2_context/shaders.rs | 15/99 | [596] `if !varying_locations.values().any(|&v| v == *loc) {` | 84.85% 🟢 |
