# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 12 | 5 | 17 | 70.59% 🟡 |
| src/decompiler/ast.rs | 3 | 2 | 5 | 60.00% 🟡 |
| src/decompiler/emitter.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/decompiler/lifter.rs | 10 | 1 | 11 | 90.91% 🟢 |
| src/decompiler/module.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/decompiler/parser.rs | 15 | 5 | 20 | 75.00% 🟡 |
| src/decompiler/simplifier.rs | 35 | 6 | 41 | 85.37% 🟢 |
| src/error.rs | 34 | 7 | 41 | 82.93% 🟢 |
| src/lib.rs | 183 | 310 | 493 | 37.12% 🟠 |
| src/naga_wasm_backend/backend.rs | 161 | 29 | 190 | 84.74% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 10 | 5 | 15 | 66.67% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 37 | 30 | 67 | 55.22% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 11 | 15 | 26 | 42.31% 🟠 |
| src/naga_wasm_backend/expressions.rs | 255 | 152 | 407 | 62.65% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 14 | 5 | 19 | 73.68% 🟡 |
| src/naga_wasm_backend/functions/prep.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 12 | 0 | 12 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 9 | 1 | 10 | 90.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 7 | 2 | 9 | 77.78% 🟡 |
| src/naga_wasm_backend/types.rs | 14 | 6 | 20 | 70.00% 🟡 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 38 | 26 | 64 | 59.38% 🟡 |
| src/wasm_gl_emu/transfer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/webgl2_context/blend.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 15 | 1 | 16 | 93.75% 🟢 |
| src/webgl2_context/drawing.rs | 31 | 1 | 32 | 96.88% 🟢 |
| src/webgl2_context/registry.rs | 13 | 1 | 14 | 92.86% 🟢 |
| src/webgl2_context/shaders.rs | 147 | 21 | 168 | 87.50% 🟢 |
| src/webgl2_context/state.rs | 15 | 16 | 31 | 48.39% 🟠 |
| src/webgl2_context/textures.rs | 52 | 0 | 52 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 24 | 9 | 33 | 72.73% 🟡 |
| src/webgpu/adapter.rs | 13 | 6 | 19 | 68.42% 🟡 |
| src/webgpu/backend.rs | 72 | 9 | 81 | 88.89% 🟢 |
| src/webgpu/buffer.rs | 0 | 3 | 3 | 0.00% 🟡 |
| src/webgpu/command.rs | 23 | 4 | 27 | 85.19% 🟢 |
| src/webgpu/pipeline.rs | 1 | 0 | 1 | 100.00% 🟢 |
| **Total** | **1290** | **678** | **1968** | **65.55% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 310/493 | [1996] `}` | 37.12% 🟠 |
| src/naga_wasm_backend/expressions.rs | 152/407 | [1845] `translate_expression_component(*arg, j, ctx)?;` | 62.65% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 30/67 | [488] `if let Some(ret) = &called_func.result {` | 55.22% 🟡 |
| src/naga_wasm_backend/backend.rs | 29/190 | [469] `func.instruction(&Instruction::F32Div);` | 84.74% 🟢 |
