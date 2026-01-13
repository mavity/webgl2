## Plan: WebGL2 Coverage Expansion (Phases 1-5)

This plan implements the identified missing features in specific, testable phases. Each phase requires updating the Rust backend state, the JS bindings, and critically, the software rasterizer (`wasm_gl_emu`) to honor the new states.

### Step 1: Blending Implementation
Enable transparency and composition by implementing the blending pipeline.
1. **Define State**: Add a `BlendState` struct to `Context` (in webgl2_context) tracking source/destination factors (RGB/Alpha) and equations.
2. **Implement API**: Create src/webgl2_context/blend.rs to implement `blendFunc`, `blendFuncSeparate`, `blendEquation`, and `blendEquationSeparate`.
3. **JS Bindings**: Wire up the functions in webgl2_context.js and export symbols in lib.rs.
4. **Rasterizer Integration**: Update the software rasterizer (likely in wasm_gl_emu) pixel pipeline. It must read the existing framebuffer pixel (dst), combine it with the fragment shader output (src) using the current equations, and write back the result.
5. **Validation**: Update `blendFunc.test.js` and `blendEquation.test.js` to enable tests and verify pixel results.

### Step 2: Depth, Stencil & Color Masking
Implement control over which buffers are written to during draw calls.
1. **Define State**: specific fields to `Context`: `color_mask` (bool[4]), `depth_mask` (bool), and `StencilState` (masks, funcs, ops for front/back).
2. **Implement API**: Create methods for `colorMask`, `depthMask`, `clearStencil`, `stencilFunc*`, `stencilOp*`, `stencilMask*` in state.rs (or separate files if preferred).
3. **Rasterizer Integration**:
    *   **Color Mask**: Filter final pixel writes in wasm_gl_emu.
    *   **Depth Mask**: Prevent updating the depth buffer when mask is false.
    *   **Stencil Test**: Implement the stencil rejection logic and buffer updates (Keep, Zero, Replace, etc.).
4. **Validation**: Enable `colorMask.test.js`, `depthMask.test.js`, and `stencil*.test.js`.

### Step 3: Texture Operations (Mipmaps & Copies)
Enable texture quality features and feedback loops.
1. **Implement `generateMipmap`**: In src/webgl2_context/texture.rs, implement a downsampling algorithm (box filter) to populate levels 1..N of a texture from level 0.
2. **Implement `copyTexImage2D`**: Implement logic to read from the currently bound ReadFramebuffer (or default backbuffer) and upload that data into a texture.
3. **Validation**: Verify using `generateMipmap.test.js` (checking texture sampling at different scales) and `copyTexImage2D.test.js`.

### Step 4: Shader Introspection
Enable querying program metadata.
1. **Resolve Reflection Data**: Ensure the compiler (Naga) preserves uniform/attribute metadata and exposes it to the program instance in Rust.
2. **Implement API**: Add `getActiveUniform` and `getActiveAttrib` in src/webgl2_context/program.rs.
3. **Complex Return Types**: These functions return objects (`WebGLActiveInfo`). Establish a pattern where Rust writes properties (name, size, type) to a shared memory buffer or specific pointer, and the JS binding constructs the JS object.
4. **Validation**: Test with `getActiveUniform.test.js` and `getActiveAttrib.test.js`.

### Step 5: Object Queries (`is*`)
Implement lightweight validation helpers.
1. **Resource Maps**: Leverage the existing resource maps (handle -> instance) in `Context`.
2. **Implement API**: Map `isBuffer`, `isTexture`, `isProgram`, `isShader`, `isFramebuffer`, `isRenderbuffer` to checks in the respective `webgl2_context/*.rs` modules.
3. **Validation**: Enable basic existence tests (e.g., `isBuffer.test.js`).

### Further Considerations
1. **Refining the Rasterizer**: The blending and stencil logic in Step 1/2 is computationally expensive in software. Ensure the implementation handles the "happy path" (blending/stencil disabled) without performance regression.
2. **Testing Strategy**: Since tests for these already exist but expect failures, use a "fix and enable" loop: modify the test file to expect success, then implement the feature until it passes.

# Implementation Timeline

