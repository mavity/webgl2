# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 5 | 3 | 8 | 62.50% 🟡 |
| src/decompiler/ast.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/decompiler/emitter.rs | 28 | 6 | 34 | 82.35% 🟢 |
| src/decompiler/lifter.rs | 6 | 3 | 9 | 66.67% 🟡 |
| src/decompiler/mod.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/decompiler/module.rs | 0 | 1 | 1 | 0.00% 🟡 |
| src/decompiler/parser.rs | 21 | 12 | 33 | 63.64% 🟡 |
| src/decompiler/simplifier.rs | 42 | 2 | 44 | 95.45% 🟢 |
| src/error.rs | 41 | 3 | 44 | 93.18% 🟢 |
| src/lib.rs | 94 | 313 | 407 | 23.10% 🟠 |
| src/naga_wasm_backend/backend.rs | 151 | 44 | 195 | 77.44% 🟡 |
| src/naga_wasm_backend/call_lowering.rs | 30 | 3 | 33 | 90.91% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 42 | 41 | 83 | 50.60% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 212 | 134 | 346 | 61.27% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 25 | 3 | 28 | 89.29% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 6 | 2 | 8 | 75.00% 🟡 |
| src/naga_wasm_backend/output_layout.rs | 10 | 1 | 11 | 90.91% 🟢 |
| src/naga_wasm_backend/types.rs | 15 | 7 | 22 | 68.18% 🟡 |
| src/wasm_gl_emu/device.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 32 | 40 | 72 | 44.44% 🟠 |
| src/wasm_gl_emu/transfer.rs | 10 | 12 | 22 | 45.45% 🟠 |
| src/webgl2_context/blend.rs | 9 | 1 | 10 | 90.00% 🟢 |
| src/webgl2_context/buffers.rs | 15 | 0 | 15 | 100.00% 🟢 |
| src/webgl2_context/drawing.rs | 31 | 1 | 32 | 96.88% 🟢 |
| src/webgl2_context/registry.rs | 13 | 0 | 13 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 149 | 16 | 165 | 90.30% 🟢 |
| src/webgl2_context/state.rs | 60 | 4 | 64 | 93.75% 🟢 |
| src/webgl2_context/textures.rs | 56 | 0 | 56 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgpu/backend.rs | 79 | 16 | 95 | 83.16% 🟢 |
| src/webgpu/buffer.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/command.rs | 7 | 3 | 10 | 70.00% 🟡 |
| **Total** | **1229** | **672** | **1901** | **64.65% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 313/407 | [1994] `}` | 23.10% 🟠 |
| src/naga_wasm_backend/expressions.rs | 134/346 | [805] `let offset = (*index * element_size) + (component_idx * 4);` | 61.27% 🟡 |
| src/naga_wasm_backend/backend.rs | 44/195 | [110] `code: CodeSection::new(),` | 77.44% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 41/83 | [30] `if let Some(layout) = ctx.private_memory_layout {` | 50.60% 🟡 |
