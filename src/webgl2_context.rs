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

// GL Error constants
pub const GL_NO_ERROR: u32 = 0;
pub const GL_INVALID_ENUM: u32 = 0x0500;
pub const GL_INVALID_VALUE: u32 = 0x0501;
pub const GL_INVALID_OPERATION: u32 = 0x0502;
pub const GL_OUT_OF_MEMORY: u32 = 0x0505;

pub const GL_ARRAY_BUFFER: u32 = 0x8892;
pub const GL_ELEMENT_ARRAY_BUFFER: u32 = 0x8893;

pub const GL_COMPILE_STATUS: u32 = 0x8B81;
pub const GL_LINK_STATUS: u32 = 0x8B82;
pub const GL_SHADER_TYPE: u32 = 0x8B4F;
pub const GL_DELETE_STATUS: u32 = 0x8B80;
pub const GL_INFO_LOG_LENGTH: u32 = 0x8B84;
pub const GL_ATTACHED_SHADERS: u32 = 0x8B85;
pub const GL_ACTIVE_UNIFORMS: u32 = 0x8B86;
pub const GL_ACTIVE_ATTRIBUTES: u32 = 0x8B89;

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

/// A WebGL2 buffer resource
#[derive(Clone)]
struct Buffer {
    data: Vec<u8>,
    usage: u32,
}

/// A WebGL2 shader resource
#[derive(Clone)]
struct Shader {
    type_: u32,
    source: String,
    compiled: bool,
    info_log: String,
}

/// A WebGL2 program resource
#[derive(Clone)]
struct Program {
    attached_shaders: Vec<u32>,
    linked: bool,
    info_log: String,
    attributes: HashMap<String, i32>,
    uniforms: HashMap<String, i32>,
}

/// A WebGL2 vertex attribute
#[derive(Clone)]
struct VertexAttribute {
    enabled: bool,
    size: i32,
    type_: u32,
    normalized: bool,
    stride: i32,
    offset: u32,
    buffer: Option<u32>,
}

impl VertexAttribute {
    fn new() -> Self {
        VertexAttribute {
            enabled: false,
            size: 4,
            type_: 0x1406, // GL_FLOAT
            normalized: false,
            stride: 0,
            offset: 0,
            buffer: None,
        }
    }
}

/// Per-context state
pub struct Context {
    textures: HashMap<u32, Texture>,
    framebuffers: HashMap<u32, Framebuffer>,
    buffers: HashMap<u32, Buffer>,
    shaders: HashMap<u32, Shader>,
    programs: HashMap<u32, Program>,

    next_texture_handle: u32,
    next_framebuffer_handle: u32,
    next_buffer_handle: u32,
    next_shader_handle: u32,
    next_program_handle: u32,

    bound_texture: Option<u32>,
    bound_framebuffer: Option<u32>,
    bound_array_buffer: Option<u32>,
    bound_element_array_buffer: Option<u32>,
    current_program: Option<u32>,

    vertex_attributes: Vec<VertexAttribute>,

    // State
    clear_color: [f32; 4],
    viewport: (i32, i32, u32, u32),
    gl_error: u32,
}

impl Context {
    fn new() -> Self {
        Context {
            textures: HashMap::new(),
            framebuffers: HashMap::new(),
            buffers: HashMap::new(),
            shaders: HashMap::new(),
            programs: HashMap::new(),

            next_texture_handle: FIRST_HANDLE,
            next_framebuffer_handle: FIRST_HANDLE,
            next_buffer_handle: FIRST_HANDLE,
            next_shader_handle: FIRST_HANDLE,
            next_program_handle: FIRST_HANDLE,

            bound_texture: None,
            bound_framebuffer: None,
            bound_array_buffer: None,
            bound_element_array_buffer: None,
            current_program: None,

            vertex_attributes: vec![VertexAttribute::new(); 16],

            clear_color: [0.0, 0.0, 0.0, 0.0],
            viewport: (0, 0, 0, 0),
            gl_error: GL_NO_ERROR,
        }
    }

