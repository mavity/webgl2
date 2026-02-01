# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 22 | 1 | 23 | 95.65% 🟢 |
| src/decompiler/ast.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/decompiler/emitter.rs | 19 | 4 | 23 | 82.61% 🟢 |
| src/decompiler/lifter.rs | 19 | 10 | 29 | 65.52% 🟡 |
| src/decompiler/mod.rs | 19 | 2 | 21 | 90.48% 🟢 |
| src/decompiler/module.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/decompiler/parser.rs | 9 | 10 | 19 | 47.37% 🟠 |
| src/decompiler/simplifier.rs | 35 | 1 | 36 | 97.22% 🟢 |
| src/error.rs | 35 | 4 | 39 | 89.74% 🟢 |
| src/lib.rs | 109 | 310 | 419 | 26.01% 🟠 |
| src/naga_wasm_backend/backend.rs | 107 | 9 | 116 | 92.24% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 30 | 2 | 32 | 93.75% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 37 | 42 | 79 | 46.84% 🟠 |
| src/naga_wasm_backend/debug/stub.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 157 | 121 | 278 | 56.47% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 5 | 8 | 13 | 38.46% 🟠 |
| src/naga_wasm_backend/functions/prep.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 7 | 2 | 9 | 77.78% 🟡 |
| src/naga_wasm_backend/output_layout.rs | 10 | 1 | 11 | 90.91% 🟢 |
| src/naga_wasm_backend/types.rs | 6 | 1 | 7 | 85.71% 🟢 |
| src/wasm_gl_emu/device.rs | 32 | 4 | 36 | 88.89% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 3 | 3 | 6 | 50.00% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 17 | 46 | 63 | 26.98% 🟠 |
| src/wasm_gl_emu/transfer.rs | 13 | 44 | 57 | 22.81% 🟠 |
| src/webgl2_context/drawing.rs | 1 | 1 | 2 | 50.00% 🟡 |
| src/webgl2_context/registry.rs | 9 | 1 | 10 | 90.00% 🟢 |
| src/webgl2_context/shaders.rs | 164 | 17 | 181 | 90.61% 🟢 |
| src/webgl2_context/state.rs | 58 | 6 | 64 | 90.63% 🟢 |
| src/webgl2_context/textures.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 45 | 20 | 65 | 69.23% 🟡 |
| src/webgpu/adapter.rs | 4 | 5 | 9 | 44.44% 🟠 |
| src/webgpu/backend.rs | 106 | 25 | 131 | 80.92% 🟢 |
| src/webgpu/buffer.rs | 3 | 2 | 5 | 60.00% 🟡 |
| src/webgpu/command.rs | 4 | 1 | 5 | 80.00% 🟢 |
| src/webgpu/shader.rs | 7 | 0 | 7 | 100.00% 🟢 |
| **Total** | **1132** | **703** | **1835** | **61.69% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 310/419 | [1994] `data_ptr: *const u8,` | 26.01% 🟠 |
| src/naga_wasm_backend/expressions.rs | 121/278 | [33] `let expr = &ctx.module.global_expressions[expr_handle];` | 56.47% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 46/63 | [1015] `if state.color_mask.r {` | 26.98% 🟠 |
| src/wasm_gl_emu/transfer.rs | 44/57 | [387] `dest[dst_off] = (r5 << 3) | (r5 >> 2);` | 22.81% 🟠 |
