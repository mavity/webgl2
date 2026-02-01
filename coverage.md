# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/decompiler/ast.rs | 6 | 1 | 7 | 85.71% 🟢 |
| src/decompiler/emitter.rs | 3 | 1 | 4 | 75.00% 🟡 |
| src/decompiler/lifter.rs | 12 | 10 | 22 | 54.55% 🟡 |
| src/decompiler/mod.rs | 4 | 2 | 6 | 66.67% 🟡 |
| src/decompiler/parser.rs | 1 | 1 | 2 | 50.00% 🟡 |
| src/decompiler/simplifier.rs | 44 | 3 | 47 | 93.62% 🟢 |
| src/error.rs | 28 | 7 | 35 | 80.00% 🟢 |
| src/lib.rs | 129 | 311 | 440 | 29.32% 🟠 |
| src/naga_wasm_backend/backend.rs | 143 | 39 | 182 | 78.57% 🟡 |
| src/naga_wasm_backend/call_lowering.rs | 31 | 1 | 32 | 96.88% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 35 | 41 | 76 | 46.05% 🟠 |
| src/naga_wasm_backend/debug/stub.rs | 10 | 1 | 11 | 90.91% 🟢 |
| src/naga_wasm_backend/expressions.rs | 129 | 157 | 286 | 45.10% 🟠 |
| src/naga_wasm_backend/function_abi.rs | 32 | 1 | 33 | 96.97% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 6 | 1 | 7 | 85.71% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 3 | 2 | 5 | 60.00% 🟡 |
| src/naga_wasm_backend/types.rs | 24 | 0 | 24 | 100.00% 🟢 |
| src/wasm_gl_emu/device.rs | 4 | 1 | 5 | 80.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 5 | 2 | 7 | 71.43% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 26 | 20 | 46 | 56.52% 🟡 |
| src/wasm_gl_emu/transfer.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 21 | 5 | 26 | 80.77% 🟢 |
| src/webgl2_context/drawing.rs | 25 | 5 | 30 | 83.33% 🟢 |
| src/webgl2_context/shaders.rs | 153 | 14 | 167 | 91.62% 🟢 |
| src/webgl2_context/state.rs | 20 | 0 | 20 | 100.00% 🟢 |
| src/webgl2_context/textures.rs | 5 | 5 | 10 | 50.00% 🟡 |
| src/webgl2_context/types.rs | 51 | 2 | 53 | 96.23% 🟢 |
| src/webgpu/adapter.rs | 4 | 2 | 6 | 66.67% 🟡 |
| src/webgpu/backend.rs | 101 | 27 | 128 | 78.91% 🟡 |
| src/webgpu/buffer.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/webgpu/command.rs | 3 | 1 | 4 | 75.00% 🟡 |
| **Total** | **1076** | **663** | **1739** | **61.87% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 311/440 | [1999] `}` | 29.32% 🟠 |
| src/naga_wasm_backend/expressions.rs | 157/286 | [1821] `translate_expression(*argument, ctx)?;` | 45.10% 🟠 |
| src/naga_wasm_backend/control_flow.rs | 41/76 | [172] `for (s, s_span) in block.span_iter() {` | 46.05% 🟠 |
| src/naga_wasm_backend/backend.rs | 39/182 | [1101] `n == "color"` | 78.57% 🟡 |
