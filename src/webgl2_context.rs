//! WebGL2 Context and Resource Management
//!
//! This module implements the core plumbing for the Rust-owned WebGL2 context,
//! following the design in docs/1.1.1-webgl2-prototype.md:
//!
//! - Single-threaded global registry (OnceCell + RefCell<HashMap>)
//! - Handle-based resource lifecycle (Context, Textures, Framebuffers)
//! - errno-based error reporting + last_error string buffer
//! - Memory allocation (wasm_alloc / wasm_free)

use std::alloc::{alloc, dealloc, Layout};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use naga::front::glsl::{Frontend, Options};
use naga::valid::{Capabilities, ValidationFlags, Validator};
use naga::{AddressSpace, Binding, ShaderStage};

use crate::naga_wasm_backend::{WasmBackend, WasmBackendConfig};

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

struct Vertex {
    pos: [f32; 4],
    varyings: Vec<u8>,
}

fn barycentric(p: (f32, f32), a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> (f32, f32, f32) {
    let area = (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0);
    if area.abs() < 1e-6 {
        return (-1.0, -1.0, -1.0);
    }
    let w0 = ((b.0 - p.0) * (c.1 - p.1) - (b.1 - p.1) * (c.0 - p.0)) / area;
    let w1 = ((c.0 - p.0) * (a.1 - p.1) - (c.1 - p.1) * (a.0 - p.0)) / area;
    let w2 = 1.0 - w0 - w1;
    (w0, w1, w2)
}

pub const GL_VIEWPORT: u32 = 0x0BA2;
pub const GL_COLOR_CLEAR_VALUE: u32 = 0x0C22;
pub const GL_BUFFER_SIZE: u32 = 0x8764;
pub const GL_COLOR_BUFFER_BIT: u32 = 0x00004000;

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
struct FramebufferObj {
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
    module: Option<Arc<naga::Module>>,
    info: Option<Arc<naga::valid::ModuleInfo>>,
}

/// A WebGL2 program resource
struct Program {
    attached_shaders: Vec<u32>,
    linked: bool,
    info_log: String,
    attributes: HashMap<String, i32>,
    uniforms: HashMap<String, i32>,
    vs_module: Option<Arc<naga::Module>>,
    fs_module: Option<Arc<naga::Module>>,
    vs_info: Option<Arc<naga::valid::ModuleInfo>>,
    fs_info: Option<Arc<naga::valid::ModuleInfo>>,
    vs_wasm: Option<Vec<u8>>,
    fs_wasm: Option<Vec<u8>>,
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
    default_value: [f32; 4],
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
            default_value: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

/// Per-context state
pub struct Context {
    textures: HashMap<u32, Texture>,
    framebuffers: HashMap<u32, FramebufferObj>,
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
    uniform_data: Vec<u8>,

    // Software rendering state
    pub default_framebuffer: crate::wasm_gl_emu::Framebuffer,
    pub rasterizer: crate::wasm_gl_emu::Rasterizer,

    // State
    clear_color: [f32; 4],
    viewport: (i32, i32, u32, u32),
    scissor_box: (i32, i32, u32, u32),
    scissor_test_enabled: bool,
    depth_test_enabled: bool,
    depth_func: u32,
    blend_enabled: bool,
    active_texture_unit: u32,
    texture_units: Vec<Option<u32>>,
    gl_error: u32,
    pub verbosity: u32, // 0: None, 1: Error, 2: Info, 3: Debug
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
            uniform_data: vec![0; 4096], // 4KB for uniforms

            default_framebuffer: crate::wasm_gl_emu::Framebuffer::new(640, 480),
            rasterizer: crate::wasm_gl_emu::Rasterizer::new(),

            clear_color: [0.0, 0.0, 0.0, 0.0],
            viewport: (0, 0, 640, 480),
            scissor_box: (0, 0, 640, 480),
            scissor_test_enabled: false,
            depth_test_enabled: false,
            depth_func: 0x0203, // GL_LEQUAL
            blend_enabled: false,
            active_texture_unit: 0,
            texture_units: vec![None; 16],
            gl_error: GL_NO_ERROR,
            verbosity: 2, // Default to Info
        }
    }

    fn log(&self, level: u32, s: &str) {
        if level <= self.verbosity {
            crate::js_log(level, s);
        }
    }

    fn log_static(verbosity: u32, level: u32, s: &str) {
        if level <= verbosity {
            crate::js_log(level, s);
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

    fn fetch_vertex_attributes(&self, vertex_id: u32, dest: &mut [f32]) {
        for (i, attr) in self.vertex_attributes.iter().enumerate() {
            let base_idx = i * 4;
            if !attr.enabled {
                dest[base_idx..base_idx + 4].copy_from_slice(&attr.default_value);
                continue;
            }

            if let Some(buffer_id) = attr.buffer {
                if let Some(buffer) = self.buffers.get(&buffer_id) {
                    let type_size = match attr.type_ {
                        0x1406 => 4, // GL_FLOAT
                        0x1400 => 1, // GL_BYTE
                        0x1401 => 1, // GL_UNSIGNED_BYTE
                        0x1402 => 2, // GL_SHORT
                        0x1403 => 2, // GL_UNSIGNED_SHORT
                        0x1404 => 4, // GL_INT
                        0x1405 => 4, // GL_UNSIGNED_INT
                        _ => 4,
                    };

                    let effective_stride = if attr.stride == 0 {
                        attr.size * type_size
                    } else {
                        attr.stride
                    };

                    let offset =
                        attr.offset as usize + (vertex_id as usize * effective_stride as usize);

                    for component in 0..4 {
                        if component < attr.size as usize {
                            let src_off = offset + component * type_size as usize;
                            if src_off + type_size as usize <= buffer.data.len() {
                                let val = match attr.type_ {
                                    0x1406 => f32::from_le_bytes(
                                        buffer.data[src_off..src_off + 4]
                                            .try_into()
                                            .unwrap_or([0; 4]),
                                    ),
                                    0x1401 => buffer.data[src_off] as f32 / 255.0, // GL_UNSIGNED_BYTE (normalized)
                                    _ => 0.0,
                                };
                                dest[base_idx + component] = val;
                            } else {
                                dest[base_idx + component] = 0.0;
                            }
                        } else {
                            // Fill remaining components with default (0,0,0,1)
                            dest[base_idx + component] = if component == 3 { 1.0 } else { 0.0 };
                        }
                    }
                } else {
                    dest[base_idx..base_idx + 4].copy_from_slice(&attr.default_value);
                }
            } else {
                dest[base_idx..base_idx + 4].copy_from_slice(&attr.default_value);
            }
        }
    }

    fn prepare_texture_metadata(&self, dest_ptr: u32) {
        for (i, tex_handle) in self.texture_units.iter().enumerate() {
            let offset = i * 32;
            if let Some(h) = tex_handle {
                if let Some(tex) = self.textures.get(h) {
                    unsafe {
                        let base = (dest_ptr + offset as u32) as *mut i32;
                        *base.offset(0) = tex.width as i32;
                        *base.offset(1) = tex.height as i32;
                        *base.offset(2) = tex.data.as_ptr() as i32;
                        // Rest is padding for now
                    }
                }
            }
        }
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
    LAST_ERROR.with(|e| e.borrow().len() as u32)
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
    ctx.textures.insert(
        tex_id,
        Texture {
            width: 0,
            height: 0,
            data: Vec::new(),
        },
    );
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
    let tex_val = if tex == 0 { None } else { Some(tex) };
    ctx_obj.bound_texture = tex_val;
    let unit = ctx_obj.active_texture_unit as usize;
    if unit < ctx_obj.texture_units.len() {
        ctx_obj.texture_units[unit] = tex_val;
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
    let src_slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
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
    ctx_obj.framebuffers.insert(
        fb_id,
        FramebufferObj {
            color_attachment: None,
        },
    );
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
pub fn ctx_clear(ctx: u32, mask: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if (mask & GL_COLOR_BUFFER_BIT) != 0 {
        let r = (ctx_obj.clear_color[0] * 255.0) as u8;
        let g = (ctx_obj.clear_color[1] * 255.0) as u8;
        let b = (ctx_obj.clear_color[2] * 255.0) as u8;
        let a = (ctx_obj.clear_color[3] * 255.0) as u8;

        if let Some(fb_handle) = ctx_obj.bound_framebuffer {
            if let Some(fb) = ctx_obj.framebuffers.get(&fb_handle) {
                if let Some(tex_handle) = fb.color_attachment {
                    if let Some(tex) = ctx_obj.textures.get_mut(&tex_handle) {
                        for i in (0..tex.data.len()).step_by(4) {
                            if i + 3 < tex.data.len() {
                                tex.data[i] = r;
                                tex.data[i + 1] = g;
                                tex.data[i + 2] = b;
                                tex.data[i + 3] = a;
                            }
                        }
                    }
                }
            }
        } else {
            // Clear default framebuffer
            for i in (0..ctx_obj.default_framebuffer.color.len()).step_by(4) {
                if i + 3 < ctx_obj.default_framebuffer.color.len() {
                    ctx_obj.default_framebuffer.color[i] = r;
                    ctx_obj.default_framebuffer.color[i + 1] = g;
                    ctx_obj.default_framebuffer.color[i + 2] = b;
                    ctx_obj.default_framebuffer.color[i + 3] = a;
                }
            }
        }
    }

    if (mask & 0x00000100) != 0 {
        // GL_DEPTH_BUFFER_BIT
        if ctx_obj.bound_framebuffer.is_none() {
            for d in ctx_obj.default_framebuffer.depth.iter_mut() {
                *d = 1.0; // Default clear depth
            }
        }
    }

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

pub fn ctx_scissor(ctx: u32, x: i32, y: i32, width: u32, height: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    ctx_obj.scissor_box = (x, y, width, height);
    ERR_OK
}

pub fn ctx_depth_func(ctx: u32, func: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    ctx_obj.depth_func = func;
    ERR_OK
}

pub fn ctx_active_texture(ctx: u32, texture: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    // texture is GL_TEXTURE0 + i
    if texture < 0x84C0 || texture > 0x84DF {
        set_last_error("invalid texture unit");
        return ERR_INVALID_ARGS;
    }
    ctx_obj.active_texture_unit = texture - 0x84C0;
    ERR_OK
}

pub fn ctx_enable(ctx: u32, cap: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    match cap {
        0x0C11 /* SCISSOR_TEST */ => ctx_obj.scissor_test_enabled = true,
        0x0B71 /* DEPTH_TEST */ => ctx_obj.depth_test_enabled = true,
        0x0BE2 /* BLEND */ => ctx_obj.blend_enabled = true,
        _ => {
            set_last_error("unsupported capability");
            return ERR_NOT_IMPLEMENTED;
        }
    }
    ERR_OK
}

pub fn ctx_disable(ctx: u32, cap: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    match cap {
        0x0C11 /* SCISSOR_TEST */ => ctx_obj.scissor_test_enabled = false,
        0x0B71 /* DEPTH_TEST */ => ctx_obj.depth_test_enabled = false,
        0x0BE2 /* BLEND */ => ctx_obj.blend_enabled = false,
        _ => {
            set_last_error("unsupported capability");
            return ERR_NOT_IMPLEMENTED;
        }
    }
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

/// Get a parameter (vector version).
pub fn ctx_get_parameter_v(ctx: u32, pname: u32, dest_ptr: u32, dest_len: u32) -> u32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    match pname {
        GL_VIEWPORT => {
            if dest_len < 16 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 4) };
            dest[0] = ctx_obj.viewport.0;
            dest[1] = ctx_obj.viewport.1;
            dest[2] = ctx_obj.viewport.2 as i32;
            dest[3] = ctx_obj.viewport.3 as i32;
            ERR_OK
        }
        GL_COLOR_CLEAR_VALUE => {
            if dest_len < 16 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut f32, 4) };
            dest[0] = ctx_obj.clear_color[0];
            dest[1] = ctx_obj.clear_color[1];
            dest[2] = ctx_obj.clear_color[2];
            dest[3] = ctx_obj.clear_color[3];
            ERR_OK
        }
        _ => {
            set_last_error("unsupported parameter");
            ERR_INVALID_ARGS
        }
    }
}

/// Get buffer parameter.
pub fn ctx_get_buffer_parameter(ctx: u32, target: u32, pname: u32) -> i32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return -1;
        }
    };

    let buffer_handle = match target {
        GL_ARRAY_BUFFER => ctx_obj.bound_array_buffer,
        GL_ELEMENT_ARRAY_BUFFER => ctx_obj.bound_element_array_buffer,
        _ => {
            set_last_error("invalid buffer target");
            return -1;
        }
    };

    let buffer_handle = match buffer_handle {
        Some(h) => h,
        None => {
            set_last_error("no buffer bound to target");
            return -1;
        }
    };

    let buffer = match ctx_obj.buffers.get(&buffer_handle) {
        Some(b) => b,
        None => {
            set_last_error("buffer not found");
            return -1;
        }
    };

    match pname {
        GL_BUFFER_SIZE => buffer.data.len() as i32,
        _ => {
            set_last_error("invalid parameter name");
            return -1;
        }
    }
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
    ctx_obj.buffers.insert(
        buf_id,
        Buffer {
            data: Vec::new(),
            usage: 0,
        },
    );
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
        GL_ELEMENT_ARRAY_BUFFER => {
            ctx_obj.bound_element_array_buffer = if buf == 0 { None } else { Some(buf) }
        }
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

    let src_slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };

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
    ctx_obj.shaders.insert(
        shader_id,
        Shader {
            type_,
            source: String::new(),
            compiled: false,
            info_log: String::new(),
            module: None,
            info: None,
        },
    );
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

    let source_slice =
        unsafe { std::slice::from_raw_parts(source_ptr as *const u8, source_len as usize) };
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
        let stage = match s.type_ {
            0x8B31 => naga::ShaderStage::Vertex,
            0x8B30 => naga::ShaderStage::Fragment,
            _ => {
                s.compiled = false;
                s.info_log = "Invalid shader type".to_string();
                return ERR_INVALID_ARGS;
            }
        };

        let mut frontend = Frontend::default();
        let options = Options::from(stage);
        let verbosity = ctx_obj.verbosity;

        match frontend.parse(&options, &s.source) {
            Ok(module) => {
                Context::log_static(verbosity, 3, "Shader parsed successfully");
                let mut validator = Validator::new(
                    ValidationFlags::all() & !ValidationFlags::BINDINGS,
                    Capabilities::all(),
                );
                match validator.validate(&module) {
                    Ok(info) => {
                        Context::log_static(verbosity, 3, "Shader validated successfully");
                        s.compiled = true;
                        s.info_log = "Shader compiled successfully".to_string();
                        s.module = Some(Arc::new(module));
                        s.info = Some(Arc::new(info));
                        ERR_OK
                    }
                    Err(e) => {
                        Context::log_static(
                            verbosity,
                            1,
                            &format!("Shader validation error: {:?}", e),
                        );
                        s.compiled = false;
                        s.info_log = format!("Validation error: {:?}", e);
                        ERR_OK
                    }
                }
            }
            Err(e) => {
                Context::log_static(
                    verbosity,
                    1,
                    &format!("Shader parse error: {:?} for source:\n{}", e, s.source),
                );
                s.compiled = false;
                s.info_log = format!("Compilation error: {:?}", e);
                ERR_OK
            }
        }
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
            GL_COMPILE_STATUS => {
                if s.compiled {
                    1
                } else {
                    0
                }
            }
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
    ctx_obj.programs.insert(
        program_id,
        Program {
            attached_shaders: Vec::new(),
            linked: false,
            info_log: String::new(),
            attributes: HashMap::new(),
            uniforms: HashMap::new(),
            vs_module: None,
            fs_module: None,
            vs_info: None,
            fs_info: None,
            vs_wasm: None,
            fs_wasm: None,
        },
    );
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
    ctx_obj.log(
        3,
        &format!(
            "ctx_attach_shader ctx={} program={} shader={}",
            ctx, program, shader
        ),
    );

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
    let verbosity = ctx_obj.verbosity;
    Context::log_static(
        verbosity,
        3,
        &format!("ctx_link_program ctx={} program={}", ctx, program),
    );

    if let Some(p) = ctx_obj.programs.get_mut(&program) {
        let mut vs_module = None;
        let mut fs_module = None;
        let mut vs_info = None;
        let mut fs_info = None;
        let mut vs_source = String::new();
        let mut fs_source = String::new();

        for &s_id in &p.attached_shaders {
            Context::log_static(verbosity, 3, &format!("Checking attached shader {}", s_id));
            if let Some(s) = ctx_obj.shaders.get(&s_id) {
                if !s.compiled {
                    Context::log_static(verbosity, 3, &format!("Shader {} is NOT compiled", s_id));
                    p.linked = false;
                    p.info_log = format!("Shader {} is not compiled", s_id);
                    return ERR_OK;
                }
                match s.type_ {
                    0x8B31 => {
                        Context::log_static(verbosity, 3, "Found VS");
                        vs_module = s.module.clone();
                        vs_info = s.info.clone();
                        vs_source = s.source.clone();
                    }
                    0x8B30 => {
                        Context::log_static(verbosity, 3, "Found FS");
                        fs_module = s.module.clone();
                        fs_info = s.info.clone();
                        fs_source = s.source.clone();
                    }
                    _ => {}
                }
            } else {
                Context::log_static(
                    verbosity,
                    3,
                    &format!("Shader {} NOT FOUND in context", s_id),
                );
            }
        }

        if vs_module.is_none() || fs_module.is_none() {
            Context::log_static(
                verbosity,
                3,
                &format!(
                    "Missing shaders: VS={} FS={}",
                    vs_module.is_none(),
                    fs_module.is_none()
                ),
            );
            p.linked = false;
            p.info_log = "Program must have both vertex and fragment shaders".to_string();
            return ERR_OK;
        }

        p.vs_module = vs_module;
        p.fs_module = fs_module;
        p.vs_info = vs_info;
        p.fs_info = fs_info;

        // Extract attributes and uniforms from Naga modules to ensure consistent locations
        p.attributes.clear();
        p.uniforms.clear();
        let mut uniform_locations = HashMap::new();
        let mut next_uniform_loc = 0;
        let mut varying_locations = HashMap::new();
        let mut next_varying_loc = 0; // gl_Position is handled separately at offset 0

        if let Some(vs) = &p.vs_module {
            for ep in &vs.entry_points {
                if ep.stage == ShaderStage::Vertex {
                    for arg in &ep.function.arguments {
                        if let Some(name) = &arg.name {
                            if let Some(Binding::Location { location, .. }) = &arg.binding {
                                p.attributes.insert(name.clone(), *location as i32);
                            }
                        }
                    }
                }
            }
            for (_, var) in vs.global_variables.iter() {
                if let AddressSpace::Uniform | AddressSpace::Handle = var.space {
                    if let Some(name) = &var.name {
                        if !p.uniforms.contains_key(name) {
                            p.uniforms.insert(name.clone(), next_uniform_loc as i32);
                            uniform_locations.insert(name.clone(), next_uniform_loc);
                            next_uniform_loc += 1;
                        }
                    }
                } else if var.space == AddressSpace::Private {
                    if let Some(name) = &var.name {
                        if name != "gl_Position"
                            && name != "gl_Position_1"
                            && !p.attributes.contains_key(name)
                        {
                            if !varying_locations.contains_key(name) {
                                varying_locations.insert(name.clone(), next_varying_loc);
                                next_varying_loc += 1;
                            }
                        }
                    }
                }
            }
        }

        if let Some(fs) = &p.fs_module {
            for (_, var) in fs.global_variables.iter() {
                if let AddressSpace::Uniform | AddressSpace::Handle = var.space {
                    if let Some(name) = &var.name {
                        if !p.uniforms.contains_key(name) {
                            p.uniforms.insert(name.clone(), next_uniform_loc as i32);
                            uniform_locations.insert(name.clone(), next_uniform_loc);
                            next_uniform_loc += 1;
                        }
                    }
                } else if var.space == AddressSpace::Private {
                    if let Some(name) = &var.name {
                        if name != "color" && name != "gl_FragColor" && name != "gl_FragColor_1" {
                            if !varying_locations.contains_key(name) {
                                varying_locations.insert(name.clone(), next_varying_loc);
                                next_varying_loc += 1;
                            }
                        }
                    }
                }
            }
        }

        // Compile to WASM
        let backend = WasmBackend::new(WasmBackendConfig::default());

        if let (Some(vs), Some(vsi)) = (&p.vs_module, &p.vs_info) {
            Context::log_static(verbosity, 3, "Compiling VS to WASM");
            match backend.compile(
                vs,
                vsi,
                &vs_source,
                naga::ShaderStage::Vertex,
                &uniform_locations,
                &varying_locations,
            ) {
                Ok(wasm) => {
                    Context::log_static(
                        verbosity,
                        3,
                        &format!("VS WASM compiled, size={}", wasm.wasm_bytes.len()),
                    );
                    p.vs_wasm = Some(wasm.wasm_bytes);
                }
                Err(e) => {
                    Context::log_static(verbosity, 1, &format!("VS Backend error: {:?}", e));
                    p.linked = false;
                    p.info_log = format!("VS Backend error: {:?}", e);
                    return ERR_OK;
                }
            }
        }

        if let (Some(fs), Some(fsi)) = (&p.fs_module, &p.fs_info) {
            Context::log_static(verbosity, 3, "Compiling FS to WASM");
            match backend.compile(
                fs,
                fsi,
                &fs_source,
                naga::ShaderStage::Fragment,
                &uniform_locations,
                &varying_locations,
            ) {
                Ok(wasm) => {
                    Context::log_static(
                        verbosity,
                        3,
                        &format!("FS WASM compiled, size={}", wasm.wasm_bytes.len()),
                    );
                    p.fs_wasm = Some(wasm.wasm_bytes);
                }
                Err(e) => {
                    Context::log_static(verbosity, 1, &format!("FS Backend error: {:?}", e));
                    p.linked = false;
                    p.info_log = format!("FS Backend error: {:?}", e);
                    return ERR_OK;
                }
            }
        }

        p.linked = true;
        p.info_log = "Program linked successfully.".to_string();
        return ERR_OK;
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
            GL_LINK_STATUS => {
                if p.linked {
                    1
                } else {
                    0
                }
            }
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

/// Get the length of the generated WASM for a program's shader.
pub fn ctx_get_program_wasm_len(ctx: u32, program: u32, shader_type: u32) -> u32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return 0,
    };

    if let Some(p) = ctx_obj.programs.get(&program) {
        let wasm = match shader_type {
            0x8B31 => &p.vs_wasm,
            0x8B30 => &p.fs_wasm,
            _ => return 0,
        };

        if let Some(bytes) = wasm {
            bytes.len() as u32
        } else {
            0
        }
    } else {
        0
    }
}

