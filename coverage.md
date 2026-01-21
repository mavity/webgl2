# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/ast.rs | 5 | 1 | 6 | 83.33% 🟢 |
| src/decompiler/emitter.rs | 41 | 7 | 48 | 85.42% 🟢 |
| src/decompiler/lifter.rs | 31 | 27 | 58 | 53.45% 🟡 |
| src/decompiler/module.rs | 1 | 1 | 2 | 50.00% 🟡 |
| src/decompiler/parser.rs | 12 | 1 | 13 | 92.31% 🟢 |
| src/decompiler/simplifier.rs | 42 | 4 | 46 | 91.30% 🟢 |
| src/error.rs | 28 | 3 | 31 | 90.32% 🟢 |
| src/lib.rs | 100 | 329 | 429 | 23.31% 🟠 |
| src/naga_wasm_backend/backend.rs | 106 | 76 | 182 | 58.24% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 24 | 60 | 84 | 28.57% 🟠 |
| src/naga_wasm_backend/debug/stub.rs | 26 | 1 | 27 | 96.30% 🟢 |
| src/naga_wasm_backend/expressions.rs | 78 | 131 | 209 | 37.32% 🟠 |
| src/naga_wasm_backend/function_abi.rs | 32 | 7 | 39 | 82.05% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 5 | 2 | 7 | 71.43% 🟡 |
| src/naga_wasm_backend/types.rs | 17 | 3 | 20 | 85.00% 🟢 |
| src/wasm_gl_emu/device.rs | 0 | 2 | 2 | 0.00% 🟡 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/transfer.rs | 1 | 2 | 3 | 33.33% 🟡 |
| src/webgl2_context/blend.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 28 | 0 | 28 | 100.00% 🟢 |
| src/webgl2_context/drawing.rs | 32 | 5 | 37 | 86.49% 🟢 |
| src/webgl2_context/registry.rs | 6 | 4 | 10 | 60.00% 🟡 |
| src/webgl2_context/shaders.rs | 176 | 21 | 197 | 89.34% 🟢 |
| src/webgl2_context/state.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/webgl2_context/textures.rs | 12 | 0 | 12 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 12 | 0 | 12 | 100.00% 🟢 |
| src/webgl2_context/vaos.rs | 12 | 0 | 12 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 5 | 2 | 7 | 71.43% 🟡 |
| src/webgpu/backend.rs | 62 | 60 | 122 | 50.82% 🟡 |
| src/webgpu/command.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/webgpu/pipeline.rs | 1 | 0 | 1 | 100.00% 🟢 |
| **Total** | **915** | **749** | **1664** | **54.99% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 329/429 | [2004] `) -> u32 {` | 23.31% 🟠 |
| src/naga_wasm_backend/expressions.rs | 131/209 | [927] `if let Some(handle) = found_global {` | 37.32% 🟠 |
| src/naga_wasm_backend/backend.rs | 76/182 | [262] `func.instruction(&Instruction::I32Add); // total_idx` | 58.24% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 60/84 | [233] `if let Some(break_cond) = break_if {` | 28.57% 🟠 |
