# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 14 | 5 | 19 | 73.68% 🟡 |
| src/decompiler/ast.rs | 2 | 4 | 6 | 33.33% 🟡 |
| src/decompiler/emitter.rs | 27 | 4 | 31 | 87.10% 🟢 |
| src/decompiler/lifter.rs | 6 | 2 | 8 | 75.00% 🟡 |
| src/decompiler/mod.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/decompiler/parser.rs | 3 | 2 | 5 | 60.00% 🟡 |
| src/decompiler/simplifier.rs | 35 | 2 | 37 | 94.59% 🟢 |
| src/error.rs | 31 | 6 | 37 | 83.78% 🟢 |
| src/lib.rs | 151 | 331 | 482 | 31.33% 🟠 |
| src/naga_wasm_backend/backend.rs | 153 | 30 | 183 | 83.61% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 9 | 3 | 12 | 75.00% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 41 | 27 | 68 | 60.29% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 11 | 13 | 24 | 45.83% 🟠 |
| src/naga_wasm_backend/expressions.rs | 270 | 147 | 417 | 64.75% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 19 | 5 | 24 | 79.17% 🟡 |
| src/naga_wasm_backend/functions/prep.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 12 | 0 | 12 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/naga_wasm_backend/types.rs | 23 | 6 | 29 | 79.31% 🟡 |
| src/wasm_gl_emu/device.rs | 4 | 1 | 5 | 80.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 19 | 42 | 61 | 31.15% 🟠 |
| src/wasm_gl_emu/transfer.rs | 0 | 3 | 3 | 0.00% 🟡 |
| src/webgl2_context/buffers.rs | 14 | 0 | 14 | 100.00% 🟢 |
| src/webgl2_context/drawing.rs | 29 | 0 | 29 | 100.00% 🟢 |
| src/webgl2_context/ephemeral.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 139 | 21 | 160 | 86.88% 🟢 |
| src/webgl2_context/state.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgl2_context/textures.rs | 59 | 0 | 59 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 43 | 18 | 61 | 70.49% 🟡 |
| src/webgpu/adapter.rs | 6 | 1 | 7 | 85.71% 🟢 |
| src/webgpu/backend.rs | 63 | 19 | 82 | 76.83% 🟡 |
| src/webgpu/bind_group.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/buffer.rs | 1 | 2 | 3 | 33.33% 🟡 |
| src/webgpu/command.rs | 5 | 6 | 11 | 45.45% 🟠 |
| src/webgpu/shader.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/webgpu/texture.rs | 8 | 3 | 11 | 72.73% 🟡 |
| **Total** | **1240** | **703** | **1943** | **63.82% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 331/482 | [1994] `depth_write_enabled: u32,` | 31.33% 🟠 |
| src/naga_wasm_backend/expressions.rs | 147/417 | [1135] `if let Some(handle) = found_global {` | 64.75% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 42/61 | [370] `match eq {` | 31.15% 🟠 |
| src/naga_wasm_backend/backend.rs | 30/183 | [497] `func.instruction(&Instruction::End);` | 83.61% 🟢 |
