# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 23 | 2 | 25 | 92.00% 🟢 |
| src/decompiler/ast.rs | 2 | 4 | 6 | 33.33% 🟡 |
| src/decompiler/emitter.rs | 34 | 2 | 36 | 94.44% 🟢 |
| src/decompiler/lifter.rs | 19 | 10 | 29 | 65.52% 🟡 |
| src/decompiler/mod.rs | 14 | 0 | 14 | 100.00% 🟢 |
| src/decompiler/module.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/decompiler/parser.rs | 9 | 10 | 19 | 47.37% 🟠 |
| src/decompiler/simplifier.rs | 33 | 2 | 35 | 94.29% 🟢 |
| src/error.rs | 36 | 2 | 38 | 94.74% 🟢 |
| src/lib.rs | 109 | 312 | 421 | 25.89% 🟠 |
| src/naga_wasm_backend/backend.rs | 107 | 9 | 116 | 92.24% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 30 | 2 | 32 | 93.75% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 37 | 42 | 79 | 46.84% 🟠 |
| src/naga_wasm_backend/debug/stub.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 157 | 121 | 278 | 56.47% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 5 | 8 | 13 | 38.46% 🟠 |
| src/naga_wasm_backend/functions/prep.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 7 | 1 | 8 | 87.50% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 9 | 1 | 10 | 90.00% 🟢 |
| src/naga_wasm_backend/types.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/wasm_gl_emu/device.rs | 31 | 4 | 35 | 88.57% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 4 | 5 | 20.00% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 23 | 44 | 67 | 34.33% 🟠 |
| src/wasm_gl_emu/transfer.rs | 13 | 44 | 57 | 22.81% 🟠 |
| src/webgl2_context/drawing.rs | 1 | 1 | 2 | 50.00% 🟡 |
| src/webgl2_context/registry.rs | 9 | 1 | 10 | 90.00% 🟢 |
| src/webgl2_context/shaders.rs | 164 | 17 | 181 | 90.61% 🟢 |
| src/webgl2_context/state.rs | 58 | 6 | 64 | 90.63% 🟢 |
| src/webgl2_context/textures.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 46 | 20 | 66 | 69.70% 🟡 |
| src/webgpu/adapter.rs | 4 | 6 | 10 | 40.00% 🟠 |
| src/webgpu/backend.rs | 103 | 27 | 130 | 79.23% 🟡 |
| src/webgpu/buffer.rs | 1 | 1 | 2 | 50.00% 🟡 |
| src/webgpu/command.rs | 6 | 0 | 6 | 100.00% 🟢 |
| **Total** | **1134** | **703** | **1837** | **61.73% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 312/421 | [1994] `ctx_handle,` | 25.89% 🟠 |
| src/naga_wasm_backend/expressions.rs | 121/278 | [33] `let expr = &ctx.module.global_expressions[expr_handle];` | 56.47% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 44/67 | [565] `att.internal_format,` | 34.33% 🟠 |
| src/wasm_gl_emu/transfer.rs | 44/57 | [387] `dest[dst_off] = (r5 << 3) | (r5 >> 2);` | 22.81% 🟠 |
