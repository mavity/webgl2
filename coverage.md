# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/coverage.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/decompiler/ast.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/decompiler/emitter.rs | 25 | 3 | 28 | 89.29% 🟢 |
| src/decompiler/lifter.rs | 5 | 1 | 6 | 83.33% 🟢 |
| src/decompiler/mod.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/decompiler/simplifier.rs | 34 | 8 | 42 | 80.95% 🟢 |
| src/error.rs | 28 | 6 | 34 | 82.35% 🟢 |
| src/lib.rs | 82 | 321 | 403 | 20.35% 🟠 |
| src/naga_wasm_backend/backend.rs | 149 | 40 | 189 | 78.84% 🟡 |
| src/naga_wasm_backend/call_lowering.rs | 25 | 4 | 29 | 86.21% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 39 | 41 | 80 | 48.75% 🟠 |
| src/naga_wasm_backend/debug/stub.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 158 | 160 | 318 | 49.69% 🟠 |
| src/naga_wasm_backend/function_abi.rs | 25 | 11 | 36 | 69.44% 🟡 |
| src/naga_wasm_backend/functions/prep.rs | 7 | 2 | 9 | 77.78% 🟡 |
| src/naga_wasm_backend/functions/registry.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 3 | 2 | 5 | 60.00% 🟡 |
| src/naga_wasm_backend/output_layout.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/naga_wasm_backend/types.rs | 15 | 3 | 18 | 83.33% 🟢 |
| src/wasm_gl_emu/device.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/wasm_gl_emu/framebuffer.rs | 3 | 2 | 5 | 60.00% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 12 | 35 | 47 | 25.53% 🟠 |
| src/wasm_gl_emu/transfer.rs | 9 | 9 | 18 | 50.00% 🟡 |
| src/webgl2_context/buffers.rs | 5 | 0 | 5 | 100.00% 🟢 |
| src/webgl2_context/drawing.rs | 32 | 1 | 33 | 96.97% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 2 | 12 | 83.33% 🟢 |
| src/webgl2_context/shaders.rs | 184 | 35 | 219 | 84.02% 🟢 |
| src/webgl2_context/state.rs | 15 | 2 | 17 | 88.24% 🟢 |
| src/webgl2_context/textures.rs | 13 | 3 | 16 | 81.25% 🟢 |
| src/webgl2_context/types.rs | 9 | 3 | 12 | 75.00% 🟡 |
| src/webgpu/adapter.rs | 8 | 0 | 8 | 100.00% 🟢 |
| src/webgpu/backend.rs | 94 | 39 | 133 | 70.68% 🟡 |
| src/webgpu/buffer.rs | 0 | 3 | 3 | 0.00% 🟡 |
| src/webgpu/command.rs | 12 | 5 | 17 | 70.59% 🟡 |
| **Total** | **1035** | **741** | **1776** | **58.28% 🟡** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 321/403 | [2002] `pub unsafe extern "C" fn wasm_webgpu_create_render_pipeline(` | 20.35% 🟠 |
| src/naga_wasm_backend/expressions.rs | 160/318 | [1545] `ctx.wasm_func.instruction(&Instruction::LocalTee(temp_a));` | 49.69% 🟠 |
| src/naga_wasm_backend/control_flow.rs | 41/80 | [385] `let break_depth = total_cases - 1 - i as u32;` | 48.75% 🟠 |
| src/naga_wasm_backend/backend.rs | 40/189 | [263] `func.instruction(&Instruction::LocalGet(l_height));` | 78.84% 🟡 |