    fn allocate_texture_handle(&mut self) -> u32 {
        let h = self.next_texture_handle;
        self.next_texture_handle = self.next_texture_handle.saturating_add(1);
        if self.next_texture_handle == 0 {
            self.next_texture_handle = FIRST_HANDLE;
        }
        h
    }

    fn allocate_framebuffer_handle(&mut self) -> u32 {
        let h = self.next_framebuffer_handle;
        self.next_framebuffer_handle = self.next_framebuffer_handle.saturating_add(1);
        if self.next_framebuffer_handle == 0 {
            self.next_framebuffer_handle = FIRST_HANDLE;
        }
        h
    }

    fn allocate_buffer_handle(&mut self) -> u32 {
        let h = self.next_buffer_handle;
        self.next_buffer_handle = self.next_buffer_handle.saturating_add(1);
        if self.next_buffer_handle == 0 {
            self.next_buffer_handle = FIRST_HANDLE;
        }
        h
    }

    fn allocate_shader_handle(&mut self) -> u32 {
        let h = self.next_shader_handle;
        self.next_shader_handle = self.next_shader_handle.saturating_add(1);
        if self.next_shader_handle == 0 {
            self.next_shader_handle = FIRST_HANDLE;
        }
        h
    }

    fn allocate_program_handle(&mut self) -> u32 {
        let h = self.next_program_handle;
        self.next_program_handle = self.next_program_handle.saturating_add(1);
        if self.next_program_handle == 0 {
            self.next_program_handle = FIRST_HANDLE;
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
// State Management
// ============================================================================

/// Set the clear color.
pub fn ctx_clear_color(ctx: u32, r: f32, g: f32, b: f32, a: f32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    ctx_obj.clear_color = [r, g, b, a];
    ERR_OK
}

/// Clear buffers to preset values.
pub fn ctx_clear(ctx: u32, _mask: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let _ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    // In a real implementation, this would clear the bound framebuffer.
    // For now, it's a no-op in the mock.
    ERR_OK
}

/// Set the viewport.
pub fn ctx_viewport(ctx: u32, x: i32, y: i32, width: u32, height: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    ctx_obj.viewport = (x, y, width, height);
    ERR_OK
}

/// Get the last GL error.
pub fn ctx_get_error(ctx: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            // If context is invalid, we can't really return a GL error for it.
            // But WebGL says getError returns NO_ERROR if no context? 
            // Actually, it's a method on the context.
            return GL_NO_ERROR;
        }
    };
    let err = ctx_obj.gl_error;
    ctx_obj.gl_error = GL_NO_ERROR;
    err
}

// ============================================================================
// Buffer Operations
// ============================================================================

/// Create a buffer in the given context.
pub fn ctx_create_buffer(ctx: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return 0;
        }
    };
    let buf_id = ctx_obj.allocate_buffer_handle();
    ctx_obj.buffers.insert(buf_id, Buffer {
        data: Vec::new(),
        usage: 0,
    });
    buf_id
}

/// Delete a buffer.
pub fn ctx_delete_buffer(ctx: u32, buf: u32) -> u32 {
    clear_last_error();
    if buf == INVALID_HANDLE {
        return ERR_OK; // Deleting 0 is a no-op
    }
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    ctx_obj.buffers.remove(&buf);
    if ctx_obj.bound_array_buffer == Some(buf) {
        ctx_obj.bound_array_buffer = None;
    }
    if ctx_obj.bound_element_array_buffer == Some(buf) {
        ctx_obj.bound_element_array_buffer = None;
    }
    ERR_OK
}

