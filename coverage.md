# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/ast.rs | 1 | 1 | 2 | 50.00% 🟡 |
| src/decompiler/emitter.rs | 4 | 2 | 6 | 66.67% 🟡 |
| src/decompiler/lifter.rs | 12 | 10 | 22 | 54.55% 🟡 |
| src/decompiler/mod.rs | 5 | 2 | 7 | 71.43% 🟡 |
| src/decompiler/module.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/decompiler/simplifier.rs | 38 | 10 | 48 | 79.17% 🟡 |
| src/error.rs | 23 | 9 | 32 | 71.88% 🟡 |
| src/lib.rs | 132 | 302 | 434 | 30.41% 🟠 |
| src/naga_wasm_backend/backend.rs | 154 | 24 | 178 | 86.52% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 20 | 4 | 24 | 83.33% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 32 | 43 | 75 | 42.67% 🟠 |
| src/naga_wasm_backend/debug/stub.rs | 0 | 4 | 4 | 0.00% 🟡 |
| src/naga_wasm_backend/expressions.rs | 160 | 124 | 284 | 56.34% 🟡 |
| src/naga_wasm_backend/function_abi.rs | 8 | 17 | 25 | 32.00% 🟠 |
| src/naga_wasm_backend/functions/prep.rs | 8 | 1 | 9 | 88.89% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 4 | 1 | 5 | 80.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/naga_wasm_backend/types.rs | 24 | 9 | 33 | 72.73% 🟡 |
| src/wasm_gl_emu/device.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 3 | 2 | 5 | 60.00% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 40 | 26 | 66 | 60.61% 🟡 |
| src/wasm_gl_emu/transfer.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/webgl2_context/buffers.rs | 27 | 1 | 28 | 96.43% 🟢 |
| src/webgl2_context/drawing.rs | 25 | 5 | 30 | 83.33% 🟢 |
| src/webgl2_context/registry.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 137 | 13 | 150 | 91.33% 🟢 |
| src/webgl2_context/state.rs | 13 | 0 | 13 | 100.00% 🟢 |
| src/webgl2_context/textures.rs | 22 | 2 | 24 | 91.67% 🟢 |
| src/webgl2_context/types.rs | 28 | 9 | 37 | 75.68% 🟡 |
| src/webgpu/adapter.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/webgpu/backend.rs | 59 | 21 | 80 | 73.75% 🟡 |
| **Total** | **1020** | **642** | **1662** | **61.37% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 302/434 | [1502] `#[no_mangle]` | 30.41% 🟠 |
| src/naga_wasm_backend/expressions.rs | 124/284 | [1483] `ctx.wasm_func.instruction(&Instruction::LocalSet(temp_b));` | 56.34% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 43/75 | [159] `match stmt {` | 42.67% 🟠 |
| src/wasm_gl_emu/rasterizer.rs | 26/66 | [945] `att.data[color_idx..color_idx + 4]` | 60.61% 🟡 |
