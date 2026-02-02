# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 23 | 4 | 27 | 85.19% 🟢 |
| src/decompiler/ast.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/decompiler/emitter.rs | 7 | 2 | 9 | 77.78% 🟡 |
| src/decompiler/lifter.rs | 6 | 11 | 17 | 35.29% 🟠 |
| src/decompiler/simplifier.rs | 34 | 4 | 38 | 89.47% 🟢 |
| src/error.rs | 28 | 4 | 32 | 87.50% 🟢 |
| src/lib.rs | 90 | 300 | 390 | 23.08% 🟠 |
| src/naga_wasm_backend/backend.rs | 128 | 14 | 142 | 90.14% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 23 | 4 | 27 | 85.19% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 43 | 38 | 81 | 53.09% 🟡 |
| src/naga_wasm_backend/debug/stub.rs | 36 | 0 | 36 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 155 | 133 | 288 | 53.82% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 24 | 6 | 30 | 80.00% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 12 | 0 | 12 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 3 | 1 | 4 | 75.00% 🟡 |
| src/wasm_gl_emu/device.rs | 44 | 10 | 54 | 81.48% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/wasm_gl_emu/rasterizer.rs | 71 | 12 | 83 | 85.54% 🟢 |
| src/wasm_gl_emu/transfer.rs | 34 | 20 | 54 | 62.96% 🟡 |
| src/webgl2_context/drawing.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 11 | 1 | 12 | 91.67% 🟢 |
| src/webgl2_context/shaders.rs | 164 | 10 | 174 | 94.25% 🟢 |
| src/webgl2_context/state.rs | 49 | 13 | 62 | 79.03% 🟡 |
| src/webgl2_context/textures.rs | 13 | 0 | 13 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 41 | 19 | 60 | 68.33% 🟡 |
| src/webgl2_context/vaos.rs | 21 | 5 | 26 | 80.77% 🟢 |
| src/webgpu/adapter.rs | 7 | 7 | 14 | 50.00% 🟡 |
| src/webgpu/backend.rs | 47 | 10 | 57 | 82.46% 🟢 |
| src/webgpu/buffer.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/webgpu/command.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/webgpu/shader.rs | 1 | 4 | 5 | 20.00% 🟡 |
| **Total** | **1156** | **632** | **1788** | **64.65% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 300/390 | [76] `pub static mut ACTIVE_TEXTURE_PTR: u32 = 0;` | 23.08% 🟠 |
| src/naga_wasm_backend/expressions.rs | 133/288 | [1460] `ctx.wasm_func` | 53.82% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 38/81 | [172] `for (s, s_span) in block.span_iter() {` | 53.09% 🟡 |
| src/wasm_gl_emu/transfer.rs | 20/54 | [341] `let val = src.data[src_off + i] as f32 / 255.0;` | 62.96% 🟡 |
