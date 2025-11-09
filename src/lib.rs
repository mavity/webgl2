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

pub mod naga_wasm_backend;
pub mod wasm_gl_emu;
pub mod glsl_introspection;
pub mod js_codegen;
pub mod webgl2_context;

// Re-export commonly used types
pub use naga_wasm_backend::{WasmBackend, WasmBackendConfig, WasmModule, BackendError};
#[cfg(feature = "cli")]
pub use wasm_gl_emu::ShaderRuntime;
pub use wasm_gl_emu::RuntimeError;
pub use glsl_introspection::ResourceManifest;
pub use js_codegen::generate_harness;

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

/// Create a new WebGL2 context.
/// Returns context handle (0 on failure; sets last_error).
#[no_mangle]
pub extern "C" fn wasm_create_context() -> u32 {
	webgl2_context::create_context()
}

/// Destroy a WebGL2 context by handle.
/// Returns errno (0 on success).
#[no_mangle]
pub extern "C" fn wasm_destroy_context(handle: u32) -> u32 {
	webgl2_context::destroy_context(handle)
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
		ctx, target, level, internal_format, width, height, border, format, type_, ptr, len,
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
