//! WebGL2 Context and Resource Management
//!
//! This module implements the core plumbing for the Rust-owned WebGL2 context,
//! following the design in docs/1.1.1-webgl2-prototype.md:
//!
//! - Single-threaded global registry (OnceCell + RefCell<HashMap>)
//! - Handle-based resource lifecycle (Context, Textures, Framebuffers)
//! - errno-based error reporting + last_error string buffer
//! - Memory allocation (wasm_alloc / wasm_free)

use std::cell::RefCell;
use std::collections::HashMap;
use std::alloc::{alloc, dealloc, Layout};

// Errno constants (must match JS constants if exposed)
pub const ERR_OK: u32 = 0;
pub const ERR_INVALID_HANDLE: u32 = 1;
pub const ERR_OOM: u32 = 2;
pub const ERR_INVALID_ARGS: u32 = 3;
pub const ERR_NOT_IMPLEMENTED: u32 = 4;
pub const ERR_GL: u32 = 5;
pub const ERR_INTERNAL: u32 = 6;

// Handle constants
const INVALID_HANDLE: u32 = 0;
const FIRST_HANDLE: u32 = 1;

// Last error buffer (thread-local would be better, but we're single-threaded WASM)
thread_local! {
    static LAST_ERROR: RefCell<String> = RefCell::new(String::new());
}

/// A WebGL2 texture resource
#[derive(Clone)]
struct Texture {
    width: u32,
    height: u32,
    data: Vec<u8>, // RGBA u8
}

/// A WebGL2 framebuffer resource (stores attachment texture ID)
#[derive(Clone)]
struct Framebuffer {
    color_attachment: Option<u32>, // texture handle
}

/// Per-context state
pub struct Context {
    textures: HashMap<u32, Texture>,
    framebuffers: HashMap<u32, Framebuffer>,
    next_texture_handle: u32,
    next_framebuffer_handle: u32,
    bound_texture: Option<u32>,
    bound_framebuffer: Option<u32>,
}

impl Context {
    fn new() -> Self {
        Context {
            textures: HashMap::new(),
            framebuffers: HashMap::new(),
            next_texture_handle: FIRST_HANDLE,
            next_framebuffer_handle: FIRST_HANDLE,
            bound_texture: None,
            bound_framebuffer: None,
        }
    }

    fn allocate_texture_handle(&mut self) -> u32 {
        let h = self.next_texture_handle;
        self.next_texture_handle = self.next_texture_handle.saturating_add(1);
        // Avoid handle 0 (reserved as invalid)
        if self.next_texture_handle == 0 {
            self.next_texture_handle = FIRST_HANDLE;
        }
        h
    }

    fn allocate_framebuffer_handle(&mut self) -> u32 {
        let h = self.next_framebuffer_handle;
        self.next_framebuffer_handle = self.next_framebuffer_handle.saturating_add(1);
        // Avoid handle 0
        if self.next_framebuffer_handle == 0 {
            self.next_framebuffer_handle = FIRST_HANDLE;
        }
        h
    }
}

/// Global registry: handle -> Context
///
/// Since WASM is single-threaded, we use a custom wrapper that bypasses Sync checking.
/// This is safe because WASM will never have multiple threads.
struct SyncRefCell<T>(RefCell<T>);

// SAFETY: WASM is single-threaded, so RefCell is safe to share across "threads"
// (there are none in practice).
unsafe impl<T> Sync for SyncRefCell<T> {}

fn get_registry() -> &'static RefCell<Registry> {
    static REGISTRY: std::sync::OnceLock<SyncRefCell<Registry>> = std::sync::OnceLock::new();
    &REGISTRY
        .get_or_init(|| {
            SyncRefCell(RefCell::new(Registry {
                contexts: HashMap::new(),
                next_context_handle: FIRST_HANDLE,
                allocations: HashMap::new(),
            }))
        })
        .0
}

struct Registry {
    contexts: HashMap<u32, Context>,
    next_context_handle: u32,
    /// Track allocations created via `wasm_alloc`: ptr -> size
    allocations: HashMap<u32, u32>,
}

impl Registry {
    fn allocate_context_handle(&mut self) -> u32 {
        let h = self.next_context_handle;
        self.next_context_handle = self.next_context_handle.saturating_add(1);
        if self.next_context_handle == 0 {
            self.next_context_handle = FIRST_HANDLE;
        }
        h
    }
}

// ============================================================================
// Public API (exported to WASM)
// ============================================================================

