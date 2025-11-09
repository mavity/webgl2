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

// Higher-level resource management that lives entirely in WASM.
// Textures and framebuffers are managed here; JS will call these
// exports to create textures, upload pixels (by writing into linear memory
// and passing a pointer), attach textures to framebuffers, and read pixels
// back into JS memory.

static mut TEXTURES: Option<Vec<Option<Vec<u8>>>> = None;
static mut TEX_W: Option<Vec<u32>> = None;
static mut TEX_H: Option<Vec<u32>> = None;

static mut FRAMEBUFFERS: Option<Vec<Option<u32>>> = None; // attachment texture id
static mut BOUND_FRAMEBUFFER: Option<u32> = None;
static mut BOUND_TEXTURE: Option<u32> = None;

#[no_mangle]
pub extern "C" fn wasm_init_context(_w: u32, _h: u32) {
	unsafe {
		if TEXTURES.is_none() {
			TEXTURES = Some(Vec::new());
		}
		if TEX_W.is_none() {
			TEX_W = Some(Vec::new());
		}
		if TEX_H.is_none() {
			TEX_H = Some(Vec::new());
		}
		if FRAMEBUFFERS.is_none() {
			FRAMEBUFFERS = Some(Vec::new());
		}
		BOUND_FRAMEBUFFER = None;
	}
}

#[no_mangle]
pub extern "C" fn wasm_create_texture() -> u32 {
	unsafe {
		let texs = TEXTURES.as_mut().expect("textures init");
		let wv = TEX_W.as_mut().expect("tex_w init");
		let hv = TEX_H.as_mut().expect("tex_h init");
		texs.push(Some(Vec::new()));
		wv.push(0);
		hv.push(0);
		(texs.len() - 1) as u32
	}
}

#[no_mangle]
pub unsafe extern "C" fn wasm_tex_image_2d(tex: u32, w: u32, h: u32, ptr: u32) {
	let idx = tex as usize;
	// Special-case: if tex == u32::MAX, use currently bound texture
	let mut use_idx = None;
	if tex == core::u32::MAX {
		if let Some(bt) = BOUND_TEXTURE {
			use_idx = Some(bt as usize);
		}
	} else {
		use_idx = Some(idx);
	}
	let size = (w as usize).saturating_mul(h as usize).saturating_mul(4);
	let src = core::slice::from_raw_parts(ptr as *const u8, size);
	if let Some(ti) = use_idx {
		if let (Some(ref mut texs), Some(ref mut wv), Some(ref mut hv)) = (TEXTURES.as_mut(), TEX_W.as_mut(), TEX_H.as_mut()) {
			if ti < texs.len() {
				texs[ti] = Some(src.to_vec());
				wv[ti] = w;
				hv[ti] = h;
			}
		}
	}
}

#[no_mangle]
pub extern "C" fn wasm_bind_texture(tex: u32) {
	unsafe {
		if tex == core::u32::MAX {
			BOUND_TEXTURE = None;
		} else {
			BOUND_TEXTURE = Some(tex);
		}
	}
}

#[no_mangle]
pub extern "C" fn wasm_create_framebuffer() -> u32 {
	unsafe {
		let fbs = FRAMEBUFFERS.as_mut().expect("fb init");
		fbs.push(None);
		(fbs.len() - 1) as u32
	}
}

#[no_mangle]
pub extern "C" fn wasm_bind_framebuffer(fb: u32) {
	unsafe {
		BOUND_FRAMEBUFFER = Some(fb);
	}
}

#[no_mangle]
pub extern "C" fn wasm_framebuffer_texture2d(fb: u32, tex: u32) {
	unsafe {
		if let Some(ref mut fbs) = FRAMEBUFFERS {
			let idx = fb as usize;
			if idx < fbs.len() {
				fbs[idx] = Some(tex);
			}
		}
	}
}

#[no_mangle]
pub unsafe extern "C" fn wasm_read_pixels(x: u32, y: u32, w: u32, h: u32, out_ptr: u32) {
	// Read from the currently bound framebuffer's color attachment (texture)
	// and write bytes into linear memory at out_ptr in RGBA u8 order.
	let out = out_ptr as *mut u8;
	let width = w as usize;
	let height = h as usize;
	let mut dst_off = 0usize;
	if let Some(bound) = BOUND_FRAMEBUFFER {
		let fb_idx = bound as usize;
		if let Some(ref fbs) = FRAMEBUFFERS {
			if fb_idx < fbs.len() {
				if let Some(tex_id) = fbs[fb_idx] {
					let tidx = tex_id as usize;
					if let (Some(ref texs), Some(ref wv), Some(ref hv)) = (TEXTURES.as_ref(), TEX_W.as_ref(), TEX_H.as_ref()) {
						if tidx < texs.len() {
							if let Some(ref data) = texs[tidx] {
								let tw = wv[tidx] as usize;
								let th = hv[tidx] as usize;
								for row in 0..height {
									for col in 0..width {
										let sx = x as usize + col;
										let sy = y as usize + row;
										if sx < tw && sy < th {
											let sidx = (sy * tw + sx) * 4;
											let r = data[sidx];
											let g = data[sidx + 1];
											let b = data[sidx + 2];
											let a = data[sidx + 3];
											let p = out.add(dst_off);
											core::ptr::write(p, r);
											core::ptr::write(p.add(1), g);
											core::ptr::write(p.add(2), b);
											core::ptr::write(p.add(3), a);
										} else {
											let p = out.add(dst_off);
											core::ptr::write(p, 0);
											core::ptr::write(p.add(1), 0);
											core::ptr::write(p.add(2), 0);
											core::ptr::write(p.add(3), 0);
										}
										dst_off += 4;
									}
								}
								return;
							}
						}
					}
				}
			}
		}
	}
	// Fallback: zero out the output
	for i in 0..(width * height * 4) {
		core::ptr::write(out.add(i), 0u8);
	}
}
