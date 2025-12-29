# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---|
| src/lib.rs | 108 | 30 | 138 | 78.26% 🟡 |
| src/naga_wasm_backend/backend.rs | 43 | 2 | 45 | 95.56% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 25 | 11 | 36 | 69.44% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 20 | 0 | 20 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 41 | 19 | 60 | 68.33% 🟡 |
| src/naga_wasm_backend/types.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 14 | 0 | 14 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 15 | 1 | 16 | 93.75% 🟢 |
| src/webgl2_context/drawing.rs | 29 | 4 | 33 | 87.88% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 70 | 3 | 73 | 95.89% 🟢 |
| src/webgl2_context/state.rs | 13 | 3 | 16 | 81.25% 🟢 |
| src/webgl2_context/textures.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 10 | 1 | 11 | 90.91% 🟢 |
| src/webgl2_context/vaos.rs | 37 | 0 | 37 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgpu/backend.rs | 25 | 25 | 50 | 50.00% 🟡 |
| **Total** | **494** | **99** | **593** | **83.31% 🟢** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---|
| src/lib.rs | 30/138 | [516] `/// Set uniform 3f.` | 78.26% 🟡 |
| src/webgpu/backend.rs | 25/50 | [919] `}` | 50.00% 🟡 |
| src/naga_wasm_backend/expressions.rs | 19/60 | [11] `pub fn is_integer_type(type_inner: &TypeInner) -> bool {` | 68.33% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 11/36 | [181] `for i in (0..types.len()).rev() {` | 69.44% 🟡 |