/// Get the generated WASM for a program's shader.
pub fn ctx_get_program_wasm(
    ctx: u32,
    program: u32,
    shader_type: u32,
    dest_ptr: u32,
    dest_len: u32,
) -> u32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return 0,
    };

    if let Some(p) = ctx_obj.programs.get(&program) {
        let wasm = match shader_type {
            0x8B31 => &p.vs_wasm,
            0x8B30 => &p.fs_wasm,
            _ => return 0,
        };

        if let Some(bytes) = wasm {
            let len = std::cmp::min(bytes.len(), dest_len as usize);
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), dest_ptr as *mut u8, len);
            }
            len as u32
        } else {
            0
        }
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

    let name_slice =
        unsafe { std::slice::from_raw_parts(name_ptr as *const u8, name_len as usize) };
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
pub fn ctx_bind_attrib_location(
    ctx: u32,
    program: u32,
    index: u32,
    name_ptr: u32,
    name_len: u32,
) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    let name_slice =
        unsafe { std::slice::from_raw_parts(name_ptr as *const u8, name_len as usize) };
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

    let name_slice =
        unsafe { std::slice::from_raw_parts(name_ptr as *const u8, name_len as usize) };
    let name = String::from_utf8_lossy(name_slice);

    if let Some(p) = ctx_obj.programs.get(&program) {
        if let Some(&loc) = p.uniforms.get(name.as_ref()) {
            loc
        } else {
            -1
        }
    } else {
        -1
    }
}

