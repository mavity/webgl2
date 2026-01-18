# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/ast.rs | 0 | 1 | 1 | 0.00% 🟡 |
| src/decompiler/emitter.rs | 17 | 8 | 25 | 68.00% 🟡 |
| src/decompiler/lifter.rs | 17 | 21 | 38 | 44.74% 🟠 |
| src/decompiler/mod.rs | 8 | 6 | 14 | 57.14% 🟡 |
| src/decompiler/module.rs | 3 | 1 | 4 | 75.00% 🟡 |
| src/decompiler/parser.rs | 15 | 2 | 17 | 88.24% 🟢 |
| src/decompiler/simplifier.rs | 54 | 19 | 73 | 73.97% 🟡 |
| src/error.rs | 8 | 15 | 23 | 34.78% 🟠 |
| src/lib.rs | 314 | 224 | 538 | 58.36% 🟡 |
| src/naga_wasm_backend/backend.rs | 62 | 2 | 64 | 96.88% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 19 | 0 | 19 | 100.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 30 | 40 | 70 | 42.86% 🟠 |
| src/naga_wasm_backend/debug/stub.rs | 20 | 0 | 20 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 65 | 109 | 174 | 37.36% 🟠 |
| src/naga_wasm_backend/function_abi.rs | 25 | 0 | 25 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 4 | 2 | 6 | 66.67% 🟡 |
| src/naga_wasm_backend/types.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/wasm_gl_emu/device.rs | 20 | 4 | 24 | 83.33% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 3 | 1 | 4 | 75.00% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 32 | 18 | 50 | 64.00% 🟡 |
| src/wasm_gl_emu/transfer.rs | 19 | 7 | 26 | 73.08% 🟡 |
| src/webgl2_context/blend.rs | 0 | 3 | 3 | 0.00% 🟡 |
| src/webgl2_context/buffers.rs | 20 | 4 | 24 | 83.33% 🟢 |
| src/webgl2_context/drawing.rs | 13 | 0 | 13 | 100.00% 🟢 |
| src/webgl2_context/framebuffers.rs | 11 | 1 | 12 | 91.67% 🟢 |
| src/webgl2_context/registry.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 112 | 14 | 126 | 88.89% 🟢 |
| src/webgl2_context/state.rs | 16 | 4 | 20 | 80.00% 🟢 |
| src/webgl2_context/textures.rs | 25 | 9 | 34 | 73.53% 🟡 |
| src/webgl2_context/types.rs | 16 | 0 | 16 | 100.00% 🟢 |
| src/webgl2_context/vaos.rs | 35 | 0 | 35 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 70 | 26 | 96 | 72.92% 🟡 |
| src/webgpu/command.rs | 3 | 1 | 4 | 75.00% 🟡 |
| **Total** | **1104** | **542** | **1646** | **67.07% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 224/538 | [1033] `/// Use a program.` | 58.36% 🟡 |
| src/naga_wasm_backend/expressions.rs | 109/174 | [1504] `for j in 0..count {` | 37.36% 🟠 |
| src/naga_wasm_backend/control_flow.rs | 40/70 | [218] `for (s, s_span) in body.span_iter() {` | 42.86% 🟠 |
| src/webgpu/backend.rs | 26/96 | [1404] `}` | 72.92% 🟡 |
