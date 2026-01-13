# Coverage Report

> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%

| File | Lines Covered | Lines Missed | Total Lines | Coverage |
|---|---|---|---|---:|
| src/decompiler/emitter.rs | 0 | 22 | 22 | 0.00% 🔴 |
| src/decompiler/lifter.rs | 0 | 39 | 39 | 0.00% 🔴 |
| src/decompiler/mod.rs | 0 | 13 | 13 | 0.00% 🔴 |
| src/decompiler/module.rs | 0 | 5 | 5 | 0.00% 🟡 |
| src/decompiler/parser.rs | 0 | 16 | 16 | 0.00% 🔴 |
| src/decompiler/simplifier.rs | 2 | 65 | 67 | 2.99% 🔴 |
| src/error.rs | 3 | 20 | 23 | 13.04% 🔴 |
| src/lib.rs | 163 | 325 | 488 | 33.40% 🟠 |
| src/naga_wasm_backend/backend.rs | 45 | 11 | 56 | 80.36% 🟢 |
| src/naga_wasm_backend/call_lowering.rs | 5 | 4 | 9 | 55.56% 🟡 |
| src/naga_wasm_backend/control_flow.rs | 16 | 28 | 44 | 36.36% 🟠 |
| src/naga_wasm_backend/debug/stub.rs | 0 | 21 | 21 | 0.00% 🔴 |
| src/naga_wasm_backend/expressions.rs | 22 | 49 | 71 | 30.99% 🟠 |
| src/naga_wasm_backend/function_abi.rs | 5 | 19 | 24 | 20.83% 🟠 |
| src/naga_wasm_backend/mod.rs | 1 | 0 | 1 | 100.00% 🟢 |
| src/naga_wasm_backend/output_layout.rs | 2 | 2 | 4 | 50.00% 🟡 |
| src/naga_wasm_backend/types.rs | 4 | 2 | 6 | 66.67% 🟡 |
| src/wasm_gl_emu/rasterizer.rs | 0 | 14 | 14 | 0.00% 🔴 |
| src/webgl2_context/buffers.rs | 0 | 17 | 17 | 0.00% 🔴 |
| src/webgl2_context/drawing.rs | 0 | 36 | 36 | 0.00% 🔴 |
| src/webgl2_context/framebuffers.rs | 0 | 8 | 8 | 0.00% 🔴 |
| src/webgl2_context/registry.rs | 5 | 1 | 6 | 83.33% 🟢 |
| src/webgl2_context/renderbuffers.rs | 0 | 10 | 10 | 0.00% 🔴 |
| src/webgl2_context/shaders.rs | 51 | 50 | 101 | 50.50% 🟡 |
| src/webgl2_context/state.rs | 0 | 15 | 15 | 0.00% 🔴 |
| src/webgl2_context/textures.rs | 0 | 11 | 11 | 0.00% 🔴 |
| src/webgl2_context/types.rs | 2 | 8 | 10 | 20.00% 🟠 |
| src/webgl2_context/vaos.rs | 0 | 36 | 36 | 0.00% 🔴 |
| src/webgpu/adapter.rs | 0 | 2 | 2 | 0.00% 🟡 |
| src/webgpu/backend.rs | 0 | 50 | 50 | 0.00% 🔴 |
| **Total** | **326** | **899** | **1225** | **26.61% 🟠** |

## Top Missed Files

| File | Lines Missed | Illustrative Line | Coverage |
|---|---|---|---:|
| src/lib.rs | 325/488 | [1042] `}` | 33.40% 🟠 |
| src/decompiler/simplifier.rs | 65/67 | [232] `rewrite!("and-self"; "(& ?a ?a)" => "?a"),` | 2.99% 🔴 |
| src/webgl2_context/shaders.rs | 50/101 | [597] `if !varying_locations.values().any(|&v| v == *loc) {` | 50.50% 🟡 |
| src/webgpu/backend.rs | 50/50 | [919] `}` | 0.00% 🔴 |
