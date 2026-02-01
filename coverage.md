# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 23 | 4 | 27 | 85.19% 🟢 |
| src/decompiler/ast.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/decompiler/emitter.rs | 13 | 6 | 19 | 68.42% 🟡 |
| src/decompiler/lifter.rs | 6 | 11 | 17 | 35.29% 🟠 |
| src/decompiler/mod.rs | 2 | 1 | 3 | 66.67% 🟡 |
| src/decompiler/simplifier.rs | 34 | 4 | 38 | 89.47% 🟢 |
| src/error.rs | 28 | 4 | 32 | 87.50% 🟢 |
| src/lib.rs | 87 | 302 | 389 | 22.37% 🟠 |
| src/naga_wasm_backend/backend.rs | 124 | 13 | 137 | 90.51% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 23 | 4 | 27 | 85.19% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 34 | 46 | 80 | 42.50% 🟠 |
| src/naga_wasm_backend/debug/stub.rs | 29 | 0 | 29 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 155 | 134 | 289 | 53.63% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 13 | 0 | 13 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 5 | 5 | 10 | 50.00% 🟡 |
| src/naga_wasm_backend/mod.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 1 | 2 | 3 | 33.33% 🟡 |
| src/wasm_gl_emu/device.rs | 36 | 11 | 47 | 76.60% 🟡 |
| src/wasm_gl_emu/framebuffer.rs | 4 | 3 | 7 | 57.14% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 86 | 2 | 88 | 97.73% 🟢 |
| src/wasm_gl_emu/transfer.rs | 19 | 37 | 56 | 33.93% 🟠 |
| src/webgl2_context/drawing.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/webgl2_context/registry.rs | 11 | 1 | 12 | 91.67% 🟢 |
| src/webgl2_context/shaders.rs | 164 | 10 | 174 | 94.25% 🟢 |
| src/webgl2_context/state.rs | 48 | 14 | 62 | 77.42% 🟡 |
| src/webgl2_context/textures.rs | 14 | 0 | 14 | 100.00% 🟢 |
| src/webgl2_context/types.rs | 41 | 19 | 60 | 68.33% 🟡 |
| src/webgl2_context/vaos.rs | 21 | 5 | 26 | 80.77% 🟢 |
| src/webgpu/adapter.rs | 7 | 7 | 14 | 50.00% 🟡 |
| src/webgpu/backend.rs | 47 | 12 | 59 | 79.66% 🟡 |
| src/webgpu/buffer.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/webgpu/command.rs | 7 | 0 | 7 | 100.00% 🟢 |
| src/webgpu/shader.rs | 1 | 4 | 5 | 20.00% 🟡 |
| **Total** | **1106** | **661** | **1767** | **62.59% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 302/389 | [76] `pub static mut ACTIVE_TEXTURE_PTR: u32 = 0;` | 22.37% 🟠 |
| src/naga_wasm_backend/expressions.rs | 134/289 | [1140] `|h_expr: naga::Handle<Expression>, ctx: &mut TranslationC...` | 53.63% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 46/80 | [172] `for (s, s_span) in block.span_iter() {` | 42.50% 🟠 |
| src/wasm_gl_emu/transfer.rs | 37/56 | [289] `if sx < src_w && sy < src_h {` | 33.93% 🟠 |
