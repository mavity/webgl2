# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 11 | 5 | 16 | 68.75% 🟡 |
| src/decompiler/ast.rs | 3 | 5 | 8 | 37.50% 🟠 |
| src/decompiler/emitter.rs | 30 | 3 | 33 | 90.91% 🟢 |
| src/decompiler/lifter.rs | 6 | 2 | 8 | 75.00% 🟡 |
| src/decompiler/parser.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/decompiler/simplifier.rs | 35 | 2 | 37 | 94.59% 🟢 |
| src/error.rs | 32 | 5 | 37 | 86.49% 🟢 |
| src/lib.rs | 144 | 305 | 449 | 32.07% 🟠 |
| src/naga_wasm_backend/backend.rs | 154 | 28 | 182 | 84.62% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 9 | 3 | 12 | 75.00% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 38 | 29 | 67 | 56.72% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 11 | 15 | 26 | 42.31% 🟠 |
| src/naga_wasm_backend/expressions.rs | 255 | 149 | 404 | 63.12% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 19 | 5 | 24 | 79.17% 🟡 |
| src/naga_wasm_backend/functions/prep.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 15 | 1 | 16 | 93.75% 🟢 |
| src/naga_wasm_backend/mod.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/naga_wasm_backend/types.rs | 22 | 5 | 27 | 81.48% 🟢 |
| src/wasm_gl_emu/device.rs | 3 | 2 | 5 | 60.00% 🟡 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 17 | 41 | 58 | 29.31% 🟠 |
| src/wasm_gl_emu/transfer.rs | 0 | 2 | 2 | 0.00% 🟡 |
| src/webgl2_context/buffers.rs | 14 | 0 | 14 | 100.00% 🟢 |
| src/webgl2_context/drawing.rs | 30 | 0 | 30 | 100.00% 🟢 |
| src/webgl2_context/ephemeral.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 141 | 27 | 168 | 83.93% 🟢 |
| src/webgl2_context/state.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgl2_context/textures.rs | 59 | 0 | 59 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 46 | 17 | 63 | 73.02% 🟡 |
| src/webgpu/adapter.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/webgpu/backend.rs | 61 | 21 | 82 | 74.39% 🟡 |
| src/webgpu/bind_group.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgpu/buffer.rs | 1 | 2 | 3 | 33.33% 🟡 |
| src/webgpu/command.rs | 7 | 5 | 12 | 58.33% 🟡 |
| src/webgpu/shader.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/webgpu/texture.rs | 8 | 4 | 12 | 66.67% 🟡 |
| **Total** | **1221** | **684** | **1905** | **64.09% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 305/449 | [1994] `depth_write_enabled: u32,` | 32.07% 🟠 |
| src/naga_wasm_backend/expressions.rs | 149/404 | [910] `if let Some(handle) = found_global {` | 63.12% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 41/58 | [370] `match eq {` | 29.31% 🟠 |
| src/naga_wasm_backend/control_flow.rs | 29/67 | [489] `let types = super::types::naga_to_wasm_types(` | 56.72% 🟡 |
