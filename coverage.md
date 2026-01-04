# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/error.rs | 3 | 4 | 7 | 42.86% 🟠 |
| src/lib.rs | 87 | 54 | 141 | 61.70% 🟡 |
| src/naga_wasm_backend/backend.rs | 41 | 3 | 44 | 93.18% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 22 | 21 | 43 | 51.16% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 20 | 0 | 20 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 30 | 29 | 59 | 50.85% 🟡 |
| src/naga_wasm_backend/output_layout.rs | 1 | 1 | 2 | 50.00% 🟡 |
| src/naga_wasm_backend/types.rs | 4 | 2 | 6 | 66.67% 🟡 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 14 | 8 | 22 | 63.64% 🟡 |
| src/webgl2_context/buffers.rs | 7 | 14 | 21 | 33.33% 🟠 |
| src/webgl2_context/drawing.rs | 29 | 11 | 40 | 72.50% 🟡 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 6 | 1 | 7 | 85.71% 🟢 |
| src/webgl2_context/renderbuffers.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 71 | 22 | 93 | 76.34% 🟡 |
| src/webgl2_context/state.rs | 10 | 6 | 16 | 62.50% 🟡 |
| src/webgl2_context/textures.rs | 8 | 2 | 10 | 80.00% 🟢 |
| src/webgl2_context/types.rs | 10 | 2 | 12 | 83.33% 🟢 |
| src/webgl2_context/vaos.rs | 15 | 21 | 36 | 41.67% 🟠 |
| src/webgpu/adapter.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgpu/backend.rs | 24 | 26 | 50 | 48.00% 🟠 |
| **Total** | **425** | **227** | **652** | **65.18% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 54/141 | [70] `pub fn js_log(level: u32, s: &str) {` | 61.70% 🟡 |
| src/naga_wasm_backend/expressions.rs | 29/59 | [11] `pub fn is_integer_type(type_inner: &TypeInner) -> bool {` | 50.85% 🟡 |
| src/webgpu/backend.rs | 26/50 | [919] `}` | 48.00% 🟠 |
| src/webgl2_context/shaders.rs | 22/93 | [575] `if !varying_locations.values().any(|&v| v == *loc) {` | 76.34% 🟡 |
