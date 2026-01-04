# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/error.rs | 3 | 5 | 8 | 37.50% 🟠 |
| src/lib.rs | 112 | 29 | 141 | 79.43% 🟡 |
| src/naga_wasm_backend/backend.rs | 43 | 1 | 44 | 97.73% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 30 | 12 | 42 | 71.43% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 20 | 0 | 20 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 40 | 19 | 59 | 67.80% 🟡 |
| src/naga_wasm_backend/output_layout.rs | 1 | 1 | 2 | 50.00% 🟡 |
| src/naga_wasm_backend/types.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 20 | 2 | 22 | 90.91% 🟢 |
| src/webgl2_context/buffers.rs | 16 | 1 | 17 | 94.12% 🟢 |
| src/webgl2_context/drawing.rs | 34 | 6 | 40 | 85.00% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 6 | 1 | 7 | 85.71% 🟢 |
| src/webgl2_context/renderbuffers.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 80 | 13 | 93 | 86.02% 🟢 |
| src/webgl2_context/state.rs | 14 | 2 | 16 | 87.50% 🟢 |
| src/webgl2_context/textures.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 10 | 2 | 12 | 83.33% 🟢 |
| src/webgl2_context/vaos.rs | 36 | 0 | 36 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 24 | 26 | 50 | 48.00% 🟠 |
| **Total** | **527** | **120** | **647** | **81.45% 🟢** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 29/141 | [70] `pub fn js_log(level: u32, s: &str) {` | 79.43% 🟡 |
| src/webgpu/backend.rs | 26/50 | [919] `}` | 48.00% 🟠 |
| src/naga_wasm_backend/expressions.rs | 19/59 | [11] `pub fn is_integer_type(type_inner: &TypeInner) -> bool {` | 67.80% 🟡 |
| src/webgl2_context/shaders.rs | 13/93 | [575] `if !varying_locations.values().any(|&v| v == *loc) {` | 86.02% 🟢 |