Read [](file:///c%3A/Users/mihai/webgl2/src)

Read [](file:///c%3A/Users/mihai/webgl2/src/webgl2_context)

Read [](file:///c%3A/Users/mihai/webgl2/src/wasm_gl_emu)

Read [](file:///c%3A/Users/mihai/webgl2/src/webgl2_context/mod.rs#1-1), lines 1 to 30

Read [](file:///c%3A/Users/mihai/webgl2/src/webgl2_context/state.rs#1-1), lines 1 to 50

Read [](file:///c%3A/Users/mihai/webgl2/src/webgl2_context/registry.rs#50-50), lines 50 to 110

Read [](file:///c%3A/Users/mihai/webgl2/src/webgl2_context/registry.rs#1-1), lines 1 to 50

Read [](file:///c%3A/Users/mihai/webgl2/src/webgl2_context/types.rs#100-100), lines 100 to 200

Read [](file:///c%3A/Users/mihai/webgl2/src/webgl2_context/types.rs#450-450), lines 450 to 597

Searched for text `struct Context` (`**/src/webgl2_context/types.rs`), 1 result

Read [](file:///c%3A/Users/mihai/webgl2/src/webgl2_context/types.rs#200-200), lines 200 to 280

Read [](file:///c%3A/Users/mihai/webgl2/src/webgl2_context.js#1-1), lines 1 to 110

Read [](file:///c%3A/Users/mihai/webgl2/src/webgl2_context.js#120-120), lines 120 to 180

Read [](file:///c%3A/Users/mihai/webgl2/test/blendFunc.test.js)

Read [](file:///c%3A/Users/mihai/webgl2/test/blendEquation.test.js)

Read [](file:///c%3A/Users/mihai/webgl2/test/blendFuncSeparate.test.js)

Read [](file:///c%3A/Users/mihai/webgl2/test/blendEquationSeparate.test.js)

Read [](file:///c%3A/Users/mihai/webgl2/src/wasm_gl_emu/rasterizer.rs#1-1), lines 1 to 100

Read [](file:///c%3A/Users/mihai/webgl2/src/wasm_gl_emu/rasterizer.rs#250-250), lines 250 to 493

Read [](file:///c%3A/Users/mihai/webgl2/src/wasm_gl_emu/rasterizer.rs#390-390), lines 390 to 460

Read [](file:///c%3A/Users/mihai/webgl2/src/wasm_gl_emu/rasterizer.rs#100-100), lines 100 to 250

Read [](file:///c%3A/Users/mihai/webgl2/src/wasm_gl_emu/state.rs)

## Plan: Implementation of WebGL2 Blending (Step 1)

This plan details the implementation of blending across the compiler, runtime, and emulator layers.

### Steps

1.  **Define Blend State & Logic (Emulator)**
    *   Modify rasterizer.rs:
        *   Define `BlendState` struct containing `enabled` (bool), `src_rgb`, `dst_rgb`, `src_alpha`, `dst_alpha`, `eq_rgb`, `eq_alpha` (all u32 constant representations) and `color` ([f32;4]).
        *   Implement a `blend(src: [f32;4], dst: [f32;4], state: &BlendState) -> [f32;4]` helper function handling the math for standard GL factors (ZERO, ONE, SRC_ALPHA, ONE_MINUS_SRC_ALPHA, etc.) and equations (FUNC_ADD, FUNC_SUBTRACT, etc.).
        *   Update `RenderState` struct to include `pub blend: BlendState`.
        *   Update `rasterize_triangle` (and `draw_point`) to read the destination color from the framebuffer, invoke `blend()`, and write the result, **only if** `blend.enabled` is true.

2.  **Update WebGL2 Context State (Rust Backend)**
    *   Modify types.rs:
        *   Add `blend_state` field to the `Context` struct using the new types or a compatible representation.
        *   Initialize with default values: enabled=false, src=ONE, dst=ZERO, eq=FUNC_ADD.
    *   Modify drawing.rs:
        *   When constructing `RenderState` in `draw_arrays`/`draw_elements`, populate the `blend` field from the context's current state.

3.  **Implement Blending API Functions (Rust Backend)**
    *   Create src/webgl2_context/blend.rs executing:
        *   `ctx_blend_func(ctx, sfactor, dfactor)`
        *   `ctx_blend_func_separate(ctx, srcRGB, dstRGB, srcAlpha, dstAlpha)`
        *   `ctx_blend_equation(ctx, mode)`
        *   `ctx_blend_equation_separate(ctx, modeRGB, modeAlpha)`
        *   `ctx_blend_color(ctx, r, g, b, a)`
    *   These functions must validate handles and Enum variants before updating the `Context`.
    *   Update mod.rs to include the new module.
    *   Modify `ctx_enable`/`ctx_disable` in state.rs to toggle `blend_state.enabled` when `cap` is `GL_BLEND` (0x0BE2).

4.  **Expose to JavaScript & Export Symbols**
    *   Modify lib.rs to export `wasm_ctx_blend_*` symbols.
    *   Modify webgl2_context.js to methods `blendFunc`, `blendEquation`, `blendFuncSeparate`, `blendEquationSeparate`, `blendColor`.
    *   Ensure methods call `_assertNotDestroyed()` and handle errors.

5.  **Validation & Tests**
    *   Update blendFunc.test.js and blendEquation.test.js (and related files).
    *   Remove expectations that these methods throw "not implemented".
    *   Add assertions checking that `getError()` remains `NO_ERROR` after calls.
    *   (Optional but recommended) Add a visual test case rendering two overlapping quads with alpha blending to confirm rasterizer logic.

### Further Considerations
1.  **Optimization**: Reading from the framebuffer in software rasterization is costly (cache misses). Ensure the "blend disabled" path in `rasterize_triangle` remains purely a write operation.
2.  **Type Conversions**: The rasterizer works in `u8` color space internally for storage but blending requires `f32`. Be careful with `u8 -> f32 -> blend -> u8` roundtrip precision.
3.  **Constants**: Ensure all blending constants (`GL_SRC_ALPHA`, etc.) are defined in types.rs or webgl2_context.js.