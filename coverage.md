# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/ast.rs | 3 | 2 | 5 | 60.00% 🟡 |
| src/decompiler/emitter.rs | 26 | 6 | 32 | 81.25% 🟢 |
| src/decompiler/lifter.rs | 12 | 10 | 22 | 54.55% 🟡 |
| src/decompiler/mod.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/decompiler/parser.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/decompiler/simplifier.rs | 37 | 2 | 39 | 94.87% 🟢 |
| src/error.rs | 23 | 8 | 31 | 74.19% 🟡 |
| src/lib.rs | 80 | 324 | 404 | 19.80% 🔴 |
| src/naga_wasm_backend/backend.rs | 120 | 56 | 176 | 68.18% 🟡 |
| src/naga_wasm_backend/call_lowering.rs | 21 | 7 | 28 | 75.00% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 20 | 59 | 79 | 25.32% 🟠 |
| src/naga_wasm_backend/debug/stub.rs | 11 | 6 | 17 | 64.71% 🟡 |
| src/naga_wasm_backend/expressions.rs | 129 | 143 | 272 | 47.43% 🟠 |
| src/naga_wasm_backend/function_abi.rs | 35 | 1 | 36 | 97.22% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 5 | 4 | 9 | 55.56% 🟡 |
| src/naga_wasm_backend/mod.rs | 6 | 3 | 9 | 66.67% 🟡 |
| src/naga_wasm_backend/output_layout.rs | 2 | 2 | 4 | 50.00% 🟡 |
| src/naga_wasm_backend/types.rs | 25 | 1 | 26 | 96.15% 🟢 |
| src/wasm_gl_emu/device.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 0 | 1 | 1 | 0.00% 🟡 |
| src/wasm_gl_emu/transfer.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 29 | 1 | 30 | 96.67% 🟢 |
| src/webgl2_context/drawing.rs | 22 | 3 | 25 | 88.00% 🟢 |
| src/webgl2_context/registry.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 144 | 22 | 166 | 86.75% 🟢 |
| src/webgl2_context/state.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/webgl2_context/textures.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 35 | 6 | 41 | 85.37% 🟢 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 56 | 73 | 129 | 43.41% 🟠 |
| src/webgpu/buffer.rs | 0 | 3 | 3 | 0.00% 🟡 |
| src/webgpu/command.rs | 1 | 4 | 5 | 20.00% 🟡 |
| **Total** | **894** | **747** | **1641** | **54.48% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 324/404 | [335] `/// Create a context with flags (bit0 = shader debug).` | 19.80% 🔴 |
| src/naga_wasm_backend/expressions.rs | 143/272 | [1417] `ctx.wasm_func.instruction(&Instruction::I32Sub);` | 47.43% 🟠 |
| src/webgpu/backend.rs | 73/129 | [2557] `fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fm...` | 43.41% 🟠 |
| src/naga_wasm_backend/control_flow.rs | 59/79 | [172] `for (s, s_span) in block.span_iter() {` | 25.32% 🟠 |
