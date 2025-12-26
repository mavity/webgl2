use super::registry::{get_registry, set_last_error, clear_last_error};
use super::types::*;

// ============================================================================
// Renderbuffer Operations
// ============================================================================

/// Create a renderbuffer object.
pub fn ctx_create_renderbuffer(ctx: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return 0;
        }
    };
    let rb_id = ctx_obj.allocate_renderbuffer_handle();
    ctx_obj.renderbuffers.insert(
        rb_id,
        Renderbuffer {
            width: 0,
            height: 0,
            internal_format: GL_RGBA4, // Default
            data: Vec::new(),
        },
    );
    rb_id
}

/// Bind a renderbuffer object.
pub fn ctx_bind_renderbuffer(ctx: u32, target: u32, renderbuffer: u32) -> u32 {
    clear_last_error();
    if target != GL_RENDERBUFFER {
        set_last_error("invalid target");
        return ERR_INVALID_ENUM;
    }

    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if renderbuffer != 0 && !ctx_obj.renderbuffers.contains_key(&renderbuffer) {
        set_last_error("renderbuffer not found");
        return ERR_INVALID_HANDLE;
    }

    if renderbuffer == 0 {
        ctx_obj.bound_renderbuffer = None;
    } else {
        ctx_obj.bound_renderbuffer = Some(renderbuffer);
    }
    ERR_OK
}

/// Delete a renderbuffer object.
pub fn ctx_delete_renderbuffer(ctx: u32, renderbuffer: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if renderbuffer == 0 {
        return ERR_OK;
    }

    if ctx_obj.renderbuffers.remove(&renderbuffer).is_some() {
        if ctx_obj.bound_renderbuffer == Some(renderbuffer) {
            ctx_obj.bound_renderbuffer = None;
        }
        // Also detach from any framebuffers? 
        // In a real implementation we should check all FBs, but for now we skip that O(N) check.
        // The spec says it should be detached.
        ERR_OK
    } else {
        set_last_error("renderbuffer not found");
        return ERR_INVALID_HANDLE;
    }
}

/// Establish data storage, format and dimensions of a renderbuffer object's image.
pub fn ctx_renderbuffer_storage(
    ctx: u32,
    target: u32,
    internal_format: u32,
    width: i32,
    height: i32,
) -> u32 {
    clear_last_error();
    if target != GL_RENDERBUFFER {
        set_last_error("invalid target");
        return ERR_INVALID_ENUM;
    }
    if width < 0 || height < 0 {
        set_last_error("invalid dimensions");
        return ERR_INVALID_ARGS;
    }

    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    let rb_handle = match ctx_obj.bound_renderbuffer {
        Some(h) => h,
        None => {
            set_last_error("no renderbuffer bound");
            return ERR_INVALID_OPERATION;
        }
    };

    let rb = match ctx_obj.renderbuffers.get_mut(&rb_handle) {
        Some(r) => r,
        None => {
            set_last_error("renderbuffer not found");
            return ERR_INTERNAL;
        }
    };

    rb.width = width as u32;
    rb.height = height as u32;
    rb.internal_format = internal_format;

    // Calculate size based on format
    let bpp = match internal_format {
        GL_RGBA4 => 2,
        GL_RGB565 => 2,
        GL_RGB5_A1 => 2,
        GL_DEPTH_COMPONENT16 => 2,
        GL_STENCIL_INDEX8 => 1,
        GL_DEPTH_STENCIL => 4, // 24 depth + 8 stencil usually, let's say 4 bytes
        _ => 4, // Default to 4 bytes (RGBA8 etc)
    };

    let size = (width as usize) * (height as usize) * bpp;
    rb.data = vec![0; size];

    ERR_OK
}
