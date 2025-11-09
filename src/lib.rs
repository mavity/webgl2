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

// Re-export commonly used types
pub use naga_wasm_backend::{WasmBackend, WasmBackendConfig, WasmModule, BackendError};
#[cfg(feature = "cli")]
pub use wasm_gl_emu::ShaderRuntime;
pub use wasm_gl_emu::RuntimeError;
pub use glsl_introspection::ResourceManifest;
pub use js_codegen::generate_harness;

// Small framebuffer helpers exported for the JS facade to exercise. These
// provide a tiny pixel buffer inside the wasm module that the JS side can
// write to and read from using only the public `webGL2` API (no hidden
// channels). Keep implementations minimal and unsafe where needed since this
// is a small test shim.

use std::vec::Vec;

static mut FB_BUF: Option<Vec<u8>> = None;
static mut FB_W: u32 = 0;
static mut FB_H: u32 = 0;

#[no_mangle]
pub extern "C" fn wasm_fb_init(w: u32, h: u32) {
	unsafe {
		let size = (w as usize).saturating_mul(h as usize).saturating_mul(4);
		FB_BUF = Some(vec![0u8; size]);
		FB_W = w;
		FB_H = h;
	}
}

#[no_mangle]
pub extern "C" fn wasm_set_pixel(x: u32, y: u32, r: u32, g: u32, b: u32, a: u32) {
	unsafe {
		if let Some(ref mut fb) = FB_BUF {
			if x < FB_W && y < FB_H {
				let idx = ((y * FB_W + x) * 4) as usize;
				if idx + 3 < fb.len() {
					fb[idx] = r as u8;
					fb[idx + 1] = g as u8;
					fb[idx + 2] = b as u8;
					fb[idx + 3] = a as u8;
				}
			}
		}
	}
}

#[no_mangle]
pub extern "C" fn wasm_get_pixel(x: u32, y: u32) -> u32 {
	unsafe {
		if let Some(ref fb) = FB_BUF {
			if x < FB_W && y < FB_H {
				let idx = ((y * FB_W + x) * 4) as usize;
				if idx + 3 < fb.len() {
					let r = fb[idx] as u32;
					let g = fb[idx + 1] as u32;
					let b = fb[idx + 2] as u32;
					let a = fb[idx + 3] as u32;
					return (r << 24) | (g << 16) | (b << 8) | a;
				}
			}
		}
		0
	}
}
