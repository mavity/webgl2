//! WebGL2 Shader Compiler and Emulator
//!
//! A complete toolchain for compiling GLSL shaders to WebAssembly with DWARF debugging,
//! including a software rasterizer for shader emulation and TypeScript harness generation.
//!
//! # Modules
//!
//! - [`naga_wasm_backend`] - Compile Naga IR to WASM with DWARF debug information
//! - [`wasm_gl_emu`] - Software rasterizer and WASM shader runtime
//! - [`glsl_introspection`] - GLSL parser with annotation extraction
//! - [`js_codegen`] - TypeScript harness code generator

pub mod decompiler;
pub mod error;
pub mod glsl_introspection;
pub mod js_codegen;
pub mod naga_wasm_backend;
pub mod wasm_gl_emu;
pub mod webgl2_context;
pub mod webgpu;

#[cfg(feature = "coverage")]
pub mod coverage;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    fn print(ptr: *const u8, len: usize);
    fn dispatch_uncaptured_error(ptr: *const u8, len: usize);
    fn wasm_execute_shader(
        ctx: u32,
        type_: u32,
        table_idx: u32,
        attr_ptr: i32,
        uniform_ptr: i32,
        varying_ptr: i32,
        private_ptr: i32,
        texture_ptr: i32,
    );
    fn wasm_register_shader(ptr: *const u8, len: usize) -> u32;
}

// Access the linker-provided __heap_base
#[cfg(target_arch = "wasm32")]
extern "C" {
    static __heap_base: i32;
}

#[cfg(not(target_arch = "wasm32"))]
static __heap_base: i32 = 0x200000;

#[no_mangle]
pub extern "C" fn wasm_get_scratch_base() -> u32 {
    let base = unsafe { &__heap_base as *const i32 as u32 };
    // Align to 64KB page boundary for safety and predictability
    (base + 0xFFFF) & !0xFFFF
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn print(_ptr: *const u8, _len: usize) {}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn dispatch_uncaptured_error(_ptr: *const u8, _len: usize) {}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn wasm_register_shader(_ptr: *const u8, _len: usize) -> u32 {
    0
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn wasm_execute_shader(
    _ctx: u32,
    _type_: u32,
    _table_idx: u32,
    _attr_ptr: i32,
    _uniform_ptr: i32,
    _varying_ptr: i32,
    _private_ptr: i32,
    _texture_ptr: i32,
) {
}

pub fn js_print(s: &str) {
    unsafe {
        print(s.as_ptr(), s.len());
    }
}

pub fn js_dispatch_uncaptured_error(s: &str) {
    unsafe {
        dispatch_uncaptured_error(s.as_ptr(), s.len());
    }
}

pub fn js_log(level: u32, s: &str) {
    // Level 0: Error, 1: Warning, 2: Info, 3: Debug
    // For now, we just prefix and print.
    // In the future, we can check a global verbosity level.
    let prefix = match level {
        0 => "ERROR: ",
        1 => "WARN: ",
        2 => "INFO: ",
        3 => "DEBUG: ",
        _ => "",
    };
    js_print(&format!("{}{}", prefix, s));
}

pub fn js_execute_shader(
    ctx: u32,
    type_: u32,
    table_idx: u32,
    attr_ptr: u32,
    uniform_ptr: u32,
    varying_ptr: u32,
    private_ptr: u32,
    texture_ptr: u32,
) {
    unsafe {
        wasm_execute_shader(
            ctx,
            type_,
            table_idx,
            attr_ptr as i32,
            uniform_ptr as i32,
            varying_ptr as i32,
            private_ptr as i32,
            texture_ptr as i32,
        );
    }
}

pub fn js_register_shader(bytes: &[u8]) -> u32 {
    unsafe { wasm_register_shader(bytes.as_ptr(), bytes.len()) }
}

// ============================================================================
// Math Builtins (Skip Host)
// ============================================================================

use micromath::F32Ext;

#[no_mangle]
pub extern "C" fn gl_sin(x: f32) -> f32 {
    x.sin()
}

#[no_mangle]
pub extern "C" fn gl_cos(x: f32) -> f32 {
    x.cos()
}

#[no_mangle]
pub extern "C" fn gl_tan(x: f32) -> f32 {
    x.tan()
}

#[no_mangle]
pub extern "C" fn gl_asin(x: f32) -> f32 {
    x.asin()
}

#[no_mangle]
pub extern "C" fn gl_acos(x: f32) -> f32 {
    x.acos()
}

#[no_mangle]
pub extern "C" fn gl_atan(x: f32) -> f32 {
    x.atan()
}

#[no_mangle]
pub extern "C" fn gl_atan2(y: f32, x: f32) -> f32 {
    y.atan2(x)
}

#[no_mangle]
pub extern "C" fn gl_exp(x: f32) -> f32 {
    x.exp()
}

#[no_mangle]
pub extern "C" fn gl_exp2(x: f32) -> f32 {
    x.exp2()
}

#[no_mangle]
pub extern "C" fn gl_log(x: f32) -> f32 {
    x.ln()
}

#[no_mangle]
pub extern "C" fn gl_log2(x: f32) -> f32 {
    x.log2()
}

#[no_mangle]
pub extern "C" fn gl_pow(base: f32, exp: f32) -> f32 {
    base.powf(exp)
}

#[no_mangle]
pub extern "C" fn gl_sinh(x: f32) -> f32 {
    x.sinh()
}

#[no_mangle]
pub extern "C" fn gl_cosh(x: f32) -> f32 {
    x.cosh()
}

#[no_mangle]
pub extern "C" fn gl_tanh(x: f32) -> f32 {
    x.tanh()
}

#[no_mangle]
pub extern "C" fn gl_asinh(x: f32) -> f32 {
    x.asinh()
}

#[no_mangle]
pub extern "C" fn gl_acosh(x: f32) -> f32 {
    x.acosh()
}

#[no_mangle]
pub extern "C" fn gl_atanh(x: f32) -> f32 {
    x.atanh()
}

#[no_mangle]
pub extern "C" fn gl_sqrt(x: f32) -> f32 {
    x.sqrt()
}

#[no_mangle]
pub extern "C" fn gl_inversesqrt(x: f32) -> f32 {
    1.0 / x.sqrt()
}

#[no_mangle]
pub extern "C" fn gl_abs(x: f32) -> f32 {
    x.abs()
}

#[no_mangle]
pub extern "C" fn gl_sign(x: f32) -> f32 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

#[no_mangle]
pub extern "C" fn gl_floor(x: f32) -> f32 {
    x.floor()
}

#[no_mangle]
pub extern "C" fn gl_ceil(x: f32) -> f32 {
    x.ceil()
}

#[no_mangle]
pub extern "C" fn gl_fract(x: f32) -> f32 {
    x.fract()
}

#[no_mangle]
pub extern "C" fn gl_mod(x: f32, y: f32) -> f32 {
    x - y * (x / y).floor()
}

#[no_mangle]
pub extern "C" fn gl_min(x: f32, y: f32) -> f32 {
    if x < y {
        x
    } else {
        y
    }
}

#[no_mangle]
pub extern "C" fn gl_max(x: f32, y: f32) -> f32 {
    if x > y {
        x
    } else {
        y
    }
}

#[no_mangle]
pub extern "C" fn gl_clamp(x: f32, min_val: f32, max_val: f32) -> f32 {
    if x < min_val {
        min_val
    } else if x > max_val {
        max_val
    } else {
        x
    }
}

#[no_mangle]
pub extern "C" fn gl_mix(x: f32, y: f32, a: f32) -> f32 {
    x * (1.0 - a) + y * a
}

#[no_mangle]
pub extern "C" fn gl_step(edge: f32, x: f32) -> f32 {
    if x < edge {
        0.0
    } else {
        1.0
    }
}

#[no_mangle]
pub extern "C" fn gl_smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).max(0.0).min(1.0);
    t * t * (3.0 - 2.0 * t)
}

// Re-export commonly used types
pub use glsl_introspection::ResourceManifest;
pub use js_codegen::generate_harness;
pub use naga_wasm_backend::{BackendError, WasmBackend, WasmBackendConfig, WasmModule};

// ---- Context Lifecycle ----

