# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 4 | 1 | 5 | 80.00% 🟢 |
| src/decompiler/ast.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/decompiler/emitter.rs | 25 | 5 | 30 | 83.33% 🟢 |
| src/decompiler/lifter.rs | 7 | 1 | 8 | 87.50% 🟢 |
| src/decompiler/mod.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/decompiler/module.rs | 1 | 2 | 3 | 33.33% 🟡 |
| src/decompiler/parser.rs | 24 | 2 | 26 | 92.31% 🟢 |
| src/decompiler/simplifier.rs | 32 | 0 | 32 | 100.00% 🟢 |
| src/error.rs | 34 | 2 | 36 | 94.44% 🟢 |
| src/lib.rs | 83 | 317 | 400 | 20.75% 🟠 |
| src/naga_wasm_backend/backend.rs | 147 | 42 | 189 | 77.78% 🟡 |
| src/naga_wasm_backend/call_lowering.rs | 23 | 3 | 26 | 88.46% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 37 | 37 | 74 | 50.00% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 130 | 151 | 281 | 46.26% 🟠 |
| src/naga_wasm_backend/function_abi.rs | 22 | 19 | 41 | 53.66% 🟡 |
| src/naga_wasm_backend/functions/prep.rs | 7 | 2 | 9 | 77.78% 🟡 |
| src/naga_wasm_backend/functions/registry.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 6 | 1 | 7 | 85.71% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 10 | 1 | 11 | 90.91% 🟢 |
| src/naga_wasm_backend/types.rs | 19 | 3 | 22 | 86.36% 🟢 |
| src/wasm_gl_emu/device.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/wasm_gl_emu/framebuffer.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/wasm_gl_emu/transfer.rs | 11 | 10 | 21 | 52.38% 🟡 |
| src/webgl2_context/buffers.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/webgl2_context/drawing.rs | 32 | 1 | 33 | 96.97% 🟢 |
| src/webgl2_context/renderbuffers.rs | 5 | 6 | 11 | 45.45% 🟠 |
| src/webgl2_context/shaders.rs | 179 | 33 | 212 | 84.43% 🟢 |
| src/webgl2_context/state.rs | 13 | 2 | 15 | 86.67% 🟢 |
| src/webgl2_context/textures.rs | 47 | 8 | 55 | 85.45% 🟢 |
| src/webgl2_context/types.rs | 11 | 3 | 14 | 78.57% 🟡 |
| src/webgpu/adapter.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgpu/backend.rs | 124 | 25 | 149 | 83.22% 🟢 |
| src/webgpu/command.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/webgpu/texture.rs | 1 | 0 | 1 | 100.00% 🟢 |
| **Total** | **1076** | **679** | **1755** | **61.31% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 317/400 | [1994] `///` | 20.75% 🟠 |
| src/naga_wasm_backend/expressions.rs | 151/281 | [1426] `}` | 46.26% 🟠 |
| src/naga_wasm_backend/backend.rs | 42/189 | [325] `func.instruction(&Instruction::LocalGet(l_bpp));` | 77.78% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 37/74 | [464] `for arg in arguments {` | 50.00% 🟡 |
