use super::registry::{clear_last_error, get_registry, set_last_error};
use super::types::*;

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
            // We need to clone the attachment info to avoid borrowing issues
            let attachment = if let Some(fb) = ctx_obj.framebuffers.get(&fb_handle) {
                fb.color_attachment
            } else {
                None
            };

            if let Some(att) = attachment {
                match att {
                    Attachment::Texture(tex_handle) => {
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
                    Attachment::Renderbuffer(rb_handle) => {
                        if let Some(rb) = ctx_obj.renderbuffers.get_mut(&rb_handle) {
                            match rb.internal_format {
                                GL_RGBA4 => {
                                    let r4 = ((ctx_obj.clear_color[0] * 15.0).round() as u16) & 0xF;
                                    let g4 = ((ctx_obj.clear_color[1] * 15.0).round() as u16) & 0xF;
                                    let b4 = ((ctx_obj.clear_color[2] * 15.0).round() as u16) & 0xF;
                                    let a4 = ((ctx_obj.clear_color[3] * 15.0).round() as u16) & 0xF;
                                    let val = (r4 << 12) | (g4 << 8) | (b4 << 4) | a4;
                                    let bytes = val.to_le_bytes();
                                    for i in (0..rb.data.len()).step_by(2) {
                                        if i + 1 < rb.data.len() {
                                            rb.data[i] = bytes[0];
                                            rb.data[i + 1] = bytes[1];
                                        }
                                    }
                                }
                                GL_RGB565 => {
                                    let r5 =
                                        ((ctx_obj.clear_color[0] * 31.0).round() as u16) & 0x1F;
                                    let g6 =
                                        ((ctx_obj.clear_color[1] * 63.0).round() as u16) & 0x3F;
                                    let b5 =
                                        ((ctx_obj.clear_color[2] * 31.0).round() as u16) & 0x1F;
                                    let val = (r5 << 11) | (g6 << 5) | b5;
                                    let bytes = val.to_le_bytes();
                                    for i in (0..rb.data.len()).step_by(2) {
                                        if i + 1 < rb.data.len() {
                                            rb.data[i] = bytes[0];
                                            rb.data[i + 1] = bytes[1];
                                        }
                                    }
                                }
                                GL_RGB5_A1 => {
                                    let r5 =
                                        ((ctx_obj.clear_color[0] * 31.0).round() as u16) & 0x1F;
                                    let g5 =
                                        ((ctx_obj.clear_color[1] * 31.0).round() as u16) & 0x1F;
                                    let b5 =
                                        ((ctx_obj.clear_color[2] * 31.0).round() as u16) & 0x1F;
                                    let a1 = if ctx_obj.clear_color[3] >= 0.5 { 1 } else { 0 };
                                    let val = (r5 << 11) | (g5 << 6) | (b5 << 1) | a1;
                                    let bytes = val.to_le_bytes();
                                    for i in (0..rb.data.len()).step_by(2) {
                                        if i + 1 < rb.data.len() {
                                            rb.data[i] = bytes[0];
                                            rb.data[i + 1] = bytes[1];
                                        }
                                    }
                                }
                                _ => {
                                    rb.data.fill(0);
                                }
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
            // Verify
            if !ctx_obj.default_framebuffer.color.is_empty() {
                let _first_alpha = ctx_obj.default_framebuffer.color[3];
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

pub fn ctx_resize(ctx: u32, width: u32, height: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    ctx_obj.default_framebuffer = crate::wasm_gl_emu::OwnedFramebuffer::new(width, height);
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
    if !(0x84C0..=0x84DF).contains(&texture) {
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

/// Set GL error.
pub fn ctx_set_gl_error(ctx: u32, error: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    if let Some(ctx_obj) = reg.contexts.get_mut(&ctx) {
        if ctx_obj.gl_error == GL_NO_ERROR {
            ctx_obj.gl_error = error;
        }
        ERR_OK
    } else {
        ERR_INVALID_HANDLE
    }
}

/// Set verbosity level.
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
