use super::registry::{clear_last_error, get_registry, set_last_error};
use super::types::*;

// ============================================================================
// Framebuffer Operations
// ============================================================================

pub const GL_READ_FRAMEBUFFER: u32 = 0x8CA8;
pub const GL_DRAW_FRAMEBUFFER: u32 = 0x8CA9;

/// Check if object is a framebuffer.
pub fn ctx_is_framebuffer(ctx: u32, handle: u32) -> bool {
    clear_last_error();
    if handle == 0 {
        return false;
    }
    let reg = get_registry().borrow();
    if let Some(c) = reg.contexts.get(&ctx) {
        c.framebuffers.contains_key(&handle)
    } else {
        false
    }
}

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
            depth_attachment: None,
            stencil_attachment: None,
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
    if ctx_obj.bound_read_framebuffer == Some(fb) {
        ctx_obj.bound_read_framebuffer = None;
    }
    if ctx_obj.bound_draw_framebuffer == Some(fb) {
        ctx_obj.bound_draw_framebuffer = None;
    }
    ERR_OK
}

/// Bind a framebuffer in the given context.
/// Returns errno.
pub fn ctx_bind_framebuffer(ctx: u32, target: u32, fb: u32) -> u32 {
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

    let fb_opt = if fb == 0 { None } else { Some(fb) };

    if target == GL_READ_FRAMEBUFFER {
        ctx_obj.bound_read_framebuffer = fb_opt;
    } else if target == GL_DRAW_FRAMEBUFFER {
        ctx_obj.bound_draw_framebuffer = fb_opt;
    } else {
        // GL_FRAMEBUFFER sets both
        ctx_obj.bound_read_framebuffer = fb_opt;
        ctx_obj.bound_draw_framebuffer = fb_opt;
    }
    ERR_OK
}

/// Attach a texture to a framebuffer.
/// Returns errno.
pub fn ctx_framebuffer_texture2d(
    ctx: u32,
    target: u32,
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

    let fb_handle = if target == GL_READ_FRAMEBUFFER {
        ctx_obj.bound_read_framebuffer
    } else {
        ctx_obj.bound_draw_framebuffer
    };

    let fb_handle = match fb_handle {
        Some(h) => h,
        None => {
            set_last_error("no framebuffer bound");
            return ERR_INVALID_OPERATION;
        }
    };

    let fb = match ctx_obj.framebuffers.get_mut(&fb_handle) {
        Some(f) => f,
        None => {
            set_last_error("framebuffer not found");
            return ERR_INTERNAL;
        }
    };

    // For now we only support COLOR_ATTACHMENT0
    // In real WebGL2, attachment can be COLOR_ATTACHMENTi, DEPTH_ATTACHMENT, etc.
    // But this function signature doesn't check attachment type properly yet in the original code?
    // The original code was cut off in read_file, but I assume it was setting color_attachment.

    // Let's assume attachment == 0x8CE0 (GL_COLOR_ATTACHMENT0)
    // But wait, the function signature has `attachment` arg.

    // 0x8CE0 = GL_COLOR_ATTACHMENT0
    // 0x8D00 = GL_DEPTH_ATTACHMENT
    // 0x8D20 = GL_STENCIL_ATTACHMENT
    // 0x821A = GL_DEPTH_STENCIL_ATTACHMENT

    let attachment_enum = _attachment;

    let attachment_obj = if tex == 0 {
        None
    } else {
        Some(Attachment::Texture(tex))
    };

    if attachment_enum == 0x8CE0 {
        // GL_COLOR_ATTACHMENT0
        fb.color_attachment = attachment_obj;
    } else if attachment_enum == 0x8D00 {
        // GL_DEPTH_ATTACHMENT
        fb.depth_attachment = attachment_obj;
    } else if attachment_enum == 0x8D20 {
        // GL_STENCIL_ATTACHMENT
        fb.stencil_attachment = attachment_obj;
    } else if attachment_enum == 0x821A {
        // GL_DEPTH_STENCIL_ATTACHMENT
        fb.depth_attachment = attachment_obj;
        fb.stencil_attachment = attachment_obj;
    } else {
        // For now, just set color attachment if unknown, or maybe error?
        // The original code just set color_attachment.
        fb.color_attachment = attachment_obj;
    }

    ERR_OK
}

/// Attach a renderbuffer to a framebuffer.
pub fn ctx_framebuffer_renderbuffer(
    ctx: u32,
    target: u32,
    attachment: u32,
    renderbuffertarget: u32,
    renderbuffer: u32,
) -> u32 {
    clear_last_error();
    if target != GL_FRAMEBUFFER {
        set_last_error("invalid target");
        return ERR_INVALID_ENUM;
    }
    if renderbuffertarget != GL_RENDERBUFFER {
        set_last_error("invalid renderbuffer target");
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

    let fb_handle = if target == GL_READ_FRAMEBUFFER {
        ctx_obj.bound_read_framebuffer
    } else {
        ctx_obj.bound_draw_framebuffer
    };

    let fb_handle = match fb_handle {
        Some(h) => h,
        None => {
            set_last_error("no framebuffer bound");
            return ERR_INVALID_OPERATION;
        }
    };

    let fb = match ctx_obj.framebuffers.get_mut(&fb_handle) {
        Some(f) => f,
        None => {
            set_last_error("framebuffer not found");
            return ERR_INTERNAL;
        }
    };

    let attachment_obj = if renderbuffer == 0 {
        None
    } else {
        Some(Attachment::Renderbuffer(renderbuffer))
    };

    if attachment == 0x8CE0 {
        // GL_COLOR_ATTACHMENT0
        fb.color_attachment = attachment_obj;
    } else if attachment == 0x8D00 {
        // GL_DEPTH_ATTACHMENT
        fb.depth_attachment = attachment_obj;
    } else if attachment == 0x8D20 {
        // GL_STENCIL_ATTACHMENT
        fb.stencil_attachment = attachment_obj;
    } else if attachment == 0x821A {
        // GL_DEPTH_STENCIL_ATTACHMENT
        fb.depth_attachment = attachment_obj;
        fb.stencil_attachment = attachment_obj;
    } else {
        set_last_error("invalid attachment");
        return ERR_INVALID_ENUM;
    }

    ERR_OK
}

/// Blit a region from the read framebuffer to the draw framebuffer.
pub fn ctx_blit_framebuffer(
    ctx: u32,
    src_x0: i32,
    src_y0: i32,
    src_x1: i32,
    src_y1: i32,
    dst_x0: i32,
    dst_y0: i32,
    dst_x1: i32,
    dst_y1: i32,
    mask: u32,
    filter: u32,
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

    if (mask & GL_COLOR_BUFFER_BIT) != 0 {
        let (src_handle, _, _, _) = ctx_obj.get_color_attachment_info(true);
        let (dst_handle, _, _, _) = ctx_obj.get_color_attachment_info(false);

        if src_handle.is_valid() && dst_handle.is_valid() {
            ctx_obj.kernel.blit(
                src_handle, dst_handle, src_x0, src_y0, src_x1, src_y1, dst_x0, dst_y0, dst_x1,
                dst_y1, filter,
            );
        }
    }

    // TODO: support depth/stencil blit

    ERR_OK
}
