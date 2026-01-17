use super::registry::{clear_last_error, get_registry, set_last_error};
use super::types::*;

// ============================================================================
// Renderbuffer Operations
// ============================================================================

/// Check if object is a renderbuffer.
pub fn ctx_is_renderbuffer(ctx: u32, handle: u32) -> bool {
    clear_last_error();
    if handle == 0 {
        return false;
    }
    let reg = get_registry().borrow();
    if let Some(c) = reg.contexts.get(&ctx) {
        c.renderbuffers.contains_key(&handle)
    } else {
        false
    }
}

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
            gpu_handle: GpuHandle::invalid(),
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

    if let Some(rb) = ctx_obj.renderbuffers.remove(&renderbuffer) {
        ctx_obj.kernel.destroy_buffer(rb.gpu_handle);
        if ctx_obj.bound_renderbuffer == Some(renderbuffer) {
            ctx_obj.bound_renderbuffer = None;
        }
        ERR_OK
    } else {
        set_last_error("renderbuffer not found");
        ERR_INVALID_HANDLE
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

    // Destroy old buffer if valid
    if rb.gpu_handle.is_valid() {
        ctx_obj.kernel.destroy_buffer(rb.gpu_handle);
    }

    // Create new buffer in kernel
    rb.gpu_handle = ctx_obj.kernel.create_buffer(
        rb.width,
        rb.height,
        1,
        gl_to_wgt_format(internal_format),
        crate::wasm_gl_emu::device::StorageLayout::Linear,
    );

    ERR_OK
}
