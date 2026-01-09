# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/error.rs | 3 | 4 | 7 | 42.86% 🟠 |
| src/lib.rs | 112 | 29 | 141 | 79.43% 🟡 |
| src/naga_wasm_backend/backend.rs | 44 | 1 | 45 | 97.78% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 30 | 14 | 44 | 68.18% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 20 | 0 | 20 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 43 | 19 | 62 | 69.35% 🟡 |
| src/naga_wasm_backend/output_layout.rs | 2 | 2 | 4 | 50.00% 🟡 |
| src/naga_wasm_backend/types.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 20 | 2 | 22 | 90.91% 🟢 |
| src/webgl2_context/buffers.rs | 16 | 1 | 17 | 94.12% 🟢 |
| src/webgl2_context/drawing.rs | 34 | 6 | 40 | 85.00% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 6 | 1 | 7 | 85.71% 🟢 |
| src/webgl2_context/renderbuffers.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 83 | 15 | 98 | 84.69% 🟢 |
| src/webgl2_context/state.rs | 14 | 2 | 16 | 87.50% 🟢 |
| src/webgl2_context/textures.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 10 | 1 | 11 | 90.91% 🟢 |
| src/webgl2_context/vaos.rs | 36 | 0 | 36 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgpu/backend.rs | 24 | 26 | 50 | 48.00% 🟠 |
| **Total** | **536** | **123** | **659** | **81.34% 🟢** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 29/141 | [516] `pub extern "C" fn wasm_ctx_uniform1f(ctx: u32, location: ...` | 79.43% 🟡 |
| src/webgpu/backend.rs | 26/50 | [919] `}` | 48.00% 🟠 |
| src/naga_wasm_backend/expressions.rs | 19/62 | [929] `let var = &ctx.module.global_variables[*handle];` | 69.35% 🟡 |
| src/webgl2_context/shaders.rs | 15/98 | [604] `if !varying_locations.values().any(|&v| v == *loc) {` | 84.69% 🟢 |
