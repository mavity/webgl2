# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---|
| src/lib.rs | 78 | 17 | 95 | 82.11% 🟢 |
| src/naga_wasm_backend/backend.rs | 39 | 0 | 39 | 100.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 15 | 4 | 19 | 78.95% 🟡 |
| src/naga_wasm_backend/expressions.rs | 27 | 12 | 39 | 69.23% 🟡 |
| src/naga_wasm_backend/types.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 16 | 1 | 17 | 94.12% 🟢 |
| src/webgl2_context/drawing.rs | 25 | 5 | 30 | 83.33% 🟢 |
| src/webgl2_context/framebuffers.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 3 | 2 | 5 | 60.00% 🟡 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 61 | 0 | 61 | 100.00% 🟢 |
| src/webgl2_context/state.rs | 11 | 3 | 14 | 78.57% 🟡 |
| src/webgl2_context/textures.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 11 | 1 | 12 | 91.67% 🟢 |
| src/webgl2_context/vaos.rs | 16 | 0 | 16 | 100.00% 🟢 |
| **Total** | **336** | **45** | **381** | **88.19% 🟢** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---|
| src/lib.rs | 17/95 | [56] `pub fn js_log(level: u32, s: &str) {` | 82.11% 🟢 |
| src/naga_wasm_backend/expressions.rs | 12/39 | [49] `if component_idx == 0 {` | 69.23% 🟡 |
| src/webgl2_context/drawing.rs | 5/30 | [561] `for k in 0..64 {` | 83.33% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 4/19 | [80] `for _ in 0..types.len() {` | 78.95% 🟡 |
