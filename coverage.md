# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---|
| src/lib.rs | 85 | 36 | 121 | 70.25% 🟡 |
| src/naga_wasm_backend/backend.rs | 39 | 0 | 39 | 100.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 15 | 4 | 19 | 78.95% 🟡 |
| src/naga_wasm_backend/expressions.rs | 27 | 12 | 39 | 69.23% 🟡 |
| src/naga_wasm_backend/types.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 15 | 0 | 15 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 15 | 1 | 16 | 93.75% 🟢 |
| src/webgl2_context/drawing.rs | 24 | 4 | 28 | 85.71% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 60 | 0 | 60 | 100.00% 🟢 |
| src/webgl2_context/state.rs | 10 | 3 | 13 | 76.92% 🟡 |
| src/webgl2_context/textures.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 11 | 1 | 12 | 91.67% 🟢 |
| src/webgl2_context/vaos.rs | 24 | 0 | 24 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 4 | 0 | 4 | 100.00% 🟢 |
| **Total** | **367** | **61** | **428** | **85.75% 🟢** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---|
| src/lib.rs | 36/121 | [516] `#[no_mangle]` | 70.25% 🟡 |
| src/naga_wasm_backend/expressions.rs | 12/39 | [49] `if component_idx == 0 {` | 69.23% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 4/19 | [80] `for _ in 0..types.len() {` | 78.95% 🟡 |
| src/webgl2_context/drawing.rs | 4/28 | [353] `dest_slice[dst_off + 2] = 0;` | 85.71% 🟢 |
