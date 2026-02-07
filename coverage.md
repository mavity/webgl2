# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 11 | 4 | 15 | 73.33% 🟡 |
| src/decompiler/ast.rs | 1 | 4 | 5 | 20.00% 🟡 |
| src/decompiler/emitter.rs | 21 | 9 | 30 | 70.00% 🟡 |
| src/decompiler/lifter.rs | 8 | 2 | 10 | 80.00% 🟢 |
| src/decompiler/module.rs | 0 | 1 | 1 | 0.00% 🟡 |
| src/decompiler/parser.rs | 12 | 5 | 17 | 70.59% 🟡 |
| src/decompiler/simplifier.rs | 36 | 0 | 36 | 100.00% 🟢 |
| src/error.rs | 31 | 8 | 39 | 79.49% 🟡 |
| src/lib.rs | 145 | 336 | 481 | 30.15% 🟠 |
| src/naga_wasm_backend/backend.rs | 146 | 39 | 185 | 78.92% 🟡 |
| src/naga_wasm_backend/call_lowering.rs | 7 | 4 | 11 | 63.64% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 43 | 24 | 67 | 64.18% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 11 | 16 | 27 | 40.74% 🟠 |
| src/naga_wasm_backend/expressions.rs | 277 | 140 | 417 | 66.43% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 8 | 3 | 11 | 72.73% 🟡 |
| src/naga_wasm_backend/types.rs | 19 | 4 | 23 | 82.61% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 18 | 41 | 59 | 30.51% 🟠 |
| src/wasm_gl_emu/transfer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/webgl2_context/blend.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 14 | 0 | 14 | 100.00% 🟢 |
| src/webgl2_context/drawing.rs | 29 | 0 | 29 | 100.00% 🟢 |
| src/webgl2_context/ephemeral.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 142 | 21 | 163 | 87.12% 🟢 |
| src/webgl2_context/state.rs | 34 | 14 | 48 | 70.83% 🟡 |
| src/webgl2_context/textures.rs | 63 | 0 | 63 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 8 | 5 | 13 | 61.54% 🟡 |
| src/webgl2_context/vaos.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/webgpu/adapter.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/webgpu/backend.rs | 72 | 15 | 87 | 82.76% 🟢 |
| src/webgpu/bind_group.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgpu/buffer.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/webgpu/command.rs | 8 | 6 | 14 | 57.14% 🟡 |
| src/webgpu/shader.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/webgpu/texture.rs | 7 | 4 | 11 | 63.64% 🟡 |
| **Total** | **1232** | **705** | **1937** | **63.60% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 336/481 | [1581] `) -> u32 {` | 30.15% 🟠 |
| src/naga_wasm_backend/expressions.rs | 140/417 | [51] `let expr = &ctx.module.global_expressions[expr_handle];` | 66.43% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 41/59 | [370] `match eq {` | 30.51% 🟠 |
| src/naga_wasm_backend/backend.rs | 39/185 | [172] `let type_index = self.type_count;` | 78.92% 🟡 |
