# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/ast.rs | 0 | 1 | 1 | 0.00% 🟡 |
| src/decompiler/emitter.rs | 0 | 25 | 25 | 0.00% 🔴 |
| src/decompiler/lifter.rs | 0 | 38 | 38 | 0.00% 🔴 |
| src/decompiler/mod.rs | 0 | 14 | 14 | 0.00% 🔴 |
| src/decompiler/module.rs | 0 | 4 | 4 | 0.00% 🟡 |
| src/decompiler/parser.rs | 0 | 17 | 17 | 0.00% 🔴 |
| src/decompiler/simplifier.rs | 0 | 72 | 72 | 0.00% 🔴 |
| src/error.rs | 8 | 17 | 25 | 32.00% 🟠 |
| src/lib.rs | 273 | 279 | 552 | 49.46% 🟠 |
| src/naga_wasm_backend/backend.rs | 75 | 5 | 80 | 93.75% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 18 | 0 | 18 | 100.00% 🟢 |
| src/naga_wasm_backend/control_flow.rs | 22 | 46 | 68 | 32.35% 🟠 |
| src/naga_wasm_backend/debug/stub.rs | 20 | 0 | 20 | 100.00% 🟢 |
| src/naga_wasm_backend/expressions.rs | 52 | 137 | 189 | 27.51% 🟠 |
| src/naga_wasm_backend/function_abi.rs | 24 | 0 | 24 | 100.00% 🟢 |
| src/naga_wasm_backend/functions/prep.rs | 4 | 1 | 5 | 80.00% 🟢 |
| src/naga_wasm_backend/functions/registry.rs | 4 | 0 | 4 | 100.00% 🟢 |
| src/naga_wasm_backend/memory_layout.rs | 9 | 0 | 9 | 100.00% 🟢 |
| src/naga_wasm_backend/mod.rs | 0 | 1 | 1 | 0.00% 🟡 |
| src/naga_wasm_backend/output_layout.rs | 3 | 1 | 4 | 75.00% 🟡 |
| src/naga_wasm_backend/types.rs | 11 | 0 | 11 | 100.00% 🟢 |
| src/wasm_gl_emu/device.rs | 17 | 8 | 25 | 68.00% 🟡 |
| src/wasm_gl_emu/framebuffer.rs | 3 | 2 | 5 | 60.00% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 0 | 51 | 51 | 0.00% 🔴 |
| src/wasm_gl_emu/transfer.rs | 10 | 17 | 27 | 37.04% 🟠 |
| src/webgl2_context/blend.rs | 0 | 3 | 3 | 0.00% 🟡 |
| src/webgl2_context/buffers.rs | 8 | 15 | 23 | 34.78% 🟠 |
| src/webgl2_context/drawing.rs | 3 | 12 | 15 | 20.00% 🟠 |
| src/webgl2_context/framebuffers.rs | 11 | 1 | 12 | 91.67% 🟢 |
| src/webgl2_context/registry.rs | 6 | 0 | 6 | 100.00% 🟢 |
| src/webgl2_context/renderbuffers.rs | 10 | 0 | 10 | 100.00% 🟢 |
| src/webgl2_context/shaders.rs | 106 | 28 | 134 | 79.10% 🟡 |
| src/webgl2_context/state.rs | 14 | 6 | 20 | 70.00% 🟡 |
| src/webgl2_context/textures.rs | 17 | 16 | 33 | 51.52% 🟡 |
| src/webgl2_context/types.rs | 14 | 11 | 25 | 56.00% 🟡 |
| src/webgl2_context/vaos.rs | 27 | 8 | 35 | 77.14% 🟡 |
| src/webgpu/adapter.rs | 2 | 0 | 2 | 100.00% 🟢 |
| src/webgpu/backend.rs | 34 | 88 | 122 | 27.87% 🟠 |
| src/webgpu/buffer.rs | 3 | 0 | 3 | 100.00% 🟢 |
| src/webgpu/command.rs | 7 | 0 | 7 | 100.00% 🟢 |
| **Total** | **815** | **924** | **1739** | **46.87% 🟠** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 279/552 | [1045] `) -> u32 {` | 49.46% 🟠 |
| src/naga_wasm_backend/expressions.rs | 137/189 | [1399] `translate_expression_component(*arg, component_idx, ctx)?;` | 27.51% 🟠 |
| src/webgpu/backend.rs | 88/122 | [225] `let info = validator.validate(module).map_err(|e| {` | 27.87% 🟠 |
| src/decompiler/simplifier.rs | 72/72 | [235] `rewrite!("and-self"; "(& ?a ?a)" => "?a"),` | 0.00% 🔴 |
