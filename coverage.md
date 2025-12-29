# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---|
| src/lib.rs | 110 | 31 | 141 | 78.01% 🟡 |
| src/naga_wasm_backend/backend.rs | 45 | 1 | 46 | 97.83% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 24 | 11 | 35 | 68.57% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 20 | 0 | 20 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 32 | 13 | 45 | 71.11% 🟡 |
| src/naga_wasm_backend/types.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 14 | 0 | 14 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 17 | 3 | 20 | 85.00% 🟢 |
| src/webgl2_context/drawing.rs | 29 | 4 | 33 | 87.88% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 70 | 3 | 73 | 95.89% 🟢 |
| src/webgl2_context/state.rs | 14 | 3 | 17 | 82.35% 🟢 |
| src/webgl2_context/textures.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 9 | 1 | 10 | 90.00% 🟢 |
| src/webgl2_context/vaos.rs | 36 | 0 | 36 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 24 | 26 | 50 | 48.00% 🟠 |
| **Total** | **486** | **96** | **582** | **83.51% 🟢** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---|
| src/lib.rs | 31/141 | [516] `/// Set uniform 3f.` | 78.01% 🟡 |
| src/webgpu/backend.rs | 26/50 | [919] `}` | 48.00% 🟠 |
| src/naga_wasm_backend/expressions.rs | 13/45 | [11] `pub fn is_integer_type(type_inner: &TypeInner) -> bool {` | 71.11% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 11/35 | [171] `for i in (0..types.len()).rev() {` | 68.57% 🟡 |
