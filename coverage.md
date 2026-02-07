# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 9 | 5 | 14 | 64.29% 🟡 |
| src/decompiler/ast.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/decompiler/emitter.rs | 24 | 3 | 27 | 88.89% 🟢 |
| src/decompiler/lifter.rs | 6 | 2 | 8 | 75.00% 🟡 |
| src/decompiler/parser.rs | 12 | 3 | 15 | 80.00% 🟢 |
| src/decompiler/simplifier.rs | 36 | 2 | 38 | 94.74% 🟢 |
| src/error.rs | 33 | 8 | 41 | 80.49% 🟢 |
| src/lib.rs | 155 | 298 | 453 | 34.22% 🟠 |
| src/naga_wasm_backend/backend.rs | 151 | 24 | 175 | 86.29% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 9 | 3 | 12 | 75.00% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 38 | 30 | 68 | 55.88% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 11 | 16 | 27 | 40.74% 🟠 |
| src/naga_wasm_backend/expressions.rs | 270 | 155 | 425 | 63.53% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 15 | 8 | 23 | 65.22% 🟡 |
| src/naga_wasm_backend/functions/prep.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 15 | 1 | 16 | 93.75% 🟢 |
| src/naga_wasm_backend/mod.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 7 | 1 | 8 | 87.50% 🟢 |
| src/naga_wasm_backend/types.rs | 22 | 6 | 28 | 78.57% 🟡 |
| src/wasm_gl_emu/device.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 16 | 41 | 57 | 28.07% 🟠 |
| src/wasm_gl_emu/transfer.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgl2_context/blend.rs | 14 | 1 | 15 | 93.33% 🟢 |
| src/webgl2_context/buffers.rs | 12 | 0 | 12 | 100.00% 🟢 |
| src/webgl2_context/drawing.rs | 33 | 0 | 33 | 100.00% 🟢 |
| src/webgl2_context/ephemeral.rs | 0 | 1 | 1 | 0.00% 🟡 |
| src/webgl2_context/framebuffers.rs | 8 | 6 | 14 | 57.14% 🟡 |
| src/webgl2_context/registry.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 137 | 29 | 166 | 82.53% 🟢 |
| src/webgl2_context/state.rs | 33 | 9 | 42 | 78.57% 🟡 |
| src/webgl2_context/textures.rs | 60 | 0 | 60 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 45 | 21 | 66 | 68.18% 🟡 |
| src/webgpu/adapter.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgpu/backend.rs | 52 | 17 | 69 | 75.36% 🟡 |
| src/webgpu/bind_group.rs | 6 | 1 | 7 | 85.71% 🟢 |
| src/webgpu/buffer.rs | 8 | 2 | 10 | 80.00% 🟢 |
| src/webgpu/command.rs | 12 | 5 | 17 | 70.59% 🟡 |
| **Total** | **1293** | **698** | **1991** | **64.94% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 298/453 | [1994] `vertex_module_handle: u32,` | 34.22% 🟠 |
| src/naga_wasm_backend/expressions.rs | 155/425 | [910] `if let Some(handle) = found_global {` | 63.53% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 41/57 | [370] `match eq {` | 28.07% 🟠 |
| src/naga_wasm_backend/control_flow.rs | 30/68 | [489] `let types = super::types::naga_to_wasm_types(` | 55.88% 🟡 |
