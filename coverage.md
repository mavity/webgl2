# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---|
| src/lib.rs | 110 | 31 | 141 | 78.01% 🟡 |
| src/naga_wasm_backend/backend.rs | 45 | 1 | 46 | 97.83% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 24 | 11 | 35 | 68.57% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 20 | 0 | 20 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 31 | 12 | 43 | 72.09% 🟡 |
| src/naga_wasm_backend/types.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 15 | 0 | 15 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 15 | 2 | 17 | 88.24% 🟢 |
| src/webgl2_context/drawing.rs | 29 | 4 | 33 | 87.88% 🟢 |
| src/webgl2_context/framebuffers.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 70 | 3 | 73 | 95.89% 🟢 |
| src/webgl2_context/state.rs | 14 | 3 | 17 | 82.35% 🟢 |
| src/webgl2_context/textures.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 9 | 1 | 10 | 90.00% 🟢 |
| src/webgl2_context/vaos.rs | 36 | 0 | 36 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgpu/backend.rs | 41 | 20 | 61 | 67.21% 🟡 |
| **Total** | **501** | **88** | **589** | **85.06% 🟢** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---|
| src/lib.rs | 31/141 | [516] `/// Set uniform 3f.` | 78.01% 🟡 |
| src/webgpu/backend.rs | 20/61 | [919] `}` | 67.21% 🟡 |
| src/naga_wasm_backend/expressions.rs | 12/43 | [71] `if component_idx == 0 {` | 72.09% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 11/35 | [171] `for i in (0..types.len()).rev() {` | 68.57% 🟡 |
