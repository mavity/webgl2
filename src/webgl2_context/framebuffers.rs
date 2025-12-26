use super::registry::{get_registry, set_last_error, clear_last_error};
use super::types::*;

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