/// Set last error message (internal helper)
pub fn set_last_error(msg: &str) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = msg.to_string();
    });
}

/// Get pointer to last error string (UTF-8)
pub fn last_error_ptr() -> *const u8 {
    LAST_ERROR.with(|e| {
        let s = e.borrow();
        s.as_ptr()
    })
}

/// Get length of last error string
pub fn last_error_len() -> u32 {
    LAST_ERROR.with(|e| {
        e.borrow().len() as u32
    })
}

/// Clear last error
fn clear_last_error() {
    LAST_ERROR.with(|e| {
        e.borrow_mut().clear();
    });
}

// ============================================================================
// Context Lifecycle
// ============================================================================

/// Create a new WebGL2 context and return its handle.
/// Returns 0 on failure (sets last_error).
pub fn create_context() -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx = Context::new();
    let handle = reg.allocate_context_handle();
    reg.contexts.insert(handle, ctx);
    handle
}

/// Destroy a context by handle, freeing all its resources.
/// Returns errno (0 on success).
pub fn destroy_context(handle: u32) -> u32 {
    clear_last_error();
    if handle == INVALID_HANDLE {
        set_last_error("invalid context handle");
        return ERR_INVALID_HANDLE;
    }
    let mut reg = get_registry().borrow_mut();
    if reg.contexts.remove(&handle).is_none() {
        set_last_error("context not found");
        return ERR_INVALID_HANDLE;
    }
    ERR_OK
}

// ============================================================================
// Memory Allocation
// ============================================================================

/// Allocate memory within WASM linear memory.
/// Returns pointer (0 on failure).
pub fn wasm_alloc(size: u32) -> u32 {
    clear_last_error();
    if size == 0 {
        return 0; // Valid: allocating 0 bytes is OK but we return 0 for simplicity
    }
    let layout = match Layout::from_size_align(size as usize, 8) {
        Ok(l) => l,
        Err(_) => {
            set_last_error("allocation layout error");
            return 0;
        }
    };
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        set_last_error("out of memory");
        return 0;
    }
    let ptr_u32 = ptr as u32;
    // Record allocation size so wasm_free can deallocate later.
    {
        let mut reg = get_registry().borrow_mut();
        reg.allocations.insert(ptr_u32, size);
    }
    ptr_u32
}

/// Free memory allocated by wasm_alloc.
/// Returns errno (0 on success).
pub fn wasm_free(ptr: u32) -> u32 {
    clear_last_error();
    if ptr == 0 {
        // Freeing null is a no-op
        return ERR_OK;
    }
    // Look up allocation size
    let mut reg = get_registry().borrow_mut();
    let size = match reg.allocations.remove(&ptr) {
        Some(s) => s,
        None => {
            set_last_error("invalid or unknown allocation");
            return ERR_INVALID_ARGS;
        }
    };

    let layout = match Layout::from_size_align(size as usize, 8) {
        Ok(l) => l,
        Err(_) => {
            set_last_error("invalid allocation layout");
            return ERR_INTERNAL;
        }
    };
    unsafe { dealloc(ptr as *mut u8, layout) };
    ERR_OK
}

// ============================================================================
// Texture Operations
// ============================================================================

/// Create a texture in the given context.
/// Returns texture handle (0 on failure).
pub fn ctx_create_texture(ctx: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return 0;
        }
    };
    let tex_id = ctx.allocate_texture_handle();
    ctx.textures.insert(tex_id, Texture {
        width: 0,
        height: 0,
        data: Vec::new(),
    });
    tex_id
}

/// Delete a texture from the given context.
/// Returns errno.
pub fn ctx_delete_texture(ctx: u32, tex: u32) -> u32 {
    clear_last_error();
    if tex == INVALID_HANDLE {
        set_last_error("invalid texture handle");
        return ERR_INVALID_HANDLE;
    }
    let mut reg = get_registry().borrow_mut();
    let ctx = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    if ctx.textures.remove(&tex).is_none() {
        set_last_error("texture not found");
        return ERR_INVALID_HANDLE;
    }
    // If this was the bound texture, unbind it
    if ctx.bound_texture == Some(tex) {
        ctx.bound_texture = None;
    }
    ERR_OK
}

