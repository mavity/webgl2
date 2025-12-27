# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---|
| src/lib.rs | 101 | 34 | 135 | 74.81% 🟡 |
| src/naga_wasm_backend/backend.rs | 39 | 0 | 39 | 100.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 15 | 5 | 20 | 75.00% 🟡 |
| src/naga_wasm_backend/expressions.rs | 29 | 12 | 41 | 70.73% 🟡 |
| src/naga_wasm_backend/types.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 15 | 0 | 15 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 16 | 2 | 18 | 88.89% 🟢 |
| src/webgl2_context/drawing.rs | 23 | 4 | 27 | 85.19% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 67 | 4 | 71 | 94.37% 🟢 |
| src/webgl2_context/state.rs | 11 | 3 | 14 | 78.57% 🟡 |
| src/webgl2_context/textures.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 11 | 1 | 12 | 91.67% 🟢 |
| src/webgl2_context/vaos.rs | 23 | 0 | 23 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 25 | 25 | 50 | 50.00% 🟡 |
| **Total** | **416** | **90** | **506** | **82.21% 🟢** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---|
| src/lib.rs | 34/135 | [516] `/// Set uniform 3f.` | 74.81% 🟡 |
| src/webgpu/backend.rs | 25/50 | [806] `}` | 50.00% 🟡 |
| src/naga_wasm_backend/expressions.rs | 12/41 | [49] `if component_idx == 0 {` | 70.73% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 5/20 | [80] `for _ in 0..types.len() {` | 75.00% 🟡 |
