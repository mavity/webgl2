# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 4 | 1 | 5 | 80.00% 🟢 |
| src/decompiler/ast.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/decompiler/emitter.rs | 17 | 4 | 21 | 80.95% 🟢 |
| src/decompiler/lifter.rs | 7 | 1 | 8 | 87.50% 🟢 |
| src/decompiler/mod.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/decompiler/module.rs | 0 | 2 | 2 | 0.00% 🟡 |
| src/decompiler/parser.rs | 24 | 2 | 26 | 92.31% 🟢 |
| src/decompiler/simplifier.rs | 34 | 0 | 34 | 100.00% 🟢 |
| src/error.rs | 31 | 4 | 35 | 88.57% 🟢 |
| src/lib.rs | 84 | 325 | 409 | 20.54% 🟠 |
| src/naga_wasm_backend/backend.rs | 147 | 42 | 189 | 77.78% 🟡 |
| src/naga_wasm_backend/call_lowering.rs | 23 | 3 | 26 | 88.46% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 37 | 37 | 74 | 50.00% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 120 | 151 | 271 | 44.28% 🟠 |
| src/naga_wasm_backend/function_abi.rs | 28 | 11 | 39 | 71.79% 🟡 |
| src/naga_wasm_backend/functions/prep.rs | 6 | 3 | 9 | 66.67% 🟡 |
| src/naga_wasm_backend/functions/registry.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 5 | 1 | 6 | 83.33% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 10 | 1 | 11 | 90.91% 🟢 |
| src/naga_wasm_backend/types.rs | 19 | 3 | 22 | 86.36% 🟢 |
| src/wasm_gl_emu/device.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/wasm_gl_emu/transfer.rs | 10 | 13 | 23 | 43.48% 🟠 |
| src/webgl2_context/buffers.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/webgl2_context/drawing.rs | 32 | 1 | 33 | 96.97% 🟢 |
| src/webgl2_context/renderbuffers.rs | 5 | 6 | 11 | 45.45% 🟠 |
| src/webgl2_context/shaders.rs | 180 | 33 | 213 | 84.51% 🟢 |
| src/webgl2_context/state.rs | 13 | 2 | 15 | 86.67% 🟢 |
| src/webgl2_context/textures.rs | 47 | 8 | 55 | 85.45% 🟢 |
| src/webgl2_context/types.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/webgpu/backend.rs | 115 | 33 | 148 | 77.70% 🟡 |
| src/webgpu/command.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgpu/pipeline.rs | 2 | 1 | 3 | 66.67% 🟡 |
| **Total** | **1049** | **688** | **1737** | **60.39% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 325/409 | [1994] `) -> u32 {` | 20.54% 🟠 |
| src/naga_wasm_backend/expressions.rs | 151/271 | [1500] `ctx.wasm_func.instruction(&Instruction::LocalTee(temp_a));` | 44.28% 🟠 |
| src/naga_wasm_backend/backend.rs | 42/189 | [325] `func.instruction(&Instruction::LocalGet(l_bpp));` | 77.78% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 37/74 | [464] `for arg in arguments {` | 50.00% 🟡 |
