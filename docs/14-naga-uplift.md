# Plan: Implement Full Naga IR Support

This document outlines the roadmap to feature parity for the Naga WASM backend. The implementation is divided into four pragmatic buckets, prioritizing control flow and the standard math library which are essential for running real-world shaders.

## 1. Control Flow (Critical)

Currently, the backend only supports simple `If` and `Block` statements. Loops and advanced branching are missing.

### Goal
Implement `Statement::Loop`, `Break`, `Continue`, `Switch`, and `Kill`.

### Implementation Strategy
Modify `src/naga_wasm_backend/control_flow.rs`:

1.  **Loop & Branching Structure**:
    *   Map Naga `Loop` to WASM `block (label: break) loop (label: continue) ... end end`.
    *   Maintain a generic "Block Context" stack that tracks labels for `Break` and `Continue`.
    *   Implement `Statement::Break`: Emit `br <break_label_depth>`.
    *   Implement `Statement::Continue`: Emit `br <continue_label_depth>`.

2.  **Switch Case**:
    *   Map Naga `Switch` to WASM `block` + `br_table`.
    *   Generate a jump table where indices correspond to case values.
    *   Handle `default` case as the fallback target.

3.  **Kill / Discard**:
    *   Implement `Statement::Kill`.
    *   **Mechanism**: Since WASM doesn't have a native "discard", this must be a trap or a return with a "discarded" flag.
    *   **Pragmatic Approach**: Emit `unreachable` or call a special import `__discard` which traps, allowing the rasterizer to catch and ignore the fragment.

## 2. Math Standard Library & Host Calls

Shaders rely heavily on transcendentals (`sin`, `cos`, `pow`) and vector math (`mix`, `step`, `length`). These are missing and must be provided via the host environment.

### Goal
Implement `Expression::Math` completely by wiring it to a high-performance host math library.

### Implementation Strategy
Modify `src/naga_wasm_backend/expressions.rs` and `backend.rs`:

1.  **Import Definition (`backend.rs`)**:
    *   Define a standard set of imports in the `ImportSection`.
    *   Create a mapping of `naga::MathFunction` -> `Import Index`.
    *   **Host Requirement**: The rasterizer/host **MUST** export these functions.
        *   `gl_sin(f32) -> f32`
        *   `gl_cos(f32) -> f32`
        *   `gl_pow(f32, f32) -> f32`
        *   `gl_sqrt(f32) -> f32` etc.
    *   Use established fast implementations (e.g., from `std` or a specific crate) on the host side.

2.  **Expression Translation (`expressions.rs`)**:
    *   Match `Expression::Math`.
    *   For **scalar** ops (`sin`, `cos`): Emit `call <import_index>`.
    *   For **vector** ops (`vec4.sin`): Iterate over components, emit `call <import_index>` for each lane.
    *   For **built-ins** (`mix`, `smoothstep`):
        *   If simple (like `mix`), inline the WASM instructions: `x * (1-a) + y * a`.
        *   If complex, import a helper function.

3.  **Missing Operators**:
    *   Implement `BinaryOperator::Rem` (floating point modulo).
    *   Implement `RelationalFunction` (All, Any, IsNan, IsInf).

## 3. Complex Access & Types

Support for runtime indexing of arrays and matrices is currently limited.

### Goal
Implement dynamic `Access` and full Matrix algebra.

### Implementation Strategy
Modify `src/naga_wasm_backend/expressions.rs`:

1.  **Dynamic Access**:
    *   Handle `Expression::Access { base, index }`.
    *   Calculate memory address: `base_ptr + (index * stride)`.
    *   **Stride Calculation**: Use `naga::Type` information to determine element size/stride.
    *   Support both `Load` (reading) and `Store` (writing) to these calculated addresses.

2.  **Matrix Algebra**:
    *   Implement `Matrix * Matrix` and `Matrix * Vector`.
    *   **Approach**: Unroll loops inline. A 4x4 matrix multiply is 64 multiplies and 48 adds. This is acceptable for WASM.
    *   Implement `Transpose` and `Determinant`.

## 4. Advanced Features & Exclusions

Some features are necessary for completeness, while others can be stubbed.

### Goal
Fill in the gaps for Textures and Derivatives.

### Implementation Strategy

1.  **Texture Operations**:
    *   **ImageQuery**: Implement `textureSize` by importing a host function `gl_texture_size(unit) -> (width, height)`.
    *   **TexelFetch**: Implement `texelFetch` using direct buffer access or a specific import that doesn't filter.

2.  **Derivatives (`dFdx`, `dFdy`)**:
    *   **Pragmatic Stub**: For now, return `0.0`. Implementing true derivatives requires quad-processing or dual-execution support in the specific rasterizer architecture which is complex.
    *   If the host supports it, import `gl_derivative_x(...)`.

3.  **Compute / WGPU Features**:
    *   **Atomics / Barriers**: Stub with `unreachable` or a "not implemented" trap.
    *   **Storage Buffers**: Treat similar to Uniforms but writable (requires `i32/f32.store` to Global pointers).
    *   Leave these as "Low Priority" or "Won't Fix" until basic graphical shaders are 100% robust.

---

### Callback Mechanism Note
The "Table Call" pattern allows tight integration.
*   **Rasterizer -> Shader**: Calls `entry_point` via Table/Export.
*   **Shader -> Rasterizer**: Calls `Math` / `Texture` functions via Import.

Ensure the `ImportSection` in `backend.rs` perfectly matches the `Exports` provided by the `wasm_gl_emu` or `webgl2_context` host.