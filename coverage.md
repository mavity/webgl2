# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---|
| src/lib.rs | 102 | 34 | 136 | 75.00% 🟡 |
| src/naga_wasm_backend/backend.rs | 39 | 0 | 39 | 100.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 19 | 3 | 22 | 86.36% 🟢 |
| src/naga_wasm_backend/expressions.rs | 30 | 11 | 41 | 73.17% 🟡 |
| src/naga_wasm_backend/types.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 15 | 0 | 15 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 16 | 1 | 17 | 94.12% 🟢 |
| src/webgl2_context/drawing.rs | 28 | 4 | 32 | 87.50% 🟢 |
| src/webgl2_context/framebuffers.rs | 7 | 1 | 8 | 87.50% 🟢 |
| src/webgl2_context/registry.rs | 5 | 1 | 6 | 83.33% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 68 | 3 | 71 | 95.77% 🟢 |
| src/webgl2_context/state.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/webgl2_context/textures.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/webgl2_context/vaos.rs | 32 | 0 | 32 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgpu/backend.rs | 24 | 26 | 50 | 48.00% 🟠 |
| **Total** | **429** | **84** | **513** | **83.63% 🟢** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---|
| src/lib.rs | 34/136 | [516] `/// Set uniform 3f.` | 75.00% 🟡 |
| src/webgpu/backend.rs | 26/50 | [919] `}` | 48.00% 🟠 |
| src/naga_wasm_backend/expressions.rs | 11/41 | [52] `if component_idx == 0 {` | 73.17% 🟡 |
| src/webgl2_context/drawing.rs | 4/32 | [410] `dest_slice[dst_off + 2] = 0;` | 87.50% 🟢 |