/// Create a context with flags (bit0 = shader debug).
#[no_mangle]
pub extern "C" fn wasm_create_context_with_flags(flags: u32, width: u32, height: u32) -> u32 {
    webgl2_context::registry::create_context_with_flags(flags, width, height)
}

/// Destroy a WebGL2 context by handle.
/// Returns errno (0 on success).
#[no_mangle]
pub extern "C" fn wasm_destroy_context(handle: u32) -> u32 {
    webgl2_context::destroy_context(handle)
}

/// Resize the default framebuffer of a context.
/// Returns errno (0 on success).
#[no_mangle]
pub extern "C" fn wasm_ctx_resize(ctx: u32, width: u32, height: u32) -> u32 {
    webgl2_context::state::ctx_resize(ctx, width, height)
}

// ---- Memory Management ----

/// Allocate memory from WASM linear memory.
/// Returns pointer (0 on failure; sets last_error).
#[no_mangle]
pub extern "C" fn wasm_alloc(size: u32) -> u32 {
    webgl2_context::wasm_alloc(size)
}

/// Free memory allocated by wasm_alloc.
/// Returns errno (0 on success).
#[no_mangle]
pub extern "C" fn wasm_free(ptr: u32) -> u32 {
    webgl2_context::wasm_free(ptr)
}

// ---- Error Reporting ----

/// Get pointer to last error message (UTF-8).
#[no_mangle]
pub extern "C" fn wasm_last_error_ptr() -> u32 {
    webgl2_context::last_error_ptr() as u32
}

/// Get length of last error message in bytes.
#[no_mangle]
pub extern "C" fn wasm_last_error_len() -> u32 {
    webgl2_context::last_error_len()
}

// ---- Texture Operations ----

/// Create a texture in the given context.
/// Returns texture handle (0 on failure).
#[no_mangle]
pub extern "C" fn wasm_ctx_create_texture(ctx: u32) -> u32 {
    webgl2_context::ctx_create_texture(ctx)
}

/// Check if object is a texture.
#[no_mangle]
pub extern "C" fn wasm_ctx_is_texture(ctx: u32, handle: u32) -> u32 {
    if webgl2_context::ctx_is_texture(ctx, handle) {
        1
    } else {
        0
    }
}

/// Delete a texture.
/// Returns errno.
#[no_mangle]
pub extern "C" fn wasm_ctx_delete_texture(ctx: u32, tex: u32) -> u32 {
    webgl2_context::ctx_delete_texture(ctx, tex)
}

/// Bind a texture.
/// Returns errno.
#[no_mangle]
pub extern "C" fn wasm_ctx_bind_texture(ctx: u32, target: u32, tex: u32) -> u32 {
    webgl2_context::ctx_bind_texture(ctx, target, tex)
}

/// Set texture parameters.
/// Returns errno.
#[no_mangle]
pub extern "C" fn wasm_ctx_tex_parameter_i(ctx: u32, target: u32, pname: u32, param: i32) -> u32 {
    webgl2_context::ctx_tex_parameter_i(ctx, target, pname, param)
}

/// Upload pixel data to a texture.
/// ptr/len point to RGBA u8 pixel data in WASM linear memory.
/// Returns errno.
#[no_mangle]
pub extern "C" fn wasm_ctx_tex_image_2d(
    ctx: u32,
    target: u32,
    level: i32,
    internal_format: i32,
    width: u32,
    height: u32,
    border: i32,
    format: i32,
    type_: i32,
    ptr: u32,
    len: u32,
) -> u32 {
    webgl2_context::ctx_tex_image_2d(
        ctx,
        target,
        level,
        internal_format,
        width,
        height,
        border,
        format,
        type_,
        ptr,
        len,
    )
}

#[no_mangle]
pub extern "C" fn wasm_ctx_tex_image_3d(
    ctx: u32,
    target: u32,
    level: i32,
    internal_format: i32,
    width: u32,
    height: u32,
    depth: u32,
    border: i32,
    format: i32,
    type_: i32,
    ptr: u32,
    len: u32,
) -> u32 {
    webgl2_context::ctx_tex_image_3d(
        ctx,
        target,
        level,
        internal_format,
        width,
        height,
        depth,
        border,
        format,
        type_,
        ptr,
        len,
    )
}

#[no_mangle]
pub extern "C" fn wasm_ctx_tex_sub_image_2d(
    ctx: u32,
    target: u32,
    level: i32,
    xoffset: i32,
    yoffset: i32,
    width: u32,
    height: u32,
    format: i32,
    type_: i32,
    ptr: u32,
    len: u32,
) -> u32 {
    webgl2_context::ctx_tex_sub_image_2d(
        ctx, target, level, xoffset, yoffset, width, height, format, type_, ptr, len,
    )
}

/// Generate mipmaps.
/// Returns errno.
#[no_mangle]
pub extern "C" fn wasm_ctx_generate_mipmap(ctx: u32, target: u32) -> u32 {
    webgl2_context::ctx_generate_mipmap(ctx, target)
}

/// Copy texture image 2D.
/// Returns errno.
#[no_mangle]
pub extern "C" fn wasm_ctx_copy_tex_image_2d(
    ctx: u32,
    target: u32,
    level: i32,
    internal_format: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    border: i32,
) -> u32 {
    webgl2_context::ctx_copy_tex_image_2d(
        ctx,
        target,
        level,
        internal_format,
        x,
        y,
        width,
        height,
        border,
    )
}

// ---- Framebuffer Operations ----

/// Create a framebuffer in the given context.
/// Returns framebuffer handle (0 on failure).
#[no_mangle]
pub extern "C" fn wasm_ctx_create_framebuffer(ctx: u32) -> u32 {
    webgl2_context::ctx_create_framebuffer(ctx)
}

/// Check if object is a framebuffer.
#[no_mangle]
pub extern "C" fn wasm_ctx_is_framebuffer(ctx: u32, handle: u32) -> u32 {
    if webgl2_context::ctx_is_framebuffer(ctx, handle) {
        1
    } else {
        0
    }
}

/// Delete a framebuffer.
/// Returns errno.
#[no_mangle]
pub extern "C" fn wasm_ctx_delete_framebuffer(ctx: u32, fb: u32) -> u32 {
    webgl2_context::ctx_delete_framebuffer(ctx, fb)
}

/// Bind a framebuffer.
/// Returns errno.
#[no_mangle]
pub extern "C" fn wasm_ctx_bind_framebuffer(ctx: u32, target: u32, fb: u32) -> u32 {
    webgl2_context::ctx_bind_framebuffer(ctx, target, fb)
}

/// Attach a texture to the bound framebuffer.
/// Returns errno.
#[no_mangle]
pub extern "C" fn wasm_ctx_framebuffer_texture2d(
    ctx: u32,
    target: u32,
    attachment: u32,
    textarget: u32,
    tex: u32,
    level: i32,
) -> u32 {
    webgl2_context::ctx_framebuffer_texture2d(ctx, target, attachment, textarget, tex, level)
}

/// Blit a region from the read framebuffer to the draw framebuffer.
#[no_mangle]
pub extern "C" fn wasm_ctx_blit_framebuffer(
    ctx: u32,
    src_x0: i32,
    src_y0: i32,
    src_x1: i32,
    src_y1: i32,
    dst_x0: i32,
    dst_y0: i32,
    dst_x1: i32,
    dst_y1: i32,
    mask: u32,
    filter: u32,
) -> u32 {
    webgl2_context::ctx_blit_framebuffer(
        ctx, src_x0, src_y0, src_x1, src_y1, dst_x0, dst_y0, dst_x1, dst_y1, mask, filter,
    )
}

// ---- Pixel Operations ----

/// Read pixels from the bound framebuffer into dest_ptr.
/// dest_ptr/dest_len point to RGBA u8 buffer in WASM linear memory.
/// Returns errno.
#[no_mangle]
pub extern "C" fn wasm_ctx_read_pixels(
    ctx: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    format: u32,
    type_: u32,
    dest_ptr: u32,
    dest_len: u32,
) -> u32 {
    webgl2_context::ctx_read_pixels(ctx, x, y, width, height, format, type_, dest_ptr, dest_len)
}

