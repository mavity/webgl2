# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/emitter.rs | 0 | 22 | 22 | 0.00% 🔴 |
| src/decompiler/lifter.rs | 0 | 39 | 39 | 0.00% 🔴 |
| src/decompiler/mod.rs | 0 | 13 | 13 | 0.00% 🔴 |
| src/decompiler/module.rs | 0 | 5 | 5 | 0.00% 🟡 |
| src/decompiler/parser.rs | 0 | 16 | 16 | 0.00% 🔴 |
| src/decompiler/simplifier.rs | 1 | 66 | 67 | 1.49% 🔴 |
| src/error.rs | 2 | 21 | 23 | 8.70% 🔴 |
| src/lib.rs | 11 | 477 | 488 | 2.25% 🔴 |
| src/naga_wasm_backend/backend.rs | 0 | 56 | 56 | 0.00% 🔴 |
| src/naga_wasm_backend/call_lowering.rs | 0 | 9 | 9 | 0.00% 🔴 |
| src/naga_wasm_backend/control_flow.rs | 0 | 44 | 44 | 0.00% 🔴 |
| src/naga_wasm_backend/debug/stub.rs | 0 | 21 | 21 | 0.00% 🔴 |
| src/naga_wasm_backend/expressions.rs | 0 | 71 | 71 | 0.00% 🔴 |
| src/naga_wasm_backend/function_abi.rs | 0 | 24 | 24 | 0.00% 🔴 |
| src/naga_wasm_backend/mod.rs | 0 | 1 | 1 | 0.00% 🟡 |
| src/naga_wasm_backend/output_layout.rs | 0 | 4 | 4 | 0.00% 🟡 |
| src/naga_wasm_backend/types.rs | 0 | 6 | 6 | 0.00% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 0 | 14 | 14 | 0.00% 🔴 |
| src/webgl2_context/buffers.rs | 0 | 17 | 17 | 0.00% 🔴 |
| src/webgl2_context/drawing.rs | 0 | 36 | 36 | 0.00% 🔴 |
| src/webgl2_context/framebuffers.rs | 0 | 8 | 8 | 0.00% 🔴 |
| src/webgl2_context/registry.rs | 5 | 1 | 6 | 83.33% 🟢 |
| src/webgl2_context/renderbuffers.rs | 0 | 10 | 10 | 0.00% 🔴 |
| src/webgl2_context/shaders.rs | 14 | 87 | 101 | 13.86% 🔴 |
| src/webgl2_context/state.rs | 0 | 15 | 15 | 0.00% 🔴 |
| src/webgl2_context/textures.rs | 0 | 11 | 11 | 0.00% 🔴 |
| src/webgl2_context/types.rs | 2 | 8 | 10 | 20.00% 🟠 |
| src/webgl2_context/vaos.rs | 0 | 36 | 36 | 0.00% 🔴 |
| src/webgpu/adapter.rs | 0 | 2 | 2 | 0.00% 🟡 |
| src/webgpu/backend.rs | 0 | 50 | 50 | 0.00% 🔴 |
| **Total** | **35** | **1190** | **1225** | **2.86% 🔴** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 477/488 | [1042] `}` | 2.25% 🔴 |
| src/webgl2_context/shaders.rs | 87/101 | [321] `p.vs_module = vs_module;` | 13.86% 🔴 |
| src/naga_wasm_backend/expressions.rs | 71/71 | [11] `pub fn is_integer_type(type_inner: &TypeInner) -> bool {` | 0.00% 🔴 |
| src/decompiler/simplifier.rs | 66/67 | [232] `rewrite!("and-self"; "(& ?a ?a)" => "?a"),` | 1.49% 🔴 |