/// Set uniform 1f.
pub fn ctx_uniform1f(ctx: u32, location: i32, x: f32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if location < 0 {
        return ERR_OK;
    }

    if (location as usize * 64 + 4) <= ctx_obj.uniform_data.len() {
        let offset = location as usize * 64;
        ctx_obj.uniform_data[offset..offset + 4].copy_from_slice(&x.to_le_bytes());
        ERR_OK
    } else {
        set_last_error("invalid uniform location");
        ERR_INVALID_ARGS
    }
}

/// Set uniform 2f.
pub fn ctx_uniform2f(ctx: u32, location: i32, x: f32, y: f32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if location < 0 {
        return ERR_OK;
    }

    if (location as usize * 64 + 8) <= ctx_obj.uniform_data.len() {
        let offset = location as usize * 64;
        ctx_obj.uniform_data[offset..offset + 4].copy_from_slice(&x.to_le_bytes());
        ctx_obj.uniform_data[offset + 4..offset + 8].copy_from_slice(&y.to_le_bytes());
        ERR_OK
    } else {
        set_last_error("invalid uniform location");
        ERR_INVALID_ARGS
    }
}

/// Set uniform 3f.
pub fn ctx_uniform3f(ctx: u32, location: i32, x: f32, y: f32, z: f32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if location < 0 {
        return ERR_OK;
    }

    if (location as usize * 64 + 12) <= ctx_obj.uniform_data.len() {
        let offset = location as usize * 64;
        ctx_obj.uniform_data[offset..offset + 4].copy_from_slice(&x.to_le_bytes());
        ctx_obj.uniform_data[offset + 4..offset + 8].copy_from_slice(&y.to_le_bytes());
        ctx_obj.uniform_data[offset + 8..offset + 12].copy_from_slice(&z.to_le_bytes());
        ERR_OK
    } else {
        set_last_error("invalid uniform location");
        ERR_INVALID_ARGS
    }
}