// ---- State Management ----

/// Set the clear color.
#[no_mangle]
pub extern "C" fn wasm_ctx_clear_color(ctx: u32, r: f32, g: f32, b: f32, a: f32) -> u32 {
    webgl2_context::ctx_clear_color(ctx, r, g, b, a)
}

/// Clear buffers.
#[no_mangle]
pub extern "C" fn wasm_ctx_clear(ctx: u32, mask: u32) -> u32 {
    webgl2_context::ctx_clear(ctx, mask)
}

#[no_mangle]
pub extern "C" fn wasm_ctx_blend_func(ctx: u32, sfactor: u32, dfactor: u32) -> u32 {
    webgl2_context::ctx_blend_func(ctx, sfactor, dfactor)
}

#[no_mangle]
pub extern "C" fn wasm_ctx_blend_func_separate(
    ctx: u32,
    src_rgb: u32,
    dst_rgb: u32,
    src_alpha: u32,
    dst_alpha: u32,
) -> u32 {
    webgl2_context::ctx_blend_func_separate(ctx, src_rgb, dst_rgb, src_alpha, dst_alpha)
}

#[no_mangle]
pub extern "C" fn wasm_ctx_blend_equation(ctx: u32, mode: u32) -> u32 {
    webgl2_context::ctx_blend_equation(ctx, mode)
}

#[no_mangle]
pub extern "C" fn wasm_ctx_blend_equation_separate(
    ctx: u32,
    mode_rgb: u32,
    mode_alpha: u32,
) -> u32 {
    webgl2_context::ctx_blend_equation_separate(ctx, mode_rgb, mode_alpha)
}

#[no_mangle]
pub extern "C" fn wasm_ctx_blend_color(ctx: u32, r: f32, g: f32, b: f32, a: f32) -> u32 {
    webgl2_context::ctx_blend_color(ctx, r, g, b, a)
}

/// Set the viewport.
#[no_mangle]
pub extern "C" fn wasm_ctx_viewport(ctx: u32, x: i32, y: i32, width: u32, height: u32) -> u32 {
    webgl2_context::ctx_viewport(ctx, x, y, width, height)
}

/// Set the scissor box.
#[no_mangle]
pub extern "C" fn wasm_ctx_scissor(ctx: u32, x: i32, y: i32, width: u32, height: u32) -> u32 {
    webgl2_context::ctx_scissor(ctx, x, y, width, height)
}

/// Set the depth function.
#[no_mangle]
pub extern "C" fn wasm_ctx_depth_func(ctx: u32, func: u32) -> u32 {
    webgl2_context::state::ctx_depth_func(ctx, func)
}

/// Set depth mask.
#[no_mangle]
pub extern "C" fn wasm_ctx_depth_mask(ctx: u32, flag: u32) -> u32 {
    webgl2_context::state::ctx_depth_mask(ctx, flag != 0)
}

/// Set color mask.
#[no_mangle]
pub extern "C" fn wasm_ctx_color_mask(ctx: u32, r: u32, g: u32, b: u32, a: u32) -> u32 {
    webgl2_context::state::ctx_color_mask(ctx, r != 0, g != 0, b != 0, a != 0)
}

/// Set stencil function.
#[no_mangle]
pub extern "C" fn wasm_ctx_stencil_func(ctx: u32, func: u32, ref_: i32, mask: u32) -> u32 {
    webgl2_context::state::ctx_stencil_func(ctx, func, ref_, mask)
}

/// Set stencil function separate.
#[no_mangle]
pub extern "C" fn wasm_ctx_stencil_func_separate(
    ctx: u32,
    face: u32,
    func: u32,
    ref_: i32,
    mask: u32,
) -> u32 {
    webgl2_context::state::ctx_stencil_func_separate(ctx, face, func, ref_, mask)
}

/// Set stencil op.
#[no_mangle]
pub extern "C" fn wasm_ctx_stencil_op(ctx: u32, fail: u32, zfail: u32, zpass: u32) -> u32 {
    webgl2_context::state::ctx_stencil_op(ctx, fail, zfail, zpass)
}

/// Set stencil op separate.
#[no_mangle]
pub extern "C" fn wasm_ctx_stencil_op_separate(
    ctx: u32,
    face: u32,
    fail: u32,
    zfail: u32,
    zpass: u32,
) -> u32 {
    webgl2_context::state::ctx_stencil_op_separate(ctx, face, fail, zfail, zpass)
}

/// Set stencil mask.
#[no_mangle]
pub extern "C" fn wasm_ctx_stencil_mask(ctx: u32, mask: u32) -> u32 {
    webgl2_context::state::ctx_stencil_mask(ctx, mask)
}

/// Set stencil mask separate.
#[no_mangle]
pub extern "C" fn wasm_ctx_stencil_mask_separate(ctx: u32, face: u32, mask: u32) -> u32 {
    webgl2_context::state::ctx_stencil_mask_separate(ctx, face, mask)
}

/// Set the active texture unit.
#[no_mangle]
pub extern "C" fn wasm_ctx_active_texture(ctx: u32, texture: u32) -> u32 {
    webgl2_context::ctx_active_texture(ctx, texture)
}

/// Enable a capability.
#[no_mangle]
pub extern "C" fn wasm_ctx_enable(ctx: u32, cap: u32) -> u32 {
    webgl2_context::ctx_enable(ctx, cap)
}

/// Disable a capability.
#[no_mangle]
pub extern "C" fn wasm_ctx_disable(ctx: u32, cap: u32) -> u32 {
    webgl2_context::ctx_disable(ctx, cap)
}

/// Get the last GL error.
#[no_mangle]
pub extern "C" fn wasm_ctx_get_error(ctx: u32) -> u32 {
    webgl2_context::ctx_get_error(ctx)
}

// ---- Buffer Operations ----

/// Create a buffer.
#[no_mangle]
pub extern "C" fn wasm_ctx_create_buffer(ctx: u32) -> u32 {
    webgl2_context::ctx_create_buffer(ctx)
}

/// Check if object is a buffer.
#[no_mangle]
pub extern "C" fn wasm_ctx_is_buffer(ctx: u32, handle: u32) -> u32 {
    if webgl2_context::ctx_is_buffer(ctx, handle) {
        1
    } else {
        0
    }
}

/// Delete a buffer.
#[no_mangle]
pub extern "C" fn wasm_ctx_delete_buffer(ctx: u32, buf: u32) -> u32 {
    webgl2_context::ctx_delete_buffer(ctx, buf)
}

/// Bind a buffer.
#[no_mangle]
pub extern "C" fn wasm_ctx_bind_buffer(ctx: u32, target: u32, buf: u32) -> u32 {
    webgl2_context::ctx_bind_buffer(ctx, target, buf)
}

/// Upload data to the bound buffer.
#[no_mangle]
pub extern "C" fn wasm_ctx_buffer_data(
    ctx: u32,
    target: u32,
    ptr: u32,
    len: u32,
    usage: u32,
) -> u32 {
    webgl2_context::ctx_buffer_data(ctx, target, ptr, len, usage)
}

/// Update a subset of the bound buffer's data.
#[no_mangle]
pub extern "C" fn wasm_ctx_buffer_sub_data(
    ctx: u32,
    target: u32,
    offset: u32,
    ptr: u32,
    len: u32,
) -> u32 {
    webgl2_context::ctx_buffer_sub_data(ctx, target, offset, ptr, len)
}

#[no_mangle]
pub extern "C" fn wasm_ctx_copy_buffer_sub_data(
    ctx: u32,
    read_target: u32,
    write_target: u32,
    read_offset: u32,
    write_offset: u32,
    size: u32,
) -> u32 {
    webgl2_context::ctx_copy_buffer_sub_data(
        ctx,
        read_target,
        write_target,
        read_offset,
        write_offset,
        size,
    )
}

// ---- Shader and Program Operations ----

/// Create a shader.
#[no_mangle]
pub extern "C" fn wasm_ctx_create_shader(ctx: u32, type_: u32) -> u32 {
    webgl2_context::ctx_create_shader(ctx, type_)
}