/// Bind a buffer to a target.
pub fn ctx_bind_buffer(ctx: u32, target: u32, buf: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if buf != 0 && !ctx_obj.buffers.contains_key(&buf) {
        set_last_error("buffer not found");
        return ERR_INVALID_HANDLE;
    }

    match target {
        GL_ARRAY_BUFFER => ctx_obj.bound_array_buffer = if buf == 0 { None } else { Some(buf) },
        GL_ELEMENT_ARRAY_BUFFER => ctx_obj.bound_element_array_buffer = if buf == 0 { None } else { Some(buf) },
        _ => {
            set_last_error("invalid buffer target");
            return ERR_INVALID_ARGS;
        }
    }
    ERR_OK
}

/// Upload data to the bound buffer.
pub fn ctx_buffer_data(ctx: u32, target: u32, ptr: u32, len: u32, usage: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    let buf_handle = match target {
        GL_ARRAY_BUFFER => ctx_obj.bound_array_buffer,
        GL_ELEMENT_ARRAY_BUFFER => ctx_obj.bound_element_array_buffer,
        _ => {
            set_last_error("invalid buffer target");
            return ERR_INVALID_ARGS;
        }
    };

    let buf_handle = match buf_handle {
        Some(h) => h,
        None => {
            set_last_error("no buffer bound to target");
            return ERR_INVALID_ARGS;
        }
    };

    let src_slice = unsafe {
        std::slice::from_raw_parts(ptr as *const u8, len as usize)
    };

    if let Some(buf) = ctx_obj.buffers.get_mut(&buf_handle) {
        buf.data = src_slice.to_vec();
        buf.usage = usage;
        ERR_OK
    } else {
        set_last_error("buffer not found");
        ERR_INVALID_HANDLE
    }
}

// ============================================================================
// Shader and Program Operations
// ============================================================================

/// Create a shader.
pub fn ctx_create_shader(ctx: u32, type_: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return 0;
        }
    };
    let shader_id = ctx_obj.allocate_shader_handle();
    ctx_obj.shaders.insert(shader_id, Shader {
        type_,
        source: String::new(),
        compiled: false,
        info_log: String::new(),
    });
    shader_id
}

/// Delete a shader.
pub fn ctx_delete_shader(ctx: u32, shader: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    ctx_obj.shaders.remove(&shader);
    ERR_OK
}

/// Set shader source.
pub fn ctx_shader_source(ctx: u32, shader: u32, source_ptr: u32, source_len: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    let source_slice = unsafe {
        std::slice::from_raw_parts(source_ptr as *const u8, source_len as usize)
    };
    let source = String::from_utf8_lossy(source_slice).into_owned();

    if let Some(s) = ctx_obj.shaders.get_mut(&shader) {
        s.source = source;
        ERR_OK
    } else {
        set_last_error("shader not found");
        ERR_INVALID_HANDLE
    }
}

/// Compile a shader.
pub fn ctx_compile_shader(ctx: u32, shader: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    
    if let Some(s) = ctx_obj.shaders.get_mut(&shader) {
        s.compiled = true;
        s.info_log = "Shader compiled successfully (mock)".to_string();
        ERR_OK
    } else {
        set_last_error("shader not found");
        ERR_INVALID_HANDLE
    }
}

/// Get shader parameter.
pub fn ctx_get_shader_parameter(ctx: u32, shader: u32, pname: u32) -> i32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return 0,
    };

    if let Some(s) = ctx_obj.shaders.get(&shader) {
        match pname {
            GL_SHADER_TYPE => s.type_ as i32,
            GL_COMPILE_STATUS => if s.compiled { 1 } else { 0 },
            GL_DELETE_STATUS => 0, // Not implemented
            _ => 0,
        }
    } else {
        0
    }
}

/// Get shader info log.
pub fn ctx_get_shader_info_log(ctx: u32, shader: u32, dest_ptr: u32, dest_len: u32) -> u32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if let Some(s) = ctx_obj.shaders.get(&shader) {
        let bytes = s.info_log.as_bytes();
        let len = std::cmp::min(bytes.len(), dest_len as usize);
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), dest_ptr as *mut u8, len);
        }
        len as u32
    } else {
        0
    }
}

