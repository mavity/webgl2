# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---|
| src/lib.rs | 103 | 34 | 137 | 75.18% 🟡 |
| src/naga_wasm_backend/backend.rs | 39 | 0 | 39 | 100.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 19 | 3 | 22 | 86.36% 🟢 |
| src/naga_wasm_backend/expressions.rs | 30 | 11 | 41 | 73.17% 🟡 |
| src/naga_wasm_backend/types.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 13 | 0 | 13 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 16 | 1 | 17 | 94.12% 🟢 |
| src/webgl2_context/drawing.rs | 26 | 4 | 30 | 86.67% 🟢 |
| src/webgl2_context/framebuffers.rs | 7 | 1 | 8 | 87.50% 🟢 |
| src/webgl2_context/registry.rs | 5 | 1 | 6 | 83.33% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 67 | 4 | 71 | 94.37% 🟢 |
| src/webgl2_context/state.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/webgl2_context/textures.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/webgl2_context/vaos.rs | 32 | 0 | 32 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgpu/backend.rs | 25 | 25 | 50 | 50.00% 🟡 |
| **Total** | **426** | **84** | **510** | **83.53% 🟢** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---|
| src/lib.rs | 34/137 | [516] `/// Set uniform 3f.` | 75.18% 🟡 |
| src/webgpu/backend.rs | 25/50 | [806] `}` | 50.00% 🟡 |
| src/naga_wasm_backend/expressions.rs | 11/41 | [52] `if component_idx == 0 {` | 73.17% 🟡 |
| src/webgl2_context/drawing.rs | 4/30 | [400] `dest_slice[dst_off + 2] = 0;` | 86.67% 🟢 |