/// Check if object is a shader.
#[no_mangle]
pub extern "C" fn wasm_ctx_is_shader(ctx: u32, handle: u32) -> u32 {
    if webgl2_context::ctx_is_shader(ctx, handle) {
        1
    } else {
        0
    }
}

/// Delete a shader.
#[no_mangle]
pub extern "C" fn wasm_ctx_delete_shader(ctx: u32, shader: u32) -> u32 {
    webgl2_context::ctx_delete_shader(ctx, shader)
}

/// Set shader source.
#[no_mangle]
pub extern "C" fn wasm_ctx_shader_source(ctx: u32, shader: u32, ptr: u32, len: u32) -> u32 {
    webgl2_context::ctx_shader_source(ctx, shader, ptr, len)
}

/// Compile a shader.
#[no_mangle]
pub extern "C" fn wasm_ctx_compile_shader(ctx: u32, shader: u32) -> u32 {
    webgl2_context::ctx_compile_shader(ctx, shader)
}

/// Get shader parameter.
#[no_mangle]
pub extern "C" fn wasm_ctx_get_shader_parameter(ctx: u32, shader: u32, pname: u32) -> i32 {
    webgl2_context::ctx_get_shader_parameter(ctx, shader, pname)
}

/// Get shader info log.
#[no_mangle]
pub extern "C" fn wasm_ctx_get_shader_info_log(ctx: u32, shader: u32, ptr: u32, len: u32) -> u32 {
    webgl2_context::ctx_get_shader_info_log(ctx, shader, ptr, len)
}

/// Create a program.
#[no_mangle]
pub extern "C" fn wasm_ctx_create_program(ctx: u32) -> u32 {
    webgl2_context::ctx_create_program(ctx)
}

/// Check if object is a program.
#[no_mangle]
pub extern "C" fn wasm_ctx_is_program(ctx: u32, handle: u32) -> u32 {
    if webgl2_context::ctx_is_program(ctx, handle) {
        1
    } else {
        0
    }
}

/// Delete a program.
#[no_mangle]
pub extern "C" fn wasm_ctx_delete_program(ctx: u32, program: u32) -> u32 {
    webgl2_context::ctx_delete_program(ctx, program)
}

/// Attach a shader to a program.
#[no_mangle]
pub extern "C" fn wasm_ctx_attach_shader(ctx: u32, program: u32, shader: u32) -> u32 {
    webgl2_context::ctx_attach_shader(ctx, program, shader)
}

/// Link a program.
#[no_mangle]
pub extern "C" fn wasm_ctx_link_program(ctx: u32, program: u32) -> u32 {
    webgl2_context::ctx_link_program(ctx, program)
}

/// Get program parameter.
#[no_mangle]
pub extern "C" fn wasm_ctx_get_program_parameter(ctx: u32, program: u32, pname: u32) -> i32 {
    webgl2_context::ctx_get_program_parameter(ctx, program, pname)
}

/// Get program info log.
#[no_mangle]
pub extern "C" fn wasm_ctx_get_program_info_log(ctx: u32, program: u32, ptr: u32, len: u32) -> u32 {
    webgl2_context::ctx_get_program_info_log(ctx, program, ptr, len)
}

/// Register compiled shader function table indices.
/// Called from JS after shader WASM instances are created.
#[no_mangle]
pub extern "C" fn wasm_ctx_register_shader_indices(
    ctx: u32,
    program: u32,
    vs_idx: u32,
    fs_idx: u32,
) -> u32 {
    webgl2_context::ctx_register_shader_indices(ctx, program, vs_idx, fs_idx)
}

/// Get the length of the generated WASM for a program's shader.
#[no_mangle]
pub extern "C" fn wasm_ctx_get_program_wasm_len(ctx: u32, program: u32, shader_type: u32) -> u32 {
    webgl2_context::ctx_get_program_wasm_len(ctx, program, shader_type)
}

/// Get the generated WASM for a program's shader.
#[no_mangle]
pub extern "C" fn wasm_ctx_get_program_wasm(
    ctx: u32,
    program: u32,
    shader_type: u32,
    ptr: u32,
    len: u32,
) -> u32 {
    webgl2_context::ctx_get_program_wasm(ctx, program, shader_type, ptr, len)
}

/// Get attribute location.
#[no_mangle]
pub extern "C" fn wasm_ctx_get_attrib_location(ctx: u32, program: u32, ptr: u32, len: u32) -> i32 {
    webgl2_context::ctx_get_attrib_location(ctx, program, ptr, len)
}

/// Bind attribute location.
#[no_mangle]
pub extern "C" fn wasm_ctx_bind_attrib_location(
    ctx: u32,
    program: u32,
    index: u32,
    ptr: u32,
    len: u32,
) -> u32 {
    webgl2_context::ctx_bind_attrib_location(ctx, program, index, ptr, len)
}

/// Get uniform location.
#[no_mangle]
pub extern "C" fn wasm_ctx_get_uniform_location(ctx: u32, program: u32, ptr: u32, len: u32) -> i32 {
    webgl2_context::ctx_get_uniform_location(ctx, program, ptr, len)
}

/// Set uniform 1f.
#[no_mangle]
pub extern "C" fn wasm_ctx_uniform1f(ctx: u32, location: i32, x: f32) -> u32 {
    webgl2_context::ctx_uniform1f(ctx, location, x)
}

/// Set uniform 2f.
#[no_mangle]
pub extern "C" fn wasm_ctx_uniform2f(ctx: u32, location: i32, x: f32, y: f32) -> u32 {
    webgl2_context::ctx_uniform2f(ctx, location, x, y)
}

/// Set uniform 3f.
#[no_mangle]
pub extern "C" fn wasm_ctx_uniform3f(ctx: u32, location: i32, x: f32, y: f32, z: f32) -> u32 {
    webgl2_context::ctx_uniform3f(ctx, location, x, y, z)
}

/// Set uniform 4f.
#[no_mangle]
pub extern "C" fn wasm_ctx_uniform4f(
    ctx: u32,
    location: i32,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
) -> u32 {
    webgl2_context::ctx_uniform4f(ctx, location, x, y, z, w)
}

/// Set uniform 1i.
#[no_mangle]
pub extern "C" fn wasm_ctx_uniform1i(ctx: u32, location: i32, x: i32) -> u32 {
    webgl2_context::ctx_uniform1i(ctx, location, x)
}

/// Set uniform matrix 4fv.
#[no_mangle]
pub extern "C" fn wasm_ctx_uniform_matrix_4fv(
    ctx: u32,
    location: i32,
    transpose: u32,
    ptr: u32,
    len: u32,
) -> u32 {
    webgl2_context::ctx_uniform_matrix_4fv(ctx, location, transpose != 0, ptr, len)
}

/// Use a program.
#[no_mangle]
pub extern "C" fn wasm_ctx_use_program(ctx: u32, program: u32) -> u32 {
    webgl2_context::ctx_use_program(ctx, program)
}

/// Get active uniform info.
#[no_mangle]
pub extern "C" fn wasm_ctx_get_active_uniform(
    ctx: u32,
    program: u32,
    index: u32,
    size_ptr: u32,
    type_ptr: u32,
    name_ptr: u32,
    name_capacity: u32,
) -> u32 {
    webgl2_context::ctx_get_active_uniform(
        ctx,
        program,
        index,
        size_ptr,
        type_ptr,
        name_ptr,
        name_capacity,
    )
}

/// Get active attribute info.
#[no_mangle]
pub extern "C" fn wasm_ctx_get_active_attrib(
    ctx: u32,
    program: u32,
    index: u32,
    size_ptr: u32,
    type_ptr: u32,
    name_ptr: u32,
    name_capacity: u32,
) -> u32 {
    webgl2_context::ctx_get_active_attrib(
        ctx,
        program,
        index,
        size_ptr,
        type_ptr,
        name_ptr,
        name_capacity,
    )
}

/// Enable vertex attribute array.
#[no_mangle]
pub extern "C" fn wasm_ctx_enable_vertex_attrib_array(ctx: u32, index: u32) -> u32 {
    webgl2_context::ctx_enable_vertex_attrib_array(ctx, index)
}

/// Disable vertex attribute array.
#[no_mangle]
pub extern "C" fn wasm_ctx_disable_vertex_attrib_array(ctx: u32, index: u32) -> u32 {
    webgl2_context::ctx_disable_vertex_attrib_array(ctx, index)
}

