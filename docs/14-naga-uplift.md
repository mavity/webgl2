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

## 5. Detailed Implementation Plan & Execution Mandate

**CRITICAL MANDATE**: The implementation must follow a strict incremental pattern.
1.  **One Step at a Time**: Do not attempt to implement multiple groups simultaneously.
2.  **Verify & Commit**: After completing a lead opcode or a group, run the full build (`npm run build`) and the full test suite (`npm test`). **ALL** tests must pass.
3.  **Clean State**: Only proceed to the next step when the codebase is in a stable, passing state.
4. Commit each such safe step AFTER green build and all tests pass as a separate commit.

---

### Phase 1: Straightforward Instructions & patterns

These instructions map directly to WASM operations or simple sequences. They should be implemented first to clear the "low hanging fruit" and build momentum.

#### Group 1.1: Missing Arithmetic & Bitwise
**Pattern**: Direct mapping to `wasm_encoder::Instruction` similar to `BinaryOperator::Add`.
**Reference**: `src/naga_wasm_backend/expressions.rs` -> `translate_expression_component` (Binary match).

*   **`BinaryOperator::Rem` (Float)**:
    *   **Note**: WASM `F32Nearest` does NOT equal modulo. Use logic: `a - b * trunc(a/b)`.
    *   **Ref**: `BinaryOperator::Div`.
*   **`BinaryOperator::And/Or/Xor` (Integer)**:
    *   **Implementation**: Map to `I32And`, `I32Or`, `I32Xor`.
    *   **Ref**: `BinaryOperator::Add`.
*   **`BinaryOperator::ShiftLeft/ShiftRight`**:
    *   **Implementation**: Map to `I32Shl`, `I32ShrS` (signed) or `I32ShrU` (unsigned).
    *   **Ref**: `BinaryOperator::Add`.

#### Group 1.2: Simple Relational
**Pattern**: Floating point comparison returning 0/1 (i32).
**Reference**: `src/naga_wasm_backend/expressions.rs` -> `translate_expression_component` (Relational match).

*   **`RelationalFunction::IsInf`**:
    *   **Implementation**: Check if `abs(x) == Infinity`. In WASM: `f32.abs`, `f32.const inf`, `f32.eq`.
*   **`RelationalFunction::IsNan`**:
    *   **Implementation**: `x != x` is the standard NaN check. In WASM: `f32.ne`.

---

### Phase 2: Complex Pattern - Control Flow

This group requires managing block labels and stack depth.

#### Group 2.1: The Lead - `Statement::Break` (with Block)
**Goal**: Implement `Statement::Block` nesting and `Statement::Break`.
**Lead Task**:
1.  Modify `TranslationContext` in `control_flow.rs` to have a `block_stack` (vector of labels/depths).
2.  Implement `Statement::Block`: Push label, emit `block`, emit statements, emit `end`, pop label.
3.  Implement `Statement::Break`: Emit `br <depth>`.
4.  **Validate**: Create a test case with a nested block and a break that skips code. Ensure build/test pass.

#### Group 2.2: The Pack - Loop, Continue, Switch
**Task**: Once 2.1 is stable, implement the rest.
1.  **`Statement::Loop`**:
    *   Pattern: `loop (label: continue) block (label: break) ... br <continue> end end`.
    *   Push *two* labels to the stack (continue target and break target).
2.  **`Statement::Continue`**:
    *   Emit `br <continue_depth>`.
3.  **`Statement::Switch`**:
    *   Pattern: `block (default)`, `block (case 1)`, `br_table`.
    *   Requires calculating jump offsets.
4.  **`Statement::Kill`**:
    *   Pattern: `unreachable` (trap).

---

### Phase 3: Complex Pattern - Host Math Calls

This group requires modifying the ABI to import functions.

#### Group 3.1: The Lead - `sin(f32)`
**Goal**: Establish the "Import -> Call" pipeline.
**Lead Task**:
1.  Modify `backend.rs` -> `Compiler`: Add `ImportSection`.
2.  Define import "env.gl_sin" (f32 -> f32).
3.  Modify `expressions.rs` -> `Expression::Math`: Handle `MathFunction::Sin`.
4.  Emit `Call <func_idx>`.
5.  **Host Side**: Update `wasm_gl_emu` or test runner to provide `gl_sin`.
6.  **Validate**: Test case calling `sin(0.0)` loops through WASM and returns correct value.

#### Group 3.2: The Pack - Rest of Standard Library
**Task**: Once 3.1 works, mass-implement the rest.
1.  **Scalar Math**: `Cos`, `Tan`, `Asin`, `Acos`, `Atan`, `Sinh`, `Cosh`, `Tanh`, `Exp`, `Log`, `Sqrt`, `Floor`, `Ceil`, `Fract`.
    *   Map them to new imports in `backend.rs`.
2.  **Pow**: `Pow(f32, f32) -> f32`.
3.  **Vectorize**: Ensure `vec4(v).sin()` generates 4 distinct calls.

---

### Phase 4: Complex Pattern - Dynamic Access

This group requires calculated memory offsets.

#### Group 4.1: The Lead - `Array Load [Dynamic Index]`
**Goal**: Read from an array using a runtime variable index.
**Lead Task**:
1.  Modify `expressions.rs` -> `Expression::Access`.
2.  Decode `base` type stride.
3.  Formula: `base_addr + (index * stride) + component_offset`.
4.  Emit: `Get base_ptr`, `Get index`, `Const stride`, `Mul`, `Add`, `Load`.
5.  **Validate**: Test case with `var a: array<f32, 4>; ... x = a[i];`.

#### Group 4.2: The Pack - Store & Matrix
**Task**: Extend logic to writes and matrices.
1.  **`Store Statement`**: Update `control_flow.rs` to use same address calculation for *writing* to dynamic indices.
2.  **Matrix Access**: `mat[col][row]`. Treat column access as array access (stride = vec size).

---

## REMINDER:

Each separate lead opcode, and each group or subgroup should end with SAFE STATE:
`npm run build` succeeds, `npm test` ALL pass. If not, fix ALL failing tests.
NO exclusions, NO cases "these tests failed before". Fix ALL tests, build succeeds.

**Final Verification**:
After Phase 4, run the full "All Compatibility" suite. The backend should now support >90% of conformance tests.
