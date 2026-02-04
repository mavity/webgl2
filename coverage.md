# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/decompiler/ast.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/decompiler/emitter.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/decompiler/lifter.rs | 4 | 2 | 6 | 66.67% 🟡 |
| src/decompiler/simplifier.rs | 44 | 0 | 44 | 100.00% 🟢 |
| src/error.rs | 31 | 6 | 37 | 83.78% 🟢 |
| src/lib.rs | 99 | 314 | 413 | 23.97% 🟠 |
| src/naga_wasm_backend/backend.rs | 148 | 47 | 195 | 75.90% 🟡 |
| src/naga_wasm_backend/call_lowering.rs | 26 | 3 | 29 | 89.66% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 38 | 44 | 82 | 46.34% 🟠 |
| src/naga_wasm_backend/debug/stub.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 158 | 153 | 311 | 50.80% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 24 | 11 | 35 | 68.57% 🟡 |
| src/naga_wasm_backend/functions/prep.rs | 7 | 2 | 9 | 77.78% 🟡 |
| src/naga_wasm_backend/functions/registry.rs | 4 | 1 | 5 | 80.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 5 | 3 | 8 | 62.50% 🟡 |
| src/naga_wasm_backend/output_layout.rs | 11 | 2 | 13 | 84.62% 🟢 |
| src/naga_wasm_backend/types.rs | 18 | 4 | 22 | 81.82% 🟢 |
| src/wasm_gl_emu/device.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 4 | 1 | 5 | 80.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 25 | 24 | 49 | 51.02% 🟡 |
| src/wasm_gl_emu/transfer.rs | 11 | 12 | 23 | 47.83% 🟠 |
| src/webgl2_context/buffers.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/webgl2_context/drawing.rs | 36 | 1 | 37 | 97.30% 🟢 |
| src/webgl2_context/renderbuffers.rs | 12 | 1 | 13 | 92.31% 🟢 |
| src/webgl2_context/shaders.rs | 176 | 52 | 228 | 77.19% 🟡 |
| src/webgl2_context/state.rs | 11 | 3 | 14 | 78.57% 🟡 |
| src/webgl2_context/textures.rs | 10 | 3 | 13 | 76.92% 🟡 |
| src/webgl2_context/types.rs | 5 | 1 | 6 | 83.33% 🟢 |
| src/webgpu/adapter.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgpu/backend.rs | 112 | 25 | 137 | 81.75% 🟢 |
| src/webgpu/buffer.rs | 1 | 3 | 4 | 25.00% 🟡 |
| src/webgpu/command.rs | 5 | 6 | 11 | 45.45% 🟠 |
| src/webgpu/pipeline.rs | 0 | 1 | 1 | 0.00% 🟡 |
| **Total** | **1059** | **726** | **1785** | **59.33% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 314/413 | [1994] `}` | 23.97% 🟠 |
| src/naga_wasm_backend/expressions.rs | 153/311 | [1555] `ctx.wasm_func.instruction(&Instruction::LocalSet(temp_b));` | 50.80% 🟡 |
| src/webgl2_context/shaders.rs | 52/228 | [428] `used_locations.insert(loc, (name.clone(), origin));` | 77.19% 🟡 |
| src/naga_wasm_backend/backend.rs | 47/195 | [110] `code: CodeSection::new(),` | 75.90% 🟡 |