/// Vertex attribute pointer.
#[no_mangle]
pub extern "C" fn wasm_ctx_vertex_attrib_pointer(
    ctx: u32,
    index: u32,
    size: i32,
    type_: u32,
    normalized: u32,
    stride: i32,
    offset: u32,
) -> u32 {
    webgl2_context::ctx_vertex_attrib_pointer(
        ctx,
        index,
        size,
        type_,
        normalized != 0,
        stride,
        offset,
    )
}

/// Set vertex attribute default value (1f).
#[no_mangle]
pub extern "C" fn wasm_ctx_vertex_attrib1f(ctx: u32, index: u32, v0: f32) -> u32 {
    webgl2_context::ctx_vertex_attrib1f(ctx, index, v0)
}

/// Set vertex attribute default value (2f).
#[no_mangle]
pub extern "C" fn wasm_ctx_vertex_attrib2f(ctx: u32, index: u32, v0: f32, v1: f32) -> u32 {
    webgl2_context::ctx_vertex_attrib2f(ctx, index, v0, v1)
}

/// Set vertex attribute default value (3f).
#[no_mangle]
pub extern "C" fn wasm_ctx_vertex_attrib3f(ctx: u32, index: u32, v0: f32, v1: f32, v2: f32) -> u32 {
    webgl2_context::ctx_vertex_attrib3f(ctx, index, v0, v1, v2)
}

/// Set vertex attribute default value (4f).
#[no_mangle]
pub extern "C" fn wasm_ctx_vertex_attrib4f(
    ctx: u32,
    index: u32,
    v0: f32,
    v1: f32,
    v2: f32,
    v3: f32,
) -> u32 {
    webgl2_context::ctx_vertex_attrib4f(ctx, index, v0, v1, v2, v3)
}

/// Get vertex attribute parameter.
#[no_mangle]
pub extern "C" fn wasm_ctx_get_vertex_attrib(
    ctx: u32,
    index: u32,
    pname: u32,
    ptr: u32,
    len: u32,
) -> u32 {
    webgl2_context::ctx_get_vertex_attrib_v4(ctx, index, pname, ptr, len)
}

/// Set vertex attribute default value (I4i).
#[no_mangle]
pub extern "C" fn wasm_ctx_vertex_attrib_i4i(
    ctx: u32,
    index: u32,
    v0: i32,
    v1: i32,
    v2: i32,
    v3: i32,
) -> u32 {
    webgl2_context::ctx_vertex_attrib_i4i(ctx, index, v0, v1, v2, v3)
}

/// Set vertex attribute default value (I4ui).
#[no_mangle]
pub extern "C" fn wasm_ctx_vertex_attrib_i4ui(
    ctx: u32,
    index: u32,
    v0: u32,
    v1: u32,
    v2: u32,
    v3: u32,
) -> u32 {
    webgl2_context::ctx_vertex_attrib_i4ui(ctx, index, v0, v1, v2, v3)
}

/// Vertex attribute integer pointer.
// Rebuild trigger
#[no_mangle]
pub extern "C" fn wasm_ctx_vertex_attrib_ipointer(
    ctx: u32,
    index: u32,
    size: i32,
    type_: u32,
    stride: i32,
    offset: u32,
) -> u32 {
    webgl2_context::ctx_vertex_attrib_ipointer(ctx, index, size, type_, stride, offset)
}

/// Vertex attribute divisor.
#[no_mangle]
pub extern "C" fn wasm_ctx_vertex_attrib_divisor(ctx: u32, index: u32, divisor: u32) -> u32 {
    webgl2_context::ctx_vertex_attrib_divisor(ctx, index, divisor)
}

/// Get a parameter (vector version).
#[no_mangle]
pub extern "C" fn wasm_ctx_get_parameter_v(
    ctx: u32,
    pname: u32,
    dest_ptr: u32,
    dest_len: u32,
) -> u32 {
    webgl2_context::ctx_get_parameter_v(ctx, pname, dest_ptr, dest_len)
}

/// Set GL error.
#[no_mangle]
pub extern "C" fn wasm_ctx_set_gl_error(ctx: u32, error: u32) -> u32 {
    webgl2_context::ctx_set_gl_error(ctx, error)
}

/// Get buffer parameter.
#[no_mangle]
pub extern "C" fn wasm_ctx_get_buffer_parameter(ctx: u32, target: u32, pname: u32) -> i32 {
    webgl2_context::ctx_get_buffer_parameter(ctx, target, pname)
}

/// Draw arrays.
#[no_mangle]
pub extern "C" fn wasm_ctx_draw_arrays(ctx: u32, mode: u32, first: i32, count: i32) -> u32 {
    webgl2_context::ctx_draw_arrays(ctx, mode, first, count)
}

/// Draw arrays instanced.
#[no_mangle]
pub extern "C" fn wasm_ctx_draw_arrays_instanced(
    ctx: u32,
    mode: u32,
    first: i32,
    count: i32,
    instance_count: i32,
) -> u32 {
    webgl2_context::ctx_draw_arrays_instanced(ctx, mode, first, count, instance_count)
}

/// Draw elements.
#[no_mangle]
pub extern "C" fn wasm_ctx_draw_elements(
    ctx: u32,
    mode: u32,
    count: i32,
    type_: u32,
    offset: u32,
) -> u32 {
    webgl2_context::ctx_draw_elements(ctx, mode, count, type_, offset)
}

/// Draw elements instanced.
#[no_mangle]
pub extern "C" fn wasm_ctx_draw_elements_instanced(
    ctx: u32,
    mode: u32,
    count: i32,
    type_: u32,
    offset: u32,
    instance_count: i32,
) -> u32 {
    webgl2_context::ctx_draw_elements_instanced(ctx, mode, count, type_, offset, instance_count)
}

/// Get program debug stub.
#[no_mangle]
pub extern "C" fn wasm_ctx_get_program_debug_stub(
    ctx: u32,
    program: u32,
    shader_type: u32,
    ptr: u32,
    len: u32,
) -> u32 {
    webgl2_context::ctx_get_program_debug_stub(ctx, program, shader_type, ptr, len)
}

// ---- WAT Testing Support (docs/1.9-wat-testing.md) ----

/// Get a reference to compiled WASM bytes for a program's shader.
/// Returns a packed u64: low 32 bits = ptr, high 32 bits = len.
/// On failure or missing module, returns 0.
/// The pointer is ephemeral; callers must copy synchronously.
#[no_mangle]
pub extern "C" fn wasm_ctx_get_program_wasm_ref(ctx: u32, program: u32, shader_type: u32) -> u64 {
    let (ptr, len) = webgl2_context::ctx_get_program_wasm_ref(ctx, program, shader_type);
    ((len as u64) << 32) | (ptr as u64)
}

/// Get a reference to the WAT text for a program's shader.
/// Returns a packed u64: low 32 bits = ptr, high 32 bits = len.
/// On failure or missing module, returns 0.
/// The pointer is ephemeral; callers must copy/decode synchronously.
#[no_mangle]
pub extern "C" fn wasm_ctx_get_program_wat_ref(ctx: u32, program: u32, shader_type: u32) -> u64 {
    let (ptr, len) = webgl2_context::ctx_get_program_wat_ref(ctx, program, shader_type);
    ((len as u64) << 32) | (ptr as u64)
}

// ---- GLSL Decompiler Support (docs/11.b-decompile-theory.md) ----

/// Thread-local storage for decompiled GLSL output.
/// This is used to return the decompiled GLSL string to JavaScript.
use std::cell::RefCell;
thread_local! {
    static DECOMPILED_GLSL: RefCell<String> = RefCell::new(String::new());
}

