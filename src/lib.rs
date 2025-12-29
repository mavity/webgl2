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
    fn wasm_execute_shader(
        ctx: u32,
        type_: u32,
        attr_ptr: i32,
        uniform_ptr: i32,
        varying_ptr: i32,
        private_ptr: i32,
        texture_ptr: i32,
    );
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn print(_ptr: *const u8, _len: usize) {}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn wasm_execute_shader(
    _ctx: u32,
    _type_: u32,
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
            attr_ptr as i32,
            uniform_ptr as i32,
            varying_ptr as i32,
            private_ptr as i32,
            texture_ptr as i32,
        );
    }
}

// Re-export commonly used types
pub use glsl_introspection::ResourceManifest;
pub use js_codegen::generate_harness;
pub use naga_wasm_backend::{BackendError, WasmBackend, WasmBackendConfig, WasmModule};
pub use wasm_gl_emu::RuntimeError;
#[cfg(feature = "cli")]
pub use wasm_gl_emu::ShaderRuntime;

// Legacy WebGL2 convenience helpers removed.
// The implementation now lives in the `webgl2_context` module and is
// exposed via the `wasm_ctx_*/wasm_*` exports defined below.
//
// Old, unsafe `static mut` helpers (framebuffer, texture arrays, etc.)
// were intentionally removed to centralize the implementation in
// `src/webgl2_context.rs` which provides a safe, handle-based API.

// ============================================================================
// NEW: WebGL2 Prototype Exports (Schema v2)
// Follows docs/1.1.1-webgl2-prototype.md
// ============================================================================

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

// ---- Framebuffer Operations ----

/// Create a framebuffer in the given context.
/// Returns framebuffer handle (0 on failure).
#[no_mangle]
pub extern "C" fn wasm_ctx_create_framebuffer(ctx: u32) -> u32 {
    webgl2_context::ctx_create_framebuffer(ctx)
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
    webgl2_context::ctx_depth_func(ctx, func)
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

// ---- Shader and Program Operations ----

/// Create a shader.
#[no_mangle]
pub extern "C" fn wasm_ctx_create_shader(ctx: u32, type_: u32) -> u32 {
    webgl2_context::ctx_create_shader(ctx, type_)
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

/// Set verbosity level.
#[no_mangle]
pub extern "C" fn wasm_ctx_set_verbosity(ctx: u32, level: u32) -> u32 {
    webgl2_context::ctx_set_verbosity(ctx, level)
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
pub extern "C" fn wasm_webgpu_create_sampler(ctx_handle: u32, device_handle: u32) -> u32 {
    webgpu::texture::create_sampler(ctx_handle, device_handle)
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
    };

    webgpu::pipeline::create_render_pipeline(ctx_handle, device_handle, config)
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
    js_log(3, "wasm_webgpu_create_bind_group_layout called");
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
    js_log(3, "wasm_webgpu_create_bind_group called");
    let entries = std::slice::from_raw_parts(entries_ptr, entries_len);
    webgpu::bind_group::create_bind_group(ctx_handle, device_handle, layout_handle, entries)
}

/// Run a render pass with buffered commands.
///
/// # Safety
///
/// This function is unsafe because it takes raw pointers.
#[no_mangle]
pub unsafe extern "C" fn wasm_webgpu_command_encoder_run_render_pass(
    ctx_handle: u32,
    encoder_handle: u32,
    view_handle: u32,
    load_op: u32,
    store_op: u32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    commands_ptr: *const u32,
    commands_len: usize,
) -> u32 {
    let commands = std::slice::from_raw_parts(commands_ptr, commands_len);
    let config = webgpu::command::RenderPassConfig {
        view_handle,
        load_op,
        store_op,
        clear_r,
        clear_g,
        clear_b,
        clear_a,
    };
    webgpu::command::command_encoder_run_render_pass(ctx_handle, encoder_handle, config, commands)
}

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
pub extern "C" fn wasm_webgpu_command_encoder_begin_render_pass_1_color(
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
    webgpu::command::command_encoder_begin_render_pass_1_color(ctx_handle, encoder_handle, config)
}

// ============================================================================
// Renderbuffer Exports
// ============================================================================

#[no_mangle]
pub extern "C" fn wasm_ctx_create_renderbuffer(ctx: u32) -> u32 {
    webgl2_context::ctx_create_renderbuffer(ctx)
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