/// Set uniform 4f.
pub fn ctx_uniform4f(ctx: u32, location: i32, x: f32, y: f32, z: f32, w: f32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if location < 0 {
        return ERR_OK;
    }

    if (location as usize * 64 + 16) <= ctx_obj.uniform_data.len() {
        let offset = location as usize * 64;
        ctx_obj.uniform_data[offset..offset + 4].copy_from_slice(&x.to_le_bytes());
        ctx_obj.uniform_data[offset + 4..offset + 8].copy_from_slice(&y.to_le_bytes());
        ctx_obj.uniform_data[offset + 8..offset + 12].copy_from_slice(&z.to_le_bytes());
        ctx_obj.uniform_data[offset + 12..offset + 16].copy_from_slice(&w.to_le_bytes());
        ERR_OK
    } else {
        set_last_error("invalid uniform location");
        ERR_INVALID_ARGS
    }
}

/// Set uniform 1i.
pub fn ctx_uniform1i(ctx: u32, location: i32, x: i32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if location < 0 {
        return ERR_OK;
    }

    if (location as usize * 64 + 4) <= ctx_obj.uniform_data.len() {
        let offset = location as usize * 64;
        ctx_obj.uniform_data[offset..offset + 4].copy_from_slice(&x.to_le_bytes());
        ERR_OK
    } else {
        set_last_error("invalid uniform location");
        ERR_INVALID_ARGS
    }
}