/// Decompile WASM bytes to GLSL and store the result.
/// The wasm_bytes are read from linear memory at the given pointer and length.
/// Returns the length of the decompiled GLSL string, or 0 on error.
///
/// # Safety
///
/// The caller must ensure that `wasm_ptr` points to a valid WASM module in linear memory.
#[no_mangle]
pub unsafe extern "C" fn wasm_decompile_to_glsl(wasm_ptr: u32, wasm_len: u32) -> u32 {
    // Read WASM bytes from linear memory
    let wasm_bytes = std::slice::from_raw_parts(wasm_ptr as *const u8, wasm_len as usize);

    match decompiler::decompile_to_glsl(wasm_bytes) {
        Ok(glsl) => {
            let len = glsl.len() as u32;
            DECOMPILED_GLSL.with(|cell| {
                *cell.borrow_mut() = glsl;
            });
            len
        }
        Err(e) => {
            // Store error message in GLSL output
            let error_msg = format!("// Error: {}", e);
            let len = error_msg.len() as u32;
            DECOMPILED_GLSL.with(|cell| {
                *cell.borrow_mut() = error_msg;
            });
            len
        }
    }
}

/// Get a pointer to the decompiled GLSL string.
/// Must be called after wasm_decompile_to_glsl.
#[no_mangle]
pub extern "C" fn wasm_get_decompiled_glsl_ptr() -> u32 {
    DECOMPILED_GLSL.with(|cell| cell.borrow().as_ptr() as u32)
}

/// Get the length of the decompiled GLSL string.
#[no_mangle]
pub extern "C" fn wasm_get_decompiled_glsl_len() -> u32 {
    DECOMPILED_GLSL.with(|cell| cell.borrow().len() as u32)
}

/// Decompile a single function from WASM bytes to GLSL.
/// Returns the length of the decompiled GLSL string, or 0 on error.
///
/// # Safety
///
/// The caller must ensure that `wasm_ptr` points to a valid WASM module in linear memory.
#[no_mangle]
pub unsafe extern "C" fn wasm_decompile_function_to_glsl(
    wasm_ptr: u32,
    wasm_len: u32,
    func_idx: u32,
) -> u32 {
    let wasm_bytes = std::slice::from_raw_parts(wasm_ptr as *const u8, wasm_len as usize);

    match decompiler::decompile_function_to_glsl(wasm_bytes, func_idx) {
        Ok(glsl) => {
            let len = glsl.len() as u32;
            DECOMPILED_GLSL.with(|cell| {
                *cell.borrow_mut() = glsl;
            });
            len
        }
        Err(e) => {
            let error_msg = format!("// Error: {}", e);
            let len = error_msg.len() as u32;
            DECOMPILED_GLSL.with(|cell| {
                *cell.borrow_mut() = error_msg;
            });
            len
        }
    }
}

// ---- Coverage Support (when enabled) ----

#[cfg(feature = "coverage")]
pub use coverage::{
    wasm_get_lcov_report_len, wasm_get_lcov_report_ptr, wasm_init_coverage, wasm_reset_coverage,
    COV_HITS_PTR, COV_MAP_LEN, COV_MAP_PTR,
};

// ============================================================================
// Vertex Array Object Exports
// ============================================================================

#[no_mangle]
pub extern "C" fn wasm_ctx_create_vertex_array(ctx: u32) -> u32 {
    webgl2_context::ctx_create_vertex_array(ctx)
}

#[no_mangle]
pub extern "C" fn wasm_ctx_delete_vertex_array(ctx: u32, vao: u32) -> u32 {
    webgl2_context::ctx_delete_vertex_array(ctx, vao)
}

#[no_mangle]
pub extern "C" fn wasm_ctx_bind_vertex_array(ctx: u32, vao: u32) -> u32 {
    webgl2_context::ctx_bind_vertex_array(ctx, vao)
}

#[no_mangle]
pub extern "C" fn wasm_ctx_is_vertex_array(ctx: u32, vao: u32) -> u32 {
    webgl2_context::ctx_is_vertex_array(ctx, vao)
}

// ============================================================================
// WebGPU API Exports
// ============================================================================

