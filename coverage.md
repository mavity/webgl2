# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---|
| src/lib.rs | 101 | 30 | 131 | 77.10% 🟡 |
| src/naga_wasm_backend/backend.rs | 39 | 0 | 39 | 100.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 15 | 4 | 19 | 78.95% 🟡 |
| src/naga_wasm_backend/expressions.rs | 29 | 12 | 41 | 70.73% 🟡 |
| src/naga_wasm_backend/types.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 15 | 0 | 15 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 16 | 1 | 17 | 94.12% 🟢 |
| src/webgl2_context/drawing.rs | 24 | 4 | 28 | 85.71% 🟢 |
| src/webgl2_context/framebuffers.rs | 7 | 1 | 8 | 87.50% 🟢 |
| src/webgl2_context/registry.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 67 | 5 | 72 | 93.06% 🟢 |
| src/webgl2_context/state.rs | 4 | 1 | 5 | 80.00% 🟢 |
| src/webgl2_context/textures.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 11 | 1 | 12 | 91.67% 🟢 |
| src/webgl2_context/vaos.rs | 23 | 0 | 23 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 23 | 20 | 43 | 53.49% 🟡 |
| **Total** | **406** | **79** | **485** | **83.71% 🟢** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---|
| src/lib.rs | 30/131 | [516] `#[no_mangle]` | 77.10% 🟡 |
| src/webgpu/backend.rs | 20/43 | [697] `}` | 53.49% 🟡 |
| src/naga_wasm_backend/expressions.rs | 12/41 | [49] `if component_idx == 0 {` | 70.73% 🟡 |
| src/webgl2_context/shaders.rs | 5/72 | [415] `if let Some(other_name) = used_locations.get(&loc) {` | 93.06% 🟢 |