/// Set uniform matrix 4fv.
pub fn ctx_uniform_matrix_4fv(ctx: u32, location: i32, transpose: bool, ptr: u32, len: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if location < 0 {
        return ERR_OK;
    }

    if transpose {
        set_last_error("transpose not supported");
        return ERR_INVALID_ARGS;
    }

    if (location as usize * 64 + len as usize * 4) <= ctx_obj.uniform_data.len() {
        let offset = location as usize * 64;
        let src_slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, (len * 4) as usize) };
        ctx_obj.uniform_data[offset..offset + (len * 4) as usize].copy_from_slice(src_slice);
        ERR_OK
    } else {
        set_last_error("invalid uniform location or data length");
        ERR_INVALID_ARGS
    }
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
pub fn ctx_vertex_attrib_pointer(
    ctx: u32,
    index: u32,
    size: i32,
    type_: u32,
    normalized: bool,
    stride: i32,
    offset: u32,
) -> u32 {
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

/// Set vertex attribute default value (1f).
pub fn ctx_vertex_attrib1f(ctx: u32, index: u32, v0: f32) -> u32 {
    ctx_vertex_attrib4f(ctx, index, v0, 0.0, 0.0, 1.0)
}

/// Set vertex attribute default value (2f).
pub fn ctx_vertex_attrib2f(ctx: u32, index: u32, v0: f32, v1: f32) -> u32 {
    ctx_vertex_attrib4f(ctx, index, v0, v1, 0.0, 1.0)
}

/// Set vertex attribute default value (3f).
pub fn ctx_vertex_attrib3f(ctx: u32, index: u32, v0: f32, v1: f32, v2: f32) -> u32 {
    ctx_vertex_attrib4f(ctx, index, v0, v1, v2, 1.0)
}

/// Set vertex attribute default value (4f).
pub fn ctx_vertex_attrib4f(ctx: u32, index: u32, v0: f32, v1: f32, v2: f32, v3: f32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if (index as usize) < ctx_obj.vertex_attributes.len() {
        ctx_obj.vertex_attributes[index as usize].default_value = [v0, v1, v2, v3];
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
    let reg_ptr = &mut *reg;

    let ctx_obj = match reg_ptr.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    let _program_id = match ctx_obj.current_program {
        Some(p) => p,
        None => {
            set_last_error("no program bound");
            return ERR_INVALID_ARGS;
        }
    };

    let (vx, vy, vw, vh) = ctx_obj.viewport;
    let mut vertices = Vec::with_capacity(count as usize);

    // 1. Run VS for all vertices
    let mut attr_data = vec![0.0f32; 16 * 4];
    for i in 0..count {
        ctx_obj.fetch_vertex_attributes((first + i) as u32, &mut attr_data);

        let attr_ptr = 0x2000;
        let uniform_ptr = 0x1000;
        let varying_ptr = 0x3000;
        let private_ptr = 0x4000;
        let texture_ptr = 0x5000;

        // Ensure attr_data is large enough for 64-byte alignment per location
        let mut aligned_attr_data = vec![0.0f32; 1024]; // Enough for 16 locations * 16 floats
        for (loc, chunk) in attr_data.chunks(4).enumerate() {
            let offset = loc * 16;
            if offset + 4 <= aligned_attr_data.len() {
                aligned_attr_data[offset..offset + 4].copy_from_slice(chunk);
            }
        }

        unsafe {
            std::ptr::copy_nonoverlapping(
                aligned_attr_data.as_ptr() as *const u8,
                attr_ptr as *mut u8,
                aligned_attr_data.len() * 4,
            );
            std::ptr::copy_nonoverlapping(
                ctx_obj.uniform_data.as_ptr() as *const u8,
                uniform_ptr as *mut u8,
                ctx_obj.uniform_data.len(),
            );
            ctx_obj.prepare_texture_metadata(texture_ptr);
        }

        crate::js_execute_shader(
            0x8B31, /* VERTEX_SHADER */
            attr_ptr as i32,
            uniform_ptr as i32,
            varying_ptr as i32,
            private_ptr as i32,
            texture_ptr as i32,
        );

        let mut pos_bytes = [0u8; 16];
        let mut varying_bytes = vec![0u8; 256]; // Capture first 256 bytes of varyings
        unsafe {
            std::ptr::copy_nonoverlapping(varying_ptr as *const u8, pos_bytes.as_mut_ptr(), 16);
            std::ptr::copy_nonoverlapping(
                varying_ptr as *const u8,
                varying_bytes.as_mut_ptr(),
                256,
            );
        }
        let pos: [f32; 4] = unsafe { std::mem::transmute(pos_bytes) };

        vertices.push(Vertex {
            pos,
            varyings: varying_bytes,
        });
    }

    // 2. Rasterize
    if mode == 0x0000 {
        // GL_POINTS
        for v in &vertices {
            let screen_x = vx as f32 + (v.pos[0] / v.pos[3] + 1.0) * 0.5 * vw as f32;
            let screen_y = vy as f32 + (v.pos[1] / v.pos[3] + 1.0) * 0.5 * vh as f32;

            // Run FS
            let uniform_ptr = 0x1000;
            let varying_ptr = 0x3000;
            let private_ptr = 0x4000;
            let texture_ptr = 0x5000;

            unsafe {
                std::ptr::copy_nonoverlapping(
                    v.varyings.as_ptr(),
                    varying_ptr as *mut u8,
                    v.varyings.len(),
                );
            }

            crate::js_execute_shader(
                0x8B30, /* FRAGMENT_SHADER */
                0,
                uniform_ptr as i32,
                varying_ptr as i32,
                private_ptr as i32,
                texture_ptr as i32,
            );

            let mut color_bytes = [0u8; 16];
            unsafe {
                std::ptr::copy_nonoverlapping(
                    private_ptr as *const u8,
                    color_bytes.as_mut_ptr(),
                    16,
                );
            }
            let c: [f32; 4] = unsafe { std::mem::transmute(color_bytes) };
            let color_u8 = [
                (c[0].clamp(0.0, 1.0) * 255.0) as u8,
                (c[1].clamp(0.0, 1.0) * 255.0) as u8,
                (c[2].clamp(0.0, 1.0) * 255.0) as u8,
                (c[3].clamp(0.0, 1.0) * 255.0) as u8,
            ];
            ctx_obj.rasterizer.draw_point(
                &mut ctx_obj.default_framebuffer,
                screen_x,
                screen_y,
                color_u8,
            );
        }
    } else if mode == 0x0004 {
        // GL_TRIANGLES
        for i in (0..vertices.len()).step_by(3) {
            if i + 2 >= vertices.len() {
                break;
            }
            let v0 = &vertices[i];
            let v1 = &vertices[i + 1];
            let v2 = &vertices[i + 2];

            // Screen coordinates (with perspective divide)
            let p0 = (
                vx as f32 + (v0.pos[0] / v0.pos[3] + 1.0) * 0.5 * vw as f32,
                vy as f32 + (v0.pos[1] / v0.pos[3] + 1.0) * 0.5 * vh as f32,
            );
            let p1 = (
                vx as f32 + (v1.pos[0] / v1.pos[3] + 1.0) * 0.5 * vw as f32,
                vy as f32 + (v1.pos[1] / v1.pos[3] + 1.0) * 0.5 * vh as f32,
            );
            let p2 = (
                vx as f32 + (v2.pos[0] / v2.pos[3] + 1.0) * 0.5 * vw as f32,
                vy as f32 + (v2.pos[1] / v2.pos[3] + 1.0) * 0.5 * vh as f32,
            );

            // Bounding box
            let min_x = p0.0.min(p1.0).min(p2.0).max(0.0).floor() as i32;
            let max_x = p0.0.max(p1.0).max(p2.0).min(vw as f32 - 1.0).ceil() as i32;
            let min_y = p0.1.min(p1.1).min(p2.1).max(0.0).floor() as i32;
            let max_y = p0.1.max(p1.1).max(p2.1).min(vh as f32 - 1.0).ceil() as i32;

            if max_x >= min_x && max_y >= min_y {
                let w0_inv = 1.0 / v0.pos[3];
                let w1_inv = 1.0 / v1.pos[3];
                let w2_inv = 1.0 / v2.pos[3];

                let v0_f32: &[f32] =
                    unsafe { std::slice::from_raw_parts(v0.varyings.as_ptr() as *const f32, 64) };
                let v1_f32: &[f32] =
                    unsafe { std::slice::from_raw_parts(v1.varyings.as_ptr() as *const f32, 64) };
                let v2_f32: &[f32] =
                    unsafe { std::slice::from_raw_parts(v2.varyings.as_ptr() as *const f32, 64) };

                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        let (u, v, w) = barycentric((x as f32 + 0.5, y as f32 + 0.5), p0, p1, p2);
                        if u >= 0.0 && v >= 0.0 && w >= 0.0 {
                            // Interpolate depth (NDC z/w mapped to [0, 1])
                            let z0 = v0.pos[2] / v0.pos[3];
                            let z1 = v1.pos[2] / v1.pos[3];
                            let z2 = v2.pos[2] / v2.pos[3];
                            let depth_ndc = u * z0 + v * z1 + w * z2;
                            let depth = (depth_ndc + 1.0) * 0.5;

                            let fb_idx =
                                (y as u32 * ctx_obj.default_framebuffer.width + x as u32) as usize;

                            if depth >= 0.0
                                && depth <= 1.0
                                && depth < ctx_obj.default_framebuffer.depth[fb_idx]
                            {
                                ctx_obj.default_framebuffer.depth[fb_idx] = depth;

                                // Perspective correct interpolation
                                let w_interp_inv = u * w0_inv + v * w1_inv + w * w2_inv;
                                let w_interp = 1.0 / w_interp_inv;

                                let mut interp_f32 = [0.0f32; 64];
                                for k in 0..64 {
                                    interp_f32[k] = (u * v0_f32[k] * w0_inv
                                        + v * v1_f32[k] * w1_inv
                                        + w * v2_f32[k] * w2_inv)
                                        * w_interp;
                                }

                                // Run FS
                                let uniform_ptr = 0x1000;
                                let varying_ptr = 0x3000;
                                let private_ptr = 0x4000;
                                let texture_ptr = 0x5000;

                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        interp_f32.as_ptr() as *const u8,
                                        varying_ptr as *mut u8,
                                        256,
                                    );
                                }

                                crate::js_execute_shader(
                                    0x8B30, /* FRAGMENT_SHADER */
                                    0,
                                    uniform_ptr as i32,
                                    varying_ptr as i32,
                                    private_ptr as i32,
                                    texture_ptr as i32,
                                );

                                let mut color_bytes = [0u8; 16];
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        private_ptr as *const u8,
                                        color_bytes.as_mut_ptr(),
                                        16,
                                    );
                                }
                                let c: [f32; 4] = unsafe { std::mem::transmute(color_bytes) };
                                let color_u8 = [
                                    (c[0].clamp(0.0, 1.0) * 255.0) as u8,
                                    (c[1].clamp(0.0, 1.0) * 255.0) as u8,
                                    (c[2].clamp(0.0, 1.0) * 255.0) as u8,
                                    (c[3].clamp(0.0, 1.0) * 255.0) as u8,
                                ];

                                let color_idx = fb_idx * 4;
                                if color_idx + 3 < ctx_obj.default_framebuffer.color.len() {
                                    ctx_obj.default_framebuffer.color[color_idx..color_idx + 4]
                                        .copy_from_slice(&color_u8);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    ERR_OK
}

/// Set verbosity level for the context.
pub fn ctx_set_verbosity(ctx: u32, level: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    if let Some(ctx_obj) = reg.contexts.get_mut(&ctx) {
        ctx_obj.verbosity = level;
        ERR_OK
    } else {
        ERR_INVALID_HANDLE
    }
}

/// Draw elements.
pub fn ctx_draw_elements(ctx: u32, _mode: u32, _count: i32, _type_: u32, _offset: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    let _program_id = match ctx_obj.current_program {
        Some(p) => p,
        None => {
            set_last_error("no program bound");
            return ERR_INVALID_ARGS;
        }
    };

    // TODO: Implement internal software rasterization/execution
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

    // Get the source data and dimensions
    let (src_data, src_width, src_height) = if let Some(fb_handle) = ctx_obj.bound_framebuffer {
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
        let tex = match ctx_obj.textures.get(&tex_handle) {
            Some(t) => t,
            None => {
                set_last_error("attached texture not found");
                return ERR_INVALID_HANDLE;
            }
        };
        (&tex.data, tex.width, tex.height)
    } else {
        (
            &ctx_obj.default_framebuffer.color,
            ctx_obj.default_framebuffer.width,
            ctx_obj.default_framebuffer.height,
        )
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
    let dest_slice =
        unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut u8, dest_len as usize) };

    let mut dst_off = 0;
    for row in 0..height {
        for col in 0..width {
            let sx = x as i32 + col as i32;
            let sy = y as i32 + row as i32;

            if sx >= 0 && sx < src_width as i32 && sy >= 0 && sy < src_height as i32 {
                let src_idx = ((sy as u32 * src_width + sx as u32) * 4) as usize;
                if src_idx + 3 < src_data.len() {
                    dest_slice[dst_off] = src_data[src_idx];
                    dest_slice[dst_off + 1] = src_data[src_idx + 1];
                    dest_slice[dst_off + 2] = src_data[src_idx + 2];
                    dest_slice[dst_off + 3] = src_data[src_idx + 3];
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