/// Create a program.
pub fn ctx_create_program(ctx: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return 0;
        }
    };
    let program_id = ctx_obj.allocate_program_handle();
    ctx_obj.programs.insert(program_id, Program {
        attached_shaders: Vec::new(),
        linked: false,
        info_log: String::new(),
        attributes: HashMap::new(),
        uniforms: HashMap::new(),
    });
    program_id
}

/// Delete a program.
pub fn ctx_delete_program(ctx: u32, program: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    ctx_obj.programs.remove(&program);
    if ctx_obj.current_program == Some(program) {
        ctx_obj.current_program = None;
    }
    ERR_OK
}

/// Attach a shader to a program.
pub fn ctx_attach_shader(ctx: u32, program: u32, shader: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if !ctx_obj.shaders.contains_key(&shader) {
        set_last_error("shader not found");
        return ERR_INVALID_HANDLE;
    }

    if let Some(p) = ctx_obj.programs.get_mut(&program) {
        if !p.attached_shaders.contains(&shader) {
            p.attached_shaders.push(shader);
        }
        ERR_OK
    } else {
        set_last_error("program not found");
        ERR_INVALID_HANDLE
    }
}

/// Link a program.
pub fn ctx_link_program(ctx: u32, program: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    
    if let Some(p) = ctx_obj.programs.get_mut(&program) {
        p.linked = true;
        p.info_log = "Program linked successfully (mock)".to_string();
        ERR_OK
    } else {
        set_last_error("program not found");
        ERR_INVALID_HANDLE
    }
}

/// Get program parameter.
pub fn ctx_get_program_parameter(ctx: u32, program: u32, pname: u32) -> i32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return 0,
    };

    if let Some(p) = ctx_obj.programs.get(&program) {
        match pname {
            GL_LINK_STATUS => if p.linked { 1 } else { 0 },
            GL_ATTACHED_SHADERS => p.attached_shaders.len() as i32,
            GL_DELETE_STATUS => 0,
            _ => 0,
        }
    } else {
        0
    }
}

/// Get program info log.
pub fn ctx_get_program_info_log(ctx: u32, program: u32, dest_ptr: u32, dest_len: u32) -> u32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if let Some(p) = ctx_obj.programs.get(&program) {
        let bytes = p.info_log.as_bytes();
        let len = std::cmp::min(bytes.len(), dest_len as usize);
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), dest_ptr as *mut u8, len);
        }
        len as u32
    } else {
        0
    }
}

/// Get attribute location.
pub fn ctx_get_attrib_location(ctx: u32, program: u32, name_ptr: u32, name_len: u32) -> i32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return -1,
    };

    let name_slice = unsafe {
        std::slice::from_raw_parts(name_ptr as *const u8, name_len as usize)
    };
    let name = String::from_utf8_lossy(name_slice);

    if let Some(p) = ctx_obj.programs.get(&program) {
        if let Some(&loc) = p.attributes.get(name.as_ref()) {
            loc
        } else {
            // In a real implementation, we'd look this up in the linked program.
            // For now, let's just return -1 if not explicitly bound.
            -1
        }
    } else {
        -1
    }
}

/// Bind attribute location.
pub fn ctx_bind_attrib_location(ctx: u32, program: u32, index: u32, name_ptr: u32, name_len: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    let name_slice = unsafe {
        std::slice::from_raw_parts(name_ptr as *const u8, name_len as usize)
    };
    let name = String::from_utf8_lossy(name_slice).into_owned();

    if let Some(p) = ctx_obj.programs.get_mut(&program) {
        p.attributes.insert(name, index as i32);
        ERR_OK
    } else {
        set_last_error("program not found");
        ERR_INVALID_HANDLE
    }
}

