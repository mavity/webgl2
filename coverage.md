# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/decompiler/ast.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/decompiler/emitter.rs | 10 | 4 | 14 | 71.43% 🟡 |
| src/decompiler/lifter.rs | 12 | 10 | 22 | 54.55% 🟡 |
| src/decompiler/mod.rs | 0 | 2 | 2 | 0.00% 🟡 |
| src/decompiler/simplifier.rs | 35 | 12 | 47 | 74.47% 🟡 |
| src/error.rs | 17 | 15 | 32 | 53.13% 🟡 |
| src/lib.rs | 86 | 313 | 399 | 21.55% 🟠 |
| src/naga_wasm_backend/backend.rs | 143 | 38 | 181 | 79.01% 🟡 |
| src/naga_wasm_backend/call_lowering.rs | 19 | 5 | 24 | 79.17% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 49 | 43 | 92 | 53.26% 🟡 |
| src/naga_wasm_backend/expressions.rs | 163 | 122 | 285 | 57.19% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 19 | 16 | 35 | 54.29% 🟡 |
| src/naga_wasm_backend/functions/prep.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 4 | 1 | 5 | 80.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 4 | 2 | 6 | 66.67% 🟡 |
| src/naga_wasm_backend/types.rs | 24 | 10 | 34 | 70.59% 🟡 |
| src/wasm_gl_emu/device.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 4 | 1 | 5 | 80.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 40 | 27 | 67 | 59.70% 🟡 |
| src/wasm_gl_emu/transfer.rs | 35 | 3 | 38 | 92.11% 🟢 |
| src/webgl2_context/blend.rs | 8 | 1 | 9 | 88.89% 🟢 |
| src/webgl2_context/buffers.rs | 20 | 2 | 22 | 90.91% 🟢 |
| src/webgl2_context/drawing.rs | 33 | 3 | 36 | 91.67% 🟢 |
| src/webgl2_context/framebuffers.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 12 | 1 | 13 | 92.31% 🟢 |
| src/webgl2_context/shaders.rs | 140 | 14 | 154 | 90.91% 🟢 |
| src/webgl2_context/state.rs | 26 | 10 | 36 | 72.22% 🟡 |
| src/webgl2_context/textures.rs | 8 | 24 | 32 | 25.00% 🟠 |
| src/webgl2_context/types.rs | 9 | 1 | 10 | 90.00% 🟢 |
| src/webgpu/adapter.rs | 3 | 5 | 8 | 37.50% 🟠 |
| src/webgpu/backend.rs | 98 | 12 | 110 | 89.09% 🟢 |
| src/webgpu/command.rs | 2 | 0 | 2 | 100.00% 🟢 |
| **Total** | **1044** | **697** | **1741** | **59.97% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 313/399 | [1065] `#[no_mangle]` | 21.55% 🟠 |
| src/naga_wasm_backend/expressions.rs | 122/285 | [68] `for comp_handle in components {` | 57.19% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 43/92 | [159] `match stmt {` | 53.26% 🟡 |
| src/naga_wasm_backend/backend.rs | 38/181 | [142] `.iter()` | 79.01% 🟡 |