/// Bind a texture in the given context.
/// Returns errno.
pub fn ctx_bind_texture(ctx: u32, _target: u32, tex: u32) -> u32 {
    clear_last_error();
    if tex != INVALID_HANDLE && tex != 0 {
        let reg = get_registry().borrow();
        let ctx_obj = match reg.contexts.get(&ctx) {
            Some(c) => c,
            None => {
                set_last_error("invalid context handle");
                return ERR_INVALID_HANDLE;
            }
        };
        if !ctx_obj.textures.contains_key(&tex) {
            set_last_error("texture not found");
            return ERR_INVALID_HANDLE;
        }
    }
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    if tex == 0 {
        ctx_obj.bound_texture = None;
    } else {
        ctx_obj.bound_texture = Some(tex);
    }
    ERR_OK
}

/// Upload pixel data to a texture.
/// ptr and len point to RGBA u8 pixel data in WASM linear memory.
/// Returns errno.
pub fn ctx_tex_image_2d(
    ctx: u32,
    _target: u32,
    _level: i32,
    _internal_format: i32,
    width: u32,
    height: u32,
    _border: i32,
    _format: i32,
    _type_: i32,
    ptr: u32,
    len: u32,
) -> u32 {
    clear_last_error();
    
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    // Determine which texture to write to (bound or error)
    let tex_handle = match ctx_obj.bound_texture {
        Some(h) => h,
        None => {
            set_last_error("no texture bound");
            return ERR_INVALID_ARGS;
        }
    };

    // Validate dimensions
    let expected_size = (width as u64)
        .saturating_mul(height as u64)
        .saturating_mul(4)
        .saturating_mul(1); // 4 bytes per RGBA pixel
    if len as u64 != expected_size {
        set_last_error("pixel data size mismatch");
        return ERR_INVALID_ARGS;
    }

    // Copy pixel data from WASM linear memory
    let src_slice = unsafe {
        std::slice::from_raw_parts(ptr as *const u8, len as usize)
    };
    let pixel_data = src_slice.to_vec();

    // Store texture data
    if let Some(tex) = ctx_obj.textures.get_mut(&tex_handle) {
        tex.width = width;
        tex.height = height;
        tex.data = pixel_data;
        ERR_OK
    } else {
        set_last_error("texture not found");
        ERR_INVALID_HANDLE
    }
}

// ============================================================================
// Framebuffer Operations
// ============================================================================

/// Create a framebuffer in the given context.
/// Returns framebuffer handle (0 on failure).
pub fn ctx_create_framebuffer(ctx: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return 0;
        }
    };
    let fb_id = ctx_obj.allocate_framebuffer_handle();
    ctx_obj.framebuffers.insert(fb_id, Framebuffer {
        color_attachment: None,
    });
    fb_id
}

/// Delete a framebuffer from the given context.
/// Returns errno.
pub fn ctx_delete_framebuffer(ctx: u32, fb: u32) -> u32 {
    clear_last_error();
    if fb == INVALID_HANDLE {
        set_last_error("invalid framebuffer handle");
        return ERR_INVALID_HANDLE;
    }
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    if ctx_obj.framebuffers.remove(&fb).is_none() {
        set_last_error("framebuffer not found");
        return ERR_INVALID_HANDLE;
    }
    // If this was the bound framebuffer, unbind it
    if ctx_obj.bound_framebuffer == Some(fb) {
        ctx_obj.bound_framebuffer = None;
    }
    ERR_OK
}

/// Bind a framebuffer in the given context.
/// Returns errno.
pub fn ctx_bind_framebuffer(ctx: u32, _target: u32, fb: u32) -> u32 {
    clear_last_error();
    if fb != INVALID_HANDLE && fb != 0 {
        let reg = get_registry().borrow();
        let ctx_obj = match reg.contexts.get(&ctx) {
            Some(c) => c,
            None => {
                set_last_error("invalid context handle");
                return ERR_INVALID_HANDLE;
            }
        };
        if !ctx_obj.framebuffers.contains_key(&fb) {
            set_last_error("framebuffer not found");
            return ERR_INVALID_HANDLE;
        }
    }
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    if fb == 0 {
        ctx_obj.bound_framebuffer = None;
    } else {
        ctx_obj.bound_framebuffer = Some(fb);
    }
    ERR_OK
}

/// Attach a texture to a framebuffer.
/// Returns errno.
pub fn ctx_framebuffer_texture2d(
    ctx: u32,
    _target: u32,
    _attachment: u32,
    _textarget: u32,
    tex: u32,
    _level: i32,
) -> u32 {
    clear_last_error();

    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    let fb_handle = match ctx_obj.bound_framebuffer {
        Some(h) => h,
        None => {
            set_last_error("no framebuffer bound");
            return ERR_INVALID_ARGS;
        }
    };

    // Validate texture exists
    if tex != 0 && !ctx_obj.textures.contains_key(&tex) {
        set_last_error("texture not found");
        return ERR_INVALID_HANDLE;
    }

    // Attach texture to framebuffer
    if let Some(fb) = ctx_obj.framebuffers.get_mut(&fb_handle) {
        fb.color_attachment = if tex == 0 { None } else { Some(tex) };
        ERR_OK
    } else {
        set_last_error("framebuffer not found");
        ERR_INVALID_HANDLE
    }
}

