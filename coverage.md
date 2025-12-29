# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---|
| src/lib.rs | 110 | 31 | 141 | 78.01% 🟡 |
| src/naga_wasm_backend/backend.rs | 45 | 1 | 46 | 97.83% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 25 | 11 | 36 | 69.44% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 20 | 0 | 20 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 41 | 20 | 61 | 67.21% 🟡 |
| src/naga_wasm_backend/types.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 15 | 0 | 15 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 14 | 1 | 15 | 93.33% 🟢 |
| src/webgl2_context/drawing.rs | 30 | 4 | 34 | 88.24% 🟢 |
| src/webgl2_context/framebuffers.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 5 | 1 | 6 | 83.33% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 69 | 3 | 72 | 95.83% 🟢 |
| src/webgl2_context/state.rs | 14 | 3 | 17 | 82.35% 🟢 |
| src/webgl2_context/textures.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 13 | 1 | 14 | 92.86% 🟢 |
| src/webgl2_context/vaos.rs | 35 | 0 | 35 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 24 | 26 | 50 | 48.00% 🟠 |
| **Total** | **497** | **102** | **599** | **82.97% 🟢** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---|
| src/lib.rs | 31/141 | [516] `/// Set uniform 3f.` | 78.01% 🟡 |
| src/webgpu/backend.rs | 26/50 | [919] `}` | 48.00% 🟠 |
| src/naga_wasm_backend/expressions.rs | 20/61 | [11] `pub fn is_integer_type(type_inner: &TypeInner) -> bool {` | 67.21% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 11/36 | [179] `for i in (0..types.len()).rev() {` | 69.44% 🟡 |