#[no_mangle]
pub extern "C" fn wasm_webgpu_create_context() -> u32 {
    webgpu::adapter::create_context()
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_destroy_context(handle: u32) -> u32 {
    webgpu::adapter::destroy_context(handle)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_request_adapter(
    ctx_handle: u32,
    power_preference: wgpu_types::PowerPreference,
) -> u32 {
    webgpu::adapter::request_adapter(ctx_handle, power_preference)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_request_device(ctx_handle: u32, adapter_handle: u32) -> u32 {
    webgpu::adapter::request_device(ctx_handle, adapter_handle)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_destroy_device(ctx_handle: u32, device_handle: u32) -> u32 {
    webgpu::adapter::destroy_device(ctx_handle, device_handle)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_get_adapter_features(ctx_handle: u32, adapter_handle: u32) -> u64 {
    webgpu::adapter::get_adapter_features(ctx_handle, adapter_handle)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_get_preferred_canvas_format() -> u32 {
    // 17: rgba8unorm, 19: bgra8unorm
    17
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_get_adapter_limits(
    ctx_handle: u32,
    adapter_handle: u32,
    ptr: *mut u32,
) {
    let limits = webgpu::adapter::get_adapter_limits(ctx_handle, adapter_handle);
    unsafe {
        *ptr.offset(0) = limits.max_texture_dimension_1d;
        *ptr.offset(1) = limits.max_texture_dimension_2d;
        *ptr.offset(2) = limits.max_texture_dimension_3d;
        *ptr.offset(3) = limits.max_texture_array_layers;
        *ptr.offset(4) = limits.max_bind_groups;
        *ptr.offset(5) = 0; // Padding
        *ptr.offset(6) = limits.max_bindings_per_bind_group;
        *ptr.offset(7) = limits.max_dynamic_uniform_buffers_per_pipeline_layout;
        *ptr.offset(8) = limits.max_dynamic_storage_buffers_per_pipeline_layout;
        *ptr.offset(9) = limits.max_sampled_textures_per_shader_stage;
        *ptr.offset(10) = limits.max_samplers_per_shader_stage;
        *ptr.offset(11) = limits.max_storage_buffers_per_shader_stage;
        *ptr.offset(12) = limits.max_storage_textures_per_shader_stage;
        *ptr.offset(13) = limits.max_uniform_buffers_per_shader_stage;
        *ptr.offset(14) = limits.max_uniform_buffer_binding_size;
        *ptr.offset(15) = limits.max_storage_buffer_binding_size;
        *ptr.offset(16) = limits.max_vertex_buffers;
        *ptr.offset(17) = limits.max_vertex_attributes;
        *ptr.offset(18) = limits.max_vertex_buffer_array_stride;
        *ptr.offset(19) = limits.max_immediate_size;
        *ptr.offset(20) = limits.min_uniform_buffer_offset_alignment;
        *ptr.offset(21) = limits.min_storage_buffer_offset_alignment;
        *ptr.offset(22) = 0; // Padding
        *ptr.offset(23) = limits.max_inter_stage_shader_variables;
        *ptr.offset(24) = limits.max_color_attachments;
        *ptr.offset(25) = limits.max_color_attachment_bytes_per_sample;
        *ptr.offset(26) = limits.max_compute_workgroup_storage_size;
        *ptr.offset(27) = limits.max_compute_invocations_per_workgroup;
        *ptr.offset(28) = limits.max_compute_workgroup_size_x;
        *ptr.offset(29) = limits.max_compute_workgroup_size_y;
        *ptr.offset(30) = limits.max_compute_workgroup_size_z;
        *ptr.offset(31) = limits.max_compute_workgroups_per_dimension;
        *ptr.offset(32) = limits.min_uniform_buffer_offset_alignment;
        *ptr.offset(33) = limits.min_storage_buffer_offset_alignment;
        let mbs_ptr = ptr.offset(34) as *mut u64;
        *mbs_ptr = limits.max_buffer_size;
    }
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_push_error_scope(filter: u32) {
    let filter_enum = match filter {
        0 => crate::error::WebGPUErrorFilter::Validation,
        1 => crate::error::WebGPUErrorFilter::OutOfMemory,
        2 => crate::error::WebGPUErrorFilter::Internal,
        _ => crate::error::WebGPUErrorFilter::Validation,
    };
    crate::error::webgpu_push_error_scope(filter_enum);
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_pop_error_scope() -> u32 {
    match crate::error::webgpu_pop_error_scope() {
        Some(_) => 1,
        None => 0,
    }
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_create_buffer(
    ctx_handle: u32,
    device_handle: u32,
    size: u64,
    usage: u32,
    mapped_at_creation: bool,
) -> u32 {
    webgpu::buffer::create_buffer(ctx_handle, device_handle, size, usage, mapped_at_creation)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_buffer_destroy(ctx_handle: u32, buffer_handle: u32) -> u32 {
    webgpu::buffer::destroy_buffer(ctx_handle, buffer_handle)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_buffer_map_async(
    ctx_handle: u32,
    device_handle: u32,
    buffer_handle: u32,
    mode: u32,
    offset: u64,
    size: u64,
) -> u32 {
    webgpu::buffer::buffer_map_async(ctx_handle, device_handle, buffer_handle, mode, offset, size)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_buffer_get_mapped_range(
    ctx_handle: u32,
    buffer_handle: u32,
    offset: u64,
    size: u64,
) -> u32 {
    webgpu::buffer::buffer_get_mapped_range(ctx_handle, buffer_handle, offset, size) as u32
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_buffer_unmap(ctx_handle: u32, buffer_handle: u32) -> u32 {
    webgpu::buffer::buffer_unmap(ctx_handle, buffer_handle)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_create_texture(
    ctx_handle: u32,
    device_handle: u32,
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    mip_level_count: u32,
    sample_count: u32,
    dimension: u32,
    format: u32,
    usage: u32,
) -> u32 {
    let config = webgpu::texture::TextureConfig {
        width,
        height,
        depth_or_array_layers,
        mip_level_count,
        sample_count,
        dimension,
        format,
        usage,
    };
    webgpu::texture::create_texture(ctx_handle, device_handle, config)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_create_texture_view(
    ctx_handle: u32,
    texture_handle: u32,
    format: u32,
    dimension: u32,
    base_mip_level: u32,
    mip_level_count: u32,
    base_array_layer: u32,
    array_layer_count: u32,
    aspect: u32,
) -> u32 {
    let config = webgpu::texture::TextureViewConfig {
        format,
        dimension,
        base_mip_level,
        mip_level_count,
        base_array_layer,
        array_layer_count,
        aspect,
    };
    webgpu::texture::create_texture_view(ctx_handle, texture_handle, config)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_create_sampler(
    ctx_handle: u32,
    device_handle: u32,
    address_mode_u: u32,
    address_mode_v: u32,
    address_mode_w: u32,
    mag_filter: u32,
    min_filter: u32,
    mipmap_filter: u32,
    lod_min_clamp: f32,
    lod_max_clamp: f32,
    compare: u32,
    max_anisotropy: u16,
) -> u32 {
    let config = webgpu::texture::SamplerConfig {
        address_mode_u,
        address_mode_v,
        address_mode_w,
        mag_filter,
        min_filter,
        mipmap_filter,
        lod_min_clamp,
        lod_max_clamp,
        compare,
        max_anisotropy,
    };
    webgpu::texture::create_sampler(ctx_handle, device_handle, config)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_destroy_texture(ctx_handle: u32, texture_handle: u32) -> u32 {
    webgpu::texture::destroy_texture(ctx_handle, texture_handle)
}

/// Create a shader module from WGSL code.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer for the shader code.
#[no_mangle]
pub unsafe extern "C" fn wasm_webgpu_create_shader_module(
    ctx_handle: u32,
    device_handle: u32,
    code_ptr: *const u8,
    code_len: usize,
) -> u32 {
    webgpu::shader::create_shader_module(ctx_handle, device_handle, code_ptr, code_len)
}

/// # Safety
///
/// This function is unsafe because it takes raw pointers.
#[no_mangle]
pub unsafe extern "C" fn wasm_webgpu_create_pipeline_layout(
    ctx_handle: u32,
    device_handle: u32,
    bind_group_layouts_ptr: *const u32,
    bind_group_layouts_len: usize,
) -> u32 {
    webgpu::pipeline::create_pipeline_layout(
        ctx_handle,
        device_handle,
        bind_group_layouts_ptr,
        bind_group_layouts_len,
    )
}

/// Create a render pipeline.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for entry point names.
#[no_mangle]
pub unsafe extern "C" fn wasm_webgpu_create_render_pipeline(
    ctx_handle: u32,
    device_handle: u32,
    vertex_module_handle: u32,
    vertex_entry_ptr: *const u8,
    vertex_entry_len: usize,
    fragment_module_handle: u32,
    fragment_entry_ptr: *const u8,
    fragment_entry_len: usize,
    layout_ptr: *const u32,
    layout_len: usize,
    pipeline_layout_handle: u32,
    primitive_topology: u32,
    depth_format: u32,
    depth_write_enabled: u32,
    depth_compare: u32,
    blend_enabled: u32,
    color_blend_src: u32,
    color_blend_dst: u32,
    color_blend_op: u32,
    alpha_blend_src: u32,
    alpha_blend_dst: u32,
    alpha_blend_op: u32,
) -> u32 {
    let v_entry = {
        let slice = std::slice::from_raw_parts(vertex_entry_ptr, vertex_entry_len);
        std::str::from_utf8_unchecked(slice)
    };
    let f_entry = {
        let slice = std::slice::from_raw_parts(fragment_entry_ptr, fragment_entry_len);
        std::str::from_utf8_unchecked(slice)
    };
    let layout_data = std::slice::from_raw_parts(layout_ptr, layout_len);

    let config = webgpu::pipeline::RenderPipelineConfig {
        vertex_module_handle,
        vertex_entry: v_entry,
        fragment_module_handle,
        fragment_entry: f_entry,
        layout_data,
        pipeline_layout_handle,
        primitive_topology,
        depth_format,
        depth_write_enabled: depth_write_enabled != 0,
        depth_compare,
        blend_enabled: blend_enabled != 0,
        color_blend_src,
        color_blend_dst,
        color_blend_op,
        alpha_blend_src,
        alpha_blend_dst,
        alpha_blend_op,
    };

    webgpu::pipeline::create_render_pipeline(ctx_handle, device_handle, config)
}

/// Get a bind group layout from a render pipeline.
#[no_mangle]
pub extern "C" fn wasm_webgpu_pipeline_get_bind_group_layout(
    ctx_handle: u32,
    pipeline_handle: u32,
    index: u32,
) -> u32 {
    webgpu::pipeline::get_render_pipeline_bind_group_layout(ctx_handle, pipeline_handle, index)
}

/// Create a bind group layout.
///
/// # Safety
///
/// This function is unsafe because it takes raw pointers.
#[no_mangle]
pub unsafe extern "C" fn wasm_webgpu_create_bind_group_layout(
    ctx_handle: u32,
    device_handle: u32,
    entries_ptr: *const u32,
    entries_len: usize,
) -> u32 {
    let entries = std::slice::from_raw_parts(entries_ptr, entries_len);
    webgpu::bind_group::create_bind_group_layout(ctx_handle, device_handle, entries)
}

/// Create a bind group.
///
/// # Safety
///
/// This function is unsafe because it takes raw pointers.
#[no_mangle]
pub unsafe extern "C" fn wasm_webgpu_create_bind_group(
    ctx_handle: u32,
    device_handle: u32,
    layout_handle: u32,
    entries_ptr: *const u32,
    entries_len: usize,
) -> u32 {
    let entries = std::slice::from_raw_parts(entries_ptr, entries_len);
    webgpu::bind_group::create_bind_group(ctx_handle, device_handle, layout_handle, entries)
}

// TODO: why was this removed???
// /// Run a render pass with buffered commands.
// ///
// /// # Safety
// ///
// /// This function is unsafe because it takes raw pointers.
// #[no_mangle]
// pub unsafe extern "C" fn wasm_webgpu_command_encoder_run_render_pass(
//     ctx_handle: u32,
//     encoder_handle: u32,
//     view_handle: u32,
//     load_op: u32,
//     store_op: u32,
//     clear_r: f64,
//     clear_g: f64,
//     clear_b: f64,
//     clear_a: f64,
//     commands_ptr: *const u32,
//     commands_len: usize,
// ) -> u32 {
//     let commands = std::slice::from_raw_parts(commands_ptr, commands_len);
//     let config = webgpu::command::RenderPassConfig {
//         view_handle,
//         load_op,
//         store_op,
//         clear_r,
//         clear_g,
//         clear_b,
//         clear_a,
//     };
//     webgpu::command::command_encoder_run_render_pass(ctx_handle, encoder_handle, config, commands)
// }

#[no_mangle]
pub extern "C" fn wasm_webgpu_create_command_encoder(ctx_handle: u32, device_handle: u32) -> u32 {
    webgpu::command::create_command_encoder(ctx_handle, device_handle)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_command_encoder_finish(ctx_handle: u32, encoder_handle: u32) -> u32 {
    webgpu::command::command_encoder_finish(ctx_handle, encoder_handle)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_command_encoder_copy_buffer_to_buffer(
    ctx_handle: u32,
    encoder_handle: u32,
    source_handle: u32,
    source_offset: u64,
    dest_handle: u32,
    dest_offset: u64,
    size: u64,
) -> u32 {
    webgpu::command::command_encoder_copy_buffer_to_buffer(
        ctx_handle,
        encoder_handle,
        source_handle,
        source_offset,
        dest_handle,
        dest_offset,
        size,
    )
}

/// Submit command buffers to the queue.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer for command buffer handles.
#[no_mangle]
pub unsafe extern "C" fn wasm_webgpu_queue_submit(
    ctx_handle: u32,
    device_handle: u32,
    cb_handles_ptr: *const u32,
    cb_handles_len: usize,
) -> u32 {
    let cb_handles = std::slice::from_raw_parts(cb_handles_ptr, cb_handles_len);
    webgpu::command::queue_submit(ctx_handle, device_handle, cb_handles)
}

#[no_mangle]
pub unsafe extern "C" fn wasm_webgpu_queue_write_buffer(
    ctx_handle: u32,
    device_handle: u32,
    buffer_handle: u32,
    offset: u64,
    data_ptr: *const u8,
    data_len: usize,
) -> u32 {
    let data = std::slice::from_raw_parts(data_ptr, data_len);
    webgpu::command::queue_write_buffer(ctx_handle, device_handle, buffer_handle, offset, data)
}

#[no_mangle]
pub unsafe extern "C" fn wasm_webgpu_queue_write_texture(
    ctx_handle: u32,
    device_handle: u32,
    texture_handle: u32,
    data_ptr: *const u8,
    data_len: usize,
    bytes_per_row: u32,
    rows_per_image: u32,
    width: u32,
    height: u32,
    depth: u32,
) -> u32 {
    let data = std::slice::from_raw_parts(data_ptr, data_len);
    webgpu::command::queue_write_texture(
        ctx_handle,
        device_handle,
        texture_handle,
        data,
        bytes_per_row,
        rows_per_image,
        width,
        height,
        depth,
    )
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_command_encoder_copy_texture_to_buffer(
    ctx_handle: u32,
    encoder_handle: u32,
    source_texture_handle: u32,
    dest_buffer_handle: u32,
    dest_offset: u64,
    dest_bytes_per_row: u32,
    dest_rows_per_image: u32,
    size_width: u32,
    size_height: u32,
    size_depth: u32,
) -> u32 {
    let config = webgpu::command::CopyTextureToBufferConfig {
        source_texture_handle,
        dest_buffer_handle,
        dest_offset,
        dest_bytes_per_row,
        dest_rows_per_image,
        size_width,
        size_height,
        size_depth,
    };
    webgpu::command::command_encoder_copy_texture_to_buffer(ctx_handle, encoder_handle, config)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_command_encoder_begin_render_pass(
    ctx_handle: u32,
    encoder_handle: u32,
    view_handle: u32,
    load_op: u32,
    store_op: u32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
) -> u32 {
    let config = webgpu::command::RenderPassConfig {
        view_handle,
        load_op,
        store_op,
        clear_r,
        clear_g,
        clear_b,
        clear_a,
    };
    webgpu::command::command_encoder_begin_render_pass(ctx_handle, encoder_handle, config)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_render_pass_set_pipeline(
    ctx_handle: u32,
    pass_handle: u32,
    pipeline_handle: u32,
) -> u32 {
    webgpu::command::render_pass_set_pipeline(ctx_handle, pass_handle, pipeline_handle)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_render_pass_set_vertex_buffer(
    ctx_handle: u32,
    pass_handle: u32,
    slot: u32,
    buffer_handle: u32,
    offset: u64,
    size: u64,
) -> u32 {
    webgpu::command::render_pass_set_vertex_buffer(
        ctx_handle,
        pass_handle,
        slot,
        buffer_handle,
        offset,
        size,
    )
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_render_pass_set_index_buffer(
    ctx_handle: u32,
    pass_handle: u32,
    buffer_handle: u32,
    format_id: u32,
    offset: u64,
    size: u64,
) -> u32 {
    webgpu::command::render_pass_set_index_buffer(
        ctx_handle,
        pass_handle,
        buffer_handle,
        format_id,
        offset,
        size,
    )
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_render_pass_draw(
    ctx_handle: u32,
    pass_handle: u32,
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
) -> u32 {
    webgpu::command::render_pass_draw(
        ctx_handle,
        pass_handle,
        vertex_count,
        instance_count,
        first_vertex,
        first_instance,
    )
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_render_pass_draw_indexed(
    ctx_handle: u32,
    pass_handle: u32,
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
) -> u32 {
    webgpu::command::render_pass_draw_indexed(
        ctx_handle,
        pass_handle,
        index_count,
        instance_count,
        first_index,
        base_vertex,
        first_instance,
    )
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_render_pass_set_bind_group(
    ctx_handle: u32,
    pass_handle: u32,
    index: u32,
    bg_handle: u32,
) -> u32 {
    webgpu::command::render_pass_set_bind_group(ctx_handle, pass_handle, index, bg_handle)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_render_pass_set_viewport(
    ctx_handle: u32,
    pass_handle: u32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    min_depth: f32,
    max_depth: f32,
) -> u32 {
    webgpu::command::render_pass_set_viewport(
        ctx_handle,
        pass_handle,
        x,
        y,
        w,
        h,
        min_depth,
        max_depth,
    )
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_render_pass_set_scissor_rect(
    ctx_handle: u32,
    pass_handle: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> u32 {
    webgpu::command::render_pass_set_scissor_rect(ctx_handle, pass_handle, x, y, w, h)
}

#[no_mangle]
pub extern "C" fn wasm_webgpu_render_pass_end(ctx_handle: u32, pass_handle: u32) -> u32 {
    webgpu::command::render_pass_end(ctx_handle, pass_handle)
}

// ============================================================================
// Renderbuffer Exports
// ============================================================================

#[no_mangle]
pub extern "C" fn wasm_ctx_create_renderbuffer(ctx: u32) -> u32 {
    webgl2_context::ctx_create_renderbuffer(ctx)
}

/// Check if object is a renderbuffer.
#[no_mangle]
pub extern "C" fn wasm_ctx_is_renderbuffer(ctx: u32, handle: u32) -> u32 {
    if webgl2_context::ctx_is_renderbuffer(ctx, handle) {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn wasm_ctx_bind_renderbuffer(ctx: u32, target: u32, renderbuffer: u32) -> u32 {
    webgl2_context::ctx_bind_renderbuffer(ctx, target, renderbuffer)
}

#[no_mangle]
pub extern "C" fn wasm_ctx_delete_renderbuffer(ctx: u32, renderbuffer: u32) -> u32 {
    webgl2_context::ctx_delete_renderbuffer(ctx, renderbuffer)
}

#[no_mangle]
pub extern "C" fn wasm_ctx_renderbuffer_storage(
    ctx: u32,
    target: u32,
    internal_format: u32,
    width: i32,
    height: i32,
) -> u32 {
    webgl2_context::ctx_renderbuffer_storage(ctx, target, internal_format, width, height)
}

#[no_mangle]
pub extern "C" fn wasm_ctx_framebuffer_renderbuffer(
    ctx: u32,
    target: u32,
    attachment: u32,
    renderbuffertarget: u32,
    renderbuffer: u32,
) -> u32 {
    webgl2_context::ctx_framebuffer_renderbuffer(
        ctx,
        target,
        attachment,
        renderbuffertarget,
        renderbuffer,
    )
}