// ============================================================================
// Pixel Read
// ============================================================================

/// Read pixels from the currently bound framebuffer's color attachment.
/// Writes RGBA u8 data to dest_ptr in WASM linear memory.
/// Returns errno.
pub fn ctx_read_pixels(
    ctx: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    _format: u32,
    _type_: u32,
    dest_ptr: u32,
    dest_len: u32,
) -> u32 {
    clear_last_error();

    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    // Get the currently bound framebuffer
    let fb_handle = match ctx_obj.bound_framebuffer {
        Some(h) => h,
        None => {
            set_last_error("no framebuffer bound");
            return ERR_INVALID_ARGS;
        }
    };

    // Get framebuffer and its attached texture
    let fb = match ctx_obj.framebuffers.get(&fb_handle) {
        Some(f) => f,
        None => {
            set_last_error("framebuffer not found");
            return ERR_INVALID_HANDLE;
        }
    };

    let tex_handle = match fb.color_attachment {
        Some(h) => h,
        None => {
            set_last_error("framebuffer has no color attachment");
            return ERR_INVALID_ARGS;
        }
    };

    // Get texture data
    let tex = match ctx_obj.textures.get(&tex_handle) {
        Some(t) => t,
        None => {
            set_last_error("attached texture not found");
            return ERR_INVALID_HANDLE;
        }
    };

    // Verify output buffer size
    let expected_size = (width as u64)
        .saturating_mul(height as u64)
        .saturating_mul(4);
    if dest_len as u64 != expected_size {
        set_last_error("output buffer size mismatch");
        return ERR_INVALID_ARGS;
    }

    // Read pixels and write to destination
    let dest_slice = unsafe {
        std::slice::from_raw_parts_mut(dest_ptr as *mut u8, dest_len as usize)
    };

    let mut dst_off = 0;
    for row in 0..height {
        for col in 0..width {
            let sx = x as u32 + col;
            let sy = y as u32 + row;

            if sx < tex.width && sy < tex.height {
                let src_idx = ((sy * tex.width + sx) * 4) as usize;
                if src_idx + 3 < tex.data.len() {
                    dest_slice[dst_off] = tex.data[src_idx];
                    dest_slice[dst_off + 1] = tex.data[src_idx + 1];
                    dest_slice[dst_off + 2] = tex.data[src_idx + 2];
                    dest_slice[dst_off + 3] = tex.data[src_idx + 3];
                } else {
                    dest_slice[dst_off] = 0;
                    dest_slice[dst_off + 1] = 0;
                    dest_slice[dst_off + 2] = 0;
                    dest_slice[dst_off + 3] = 0;
                }
            } else {
                // Out of bounds: write transparent black
                dest_slice[dst_off] = 0;
                dest_slice[dst_off + 1] = 0;
                dest_slice[dst_off + 2] = 0;
                dest_slice[dst_off + 3] = 0;
            }
            dst_off += 4;
        }
    }

    ERR_OK
}

// -----------------------------
// Unit tests for allocation APIs
// -----------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_free_roundtrip() {
        // allocate 128 bytes
        let ptr = wasm_alloc(128);
        assert!(ptr != 0, "wasm_alloc returned 0");

        // write into the allocation (safely)
        unsafe {
            let slice = std::slice::from_raw_parts_mut(ptr as *mut u8, 128);
            for i in 0..128 {
                slice[i] = (i & 0xff) as u8;
            }
        }

        // free should succeed
        let code = wasm_free(ptr);
        assert_eq!(code, ERR_OK, "wasm_free returned non-zero");
    }

    #[test]
    fn free_null_is_noop() {
        let code = wasm_free(0);
        assert_eq!(code, ERR_OK);
    }

    #[test]
    fn double_free_fails() {
        let ptr = wasm_alloc(32);
        assert!(ptr != 0);
        let first = wasm_free(ptr);
        assert_eq!(first, ERR_OK);
        // second free should return invalid args
        let second = wasm_free(ptr);
        assert_eq!(second, ERR_INVALID_ARGS);
    }
}
