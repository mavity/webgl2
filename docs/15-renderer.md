# Renderer Uplift


## Plan: Uplift renderer to support high-precision and float-format textures ✅

TL;DR — The current renderer is hardcoded for 8-bit RGBA (4 bytes/pixel) at every stage from texture allocation to fragment output quantization. This plan transforms the "one-size-fits-all" RGBA8 pipeline into a format-aware architecture that supports `R32F`, `RG32F`, and `RGBA32F` by dynamically calculating strides, removing `[0,1]` clamping in the rasterizer, and updating the WASM sampling logic to handle non-normalized float data.

### Steps
1. **Enable Format-Aware Storage** — Update the `MipLevel` and `Renderbuffer` structures in types.rs to store their `internal_format`. Update texture allocation in textures.rs to calculate buffer sizes using format-specific strides (e.g., 4, 8, or 16 bytes per pixel) instead of the hardcoded `* 4`. 📦

2. **Dequantize the Rasterizer** — Modify the fragment shader execution loop in the rasterizer to bypass the `[0.0, 1.0]` clamping and `* 255.0` conversion. Update the framebuffer write logic to perform a bitwise `f32` write (using `f32::to_bits` or direct `f32` copy) when the target texture format is a float type. ⚡

3. **Uplift WASM Sampling Logic** — Update the `__webgl_texture_sample` implementation in backend.rs. Replace the hardcoded `/ 255.0` normalization with a branch that checks texture metadata: for float textures, it should return the raw `f32` sampled values; for `u8` textures, it continues to use the existing normalization. 🔍

4. **Correct `readPixels` Stride** — Update `ctx_read_pixels` in drawing.rs to validate and use provided `format` and `type` arguments. Ensure the output buffer size check allows for the increased size of float data and that no precision-losing conversion occurs during the copy from the internal framebuffer to the user's buffer. 📤

### Further Considerations
- **Metadata Overhead** — Passing format info to the WASM shader might require adding a field to the `TextureDescriptor` struct.
- **Blending & Filtering** — Linear filtering of R32F textures and blending of float targets will require separate implementation paths in the rasterizer as they are currently optimized for integer math.
- **Compatibility** — This will move the project closer to full WebGL2 compliance, enabling GPGPU-style workflows using high-precision textures.

---

### Key Hardcoding Fixes (Examples)

- **Texture size calculation**:
  Instead of `width * height * 4`, use `width * height * get_bytes_per_pixel(internal_format)`.
- **Sampler logic**: 
  Instead of `v.f32() / 255.0`, use `if (is_float) v.f32() else v.f32() / 255.0`.
- **Fragment Output**:
  Modify the rasterizer loop that stores `fragColor` to be format-indexed, letting `f32` components flow through to `RGBA32F` targets without being squeezed into `u8` bytes.

## SwiftShader-style uplift

For this part we should **complete the generalization refactoring first**. Attempting to integrate SwiftShader-style optimizations (like tiling and quad-rasterization) into a pipeline that still has hardcoded "magic numbers" for RGBA8 would be an architectural nightmare.

By finishing the RGBA8 Uplift above, the "Data Plumbing" problem is solved. Once the renderer knows how to handle different strides and float formats without clamping, treating the "How we traverse pixels" (the SwiftShader part) as an isolated optimization layer is more fluent.


### Integration Plan: The "SShader Uplift"

Once your code is format-aware, here is the 4-step plan to integrate the SwiftShader-style optimizations:

#### Phase 1: The Memory "Swizzle" (Tiling)

Before you touch the rasterizer, you must change how your `MipLevel` buffers are addressed in memory.

* **Current:** Your `textures.rs` likely uses a linear  layout.
* **Target:** Implement **Tiled Linear** storage. Break the texture into  or  pixel tiles.
* **Why:** This ensures that when your WASM code processes a small block of pixels, they are physically contiguous in the `WebAssembly.Memory` buffer, maximizing cache hits.

#### Phase 2: Hierarchical Rasterization (The Coarse-to-Fine Pass)

Replace your "straight line" scanline logic with a two-tier approach:

1. **Coarse Pass:** Divide the screen into  "Super-tiles." Check each triangle's bounding box against these tiles.
2. **Fine Pass:** For each super-tile the triangle touches, invoke a specialized WASM-SIMD kernel to test individual pixels. This drastically reduces the number of "Is this point inside the triangle?" tests you have to perform.

#### Phase 3: Quad-Based SIMD Execution

This is the "SShader Special." Instead of iterating pixel-by-pixel, you process ** Quads**.

* **WASM Lowering:** When you JIT your Naga IR, output code that uses the `v128` type.
* **The Masking Trick:** Calculate the "Edge Equations" for the triangle once for the quad. Use a single SIMD instruction to generate a 4-bit bitmask showing which of the 4 pixels are "inside."
* **Conditional Store:** Only perform the framebuffer write for the bits that are "1" in the mask.

#### Phase 4: The "SamplerCore" Port

Uplift your `__webgl_texture_sample` logic using the patterns found in SwiftShader’s `SamplerCore.cpp`.

* **Vectorized Filtering:** Use SIMD to perform the bilinear interpolation (the "Lerp") of four texels simultaneously.
* **Float Support:** Since you've already "Dequantized" the rasterizer in your current refactoring, these float samples can flow directly into your  or  targets without precision loss.

### Summary of Priority

1. **Current Plan (15-renderer.md):** Finish "Format-Aware Storage" and "Dequantization".
2. **Next Step:** Implement **Tiling** in your texture memory layout.
3. **Final Step:** Rewrite the rasterizer loop to use **SIMD Quads**.

