# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 18 | 1 | 19 | 94.74% 🟢 |
| src/decompiler/ast.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/decompiler/emitter.rs | 35 | 2 | 37 | 94.59% 🟢 |
| src/decompiler/lifter.rs | 11 | 14 | 25 | 44.00% 🟠 |
| src/decompiler/simplifier.rs | 30 | 1 | 31 | 96.77% 🟢 |
| src/error.rs | 13 | 2 | 15 | 86.67% 🟢 |
| src/lib.rs | 87 | 299 | 386 | 22.54% 🟠 |
| src/naga_wasm_backend/backend.rs | 173 | 24 | 197 | 87.82% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 22 | 3 | 25 | 88.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 37 | 41 | 78 | 47.44% 🟠 |
| src/naga_wasm_backend/debug/stub.rs | 25 | 9 | 34 | 73.53% 🟡 |
| src/naga_wasm_backend/expressions.rs | 143 | 155 | 298 | 47.99% 🟠 |
| src/naga_wasm_backend/function_abi.rs | 31 | 0 | 31 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 4 | 2 | 6 | 66.67% 🟡 |
| src/naga_wasm_backend/functions/registry.rs | 0 | 1 | 1 | 0.00% 🟡 |
| src/naga_wasm_backend/memory_layout.rs | 16 | 4 | 20 | 80.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 5 | 5 | 10 | 50.00% 🟡 |
| src/naga_wasm_backend/output_layout.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/wasm_gl_emu/device.rs | 49 | 12 | 61 | 80.33% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 3 | 1 | 4 | 75.00% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 52 | 58 | 110 | 47.27% 🟠 |
| src/wasm_gl_emu/transfer.rs | 11 | 16 | 27 | 40.74% 🟠 |
| src/webgl2_context/shaders.rs | 160 | 14 | 174 | 91.95% 🟢 |
| src/webgl2_context/state.rs | 80 | 1 | 81 | 98.77% 🟢 |
| src/webgl2_context/textures.rs | 22 | 7 | 29 | 75.86% 🟡 |
| src/webgl2_context/types.rs | 52 | 14 | 66 | 78.79% 🟡 |
| src/webgl2_context/vaos.rs | 25 | 19 | 44 | 56.82% 🟡 |
| src/webgpu/adapter.rs | 3 | 2 | 5 | 60.00% 🟡 |
| src/webgpu/backend.rs | 68 | 20 | 88 | 77.27% 🟡 |
| src/webgpu/command.rs | 1 | 1 | 2 | 50.00% 🟡 |
| **Total** | **1182** | **729** | **1911** | **61.85% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 299/386 | [1994] `) -> u32 {` | 22.54% 🟠 |
| src/naga_wasm_backend/expressions.rs | 155/298 | [1845] `translate_expression(*argument, ctx)?;` | 47.99% 🟠 |
| src/wasm_gl_emu/rasterizer.rs | 58/110 | [370] `match eq {` | 47.27% 🟠 |
| src/naga_wasm_backend/control_flow.rs | 41/78 | [535] `let types = super::types::naga_to_wasm_types(` | 47.44% 🟠 |