/// Get uniform location.
pub fn ctx_get_uniform_location(ctx: u32, program: u32, name_ptr: u32, name_len: u32) -> i32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return -1,
    };

    let name_slice = unsafe {
        std::slice::from_raw_parts(name_ptr as *const u8, name_len as usize)
    };
    let name = String::from_utf8_lossy(name_slice);

    if let Some(p) = ctx_obj.programs.get(&program) {
        if let Some(&loc) = p.uniforms.get(name.as_ref()) {
            loc
        } else {
            // In a real implementation, we'd look this up in the linked program.
            // For now, let's just return a mock location if it's not found.
            // We'll use a simple hash of the name as a mock location.
            let mut h = 0i32;
            for b in name.as_bytes() {
                h = h.wrapping_mul(31).wrapping_add(*b as i32);
            }
            h.abs()
        }
    } else {
        -1
    }
}

/// Set uniform 1f.
pub fn ctx_uniform1f(ctx: u32, _location: i32, _x: f32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let _ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };
    // In a real implementation, this would set the uniform value.
    ERR_OK
}

/// Set uniform 2f.
pub fn ctx_uniform2f(ctx: u32, _location: i32, _x: f32, _y: f32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let _ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };
    ERR_OK
}

/// Set uniform 3f.
pub fn ctx_uniform3f(ctx: u32, _location: i32, _x: f32, _y: f32, _z: f32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let _ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };
    ERR_OK
}

/// Set uniform 4f.
pub fn ctx_uniform4f(ctx: u32, _location: i32, _x: f32, _y: f32, _z: f32, _w: f32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let _ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };
    ERR_OK
}

/// Set uniform 1i.
pub fn ctx_uniform1i(ctx: u32, _location: i32, _x: i32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let _ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };
    ERR_OK
}

/// Set uniform matrix 4fv.
pub fn ctx_uniform_matrix_4fv(ctx: u32, _location: i32, _transpose: bool, _ptr: u32, _len: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let _ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };
    ERR_OK
}

/// Use a program.
pub fn ctx_use_program(ctx: u32, program: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if program != 0 && !ctx_obj.programs.contains_key(&program) {
        set_last_error("program not found");
        return ERR_INVALID_HANDLE;
    }

    ctx_obj.current_program = if program == 0 { None } else { Some(program) };
    ERR_OK
}

/// Enable vertex attribute array.
pub fn ctx_enable_vertex_attrib_array(ctx: u32, index: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if (index as usize) < ctx_obj.vertex_attributes.len() {
        ctx_obj.vertex_attributes[index as usize].enabled = true;
        ERR_OK
    } else {
        set_last_error("index out of range");
        ERR_INVALID_ARGS
    }
}

/// Disable vertex attribute array.
pub fn ctx_disable_vertex_attrib_array(ctx: u32, index: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if (index as usize) < ctx_obj.vertex_attributes.len() {
        ctx_obj.vertex_attributes[index as usize].enabled = false;
        ERR_OK
    } else {
        set_last_error("index out of range");
        ERR_INVALID_ARGS
    }
}

/// Vertex attribute pointer.
pub fn ctx_vertex_attrib_pointer(ctx: u32, index: u32, size: i32, type_: u32, normalized: bool, stride: i32, offset: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if (index as usize) < ctx_obj.vertex_attributes.len() {
        let attr = &mut ctx_obj.vertex_attributes[index as usize];
        attr.size = size;
        attr.type_ = type_;
        attr.normalized = normalized;
        attr.stride = stride;
        attr.offset = offset;
        attr.buffer = ctx_obj.bound_array_buffer;
        ERR_OK
    } else {
        set_last_error("index out of range");
        ERR_INVALID_ARGS
    }
}

/// Draw arrays.
pub fn ctx_draw_arrays(ctx: u32, mode: u32, first: i32, count: i32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let _ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };
    // In a real implementation, this would perform the draw call.
    ERR_OK
}

/// Draw elements.
pub fn ctx_draw_elements(ctx: u32, mode: u32, count: i32, type_: u32, offset: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let _ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };
    // In a real implementation, this would perform the draw call.
    ERR_OK
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
