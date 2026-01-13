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

/// Set debug mode.
/// Deprecated: runtime debug mode toggling is not supported. Debug mode must be set at context creation.
pub fn ctx_set_debug_mode(_ctx: u32, _mode: u32) -> u32 {
    clear_last_error();
    set_last_error("ctx_set_debug_mode is deprecated; set debug at context creation instead");
    ERR_NOT_IMPLEMENTED
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
        let r = (ctx_obj.clear_color[0] * 255.0).round() as u8;
        let g = (ctx_obj.clear_color[1] * 255.0).round() as u8;
        let b = (ctx_obj.clear_color[2] * 255.0).round() as u8;
        let a = (ctx_obj.clear_color[3] * 255.0).round() as u8;

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
            if ctx_obj.depth_state.mask {
                for d in ctx_obj.default_framebuffer.depth.iter_mut() {
                    *d = 1.0; // Default clear depth
                }
            }
        }
    }

    if (mask & 0x00000400) != 0 {
        // GL_STENCIL_BUFFER_BIT
        // Currently we only have a default framebuffer stencil buffer implicitly (if we added it to data type)
        if ctx_obj.bound_framebuffer.is_none() {
            // Check stencil write mask (front face usually used for clear?)
            // Spec says: "The stencil buffer is cleared to the value set by clearStencil. The stencil write mask state is respected."
            // Actually spec says: "The scissor box and the stencil write mask affect the operation of Clear."
            let write_mask = ctx_obj.stencil_state.front.write_mask; // Use front mask? Or does clear ignore it?
                                                                     // "The pixel ownership test, the scissor test, dithering, and the buffer writemasks affect the operation of Clear."
                                                                     // So yes, we must respect mask.

            // For now assume clear value is 0 (we didn't implement clearStencil yet to set clean value to state)
            // Default clear value is 0.

            let clear_val = 0; // TODO: get from state

            for s in ctx_obj.default_framebuffer.stencil.iter_mut() {
                *s = (*s & !write_mask as u8) | (clear_val & write_mask as u8);
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
    ctx_obj.depth_state.func = func;
    ERR_OK
}

pub fn ctx_depth_mask(ctx: u32, flag: bool) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    ctx_obj.depth_state.mask = flag;
    ERR_OK
}

pub fn ctx_color_mask(ctx: u32, r: bool, g: bool, b: bool, a: bool) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    ctx_obj.color_mask.r = r;
    ctx_obj.color_mask.g = g;
    ctx_obj.color_mask.b = b;
    ctx_obj.color_mask.a = a;
    ERR_OK
}

pub fn ctx_stencil_func(ctx: u32, func: u32, ref_: i32, mask: u32) -> u32 {
    // Sets both front and back
    ctx_stencil_func_separate(ctx, 0x0408, func, ref_, mask) // GL_FRONT_AND_BACK
}

pub fn ctx_stencil_func_separate(ctx: u32, face: u32, func: u32, ref_: i32, mask: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    // GL_FRONT = 0x0404, GL_BACK = 0x0405, GL_FRONT_AND_BACK = 0x0408
    if face == 0x0404 || face == 0x0408 {
        ctx_obj.stencil_state.front.func = func;
        ctx_obj.stencil_state.front.ref_val = ref_;
        ctx_obj.stencil_state.front.mask = mask;
    }
    if face == 0x0405 || face == 0x0408 {
        ctx_obj.stencil_state.back.func = func;
        ctx_obj.stencil_state.back.ref_val = ref_;
        ctx_obj.stencil_state.back.mask = mask;
    }
    ERR_OK
}

pub fn ctx_stencil_op(ctx: u32, fail: u32, zfail: u32, zpass: u32) -> u32 {
    // Sets both front and back
    ctx_stencil_op_separate(ctx, 0x0408, fail, zfail, zpass)
}

pub fn ctx_stencil_op_separate(ctx: u32, face: u32, fail: u32, zfail: u32, zpass: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    // GL_FRONT = 0x0404, GL_BACK = 0x0405, GL_FRONT_AND_BACK = 0x0408
    if face == 0x0404 || face == 0x0408 {
        ctx_obj.stencil_state.front.fail = fail;
        ctx_obj.stencil_state.front.zfail = zfail;
        ctx_obj.stencil_state.front.zpass = zpass;
    }
    if face == 0x0405 || face == 0x0408 {
        ctx_obj.stencil_state.back.fail = fail;
        ctx_obj.stencil_state.back.zfail = zfail;
        ctx_obj.stencil_state.back.zpass = zpass;
    }
    ERR_OK
}

pub fn ctx_stencil_mask(ctx: u32, mask: u32) -> u32 {
    // Sets both front and back
    ctx_stencil_mask_separate(ctx, 0x0408, mask)
}

pub fn ctx_stencil_mask_separate(ctx: u32, face: u32, mask: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if face == 0x0404 || face == 0x0408 {
        ctx_obj.stencil_state.front.write_mask = mask;
    }
    if face == 0x0405 || face == 0x0408 {
        ctx_obj.stencil_state.back.write_mask = mask;
    }
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
        0x0B71 /* DEPTH_TEST */ => ctx_obj.depth_state.enabled = true,
        0x0BE2 /* BLEND */ => ctx_obj.blend_state.enabled = true,
        0x0B90 /* STENCIL_TEST */ => ctx_obj.stencil_state.enabled = true,
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
        0x0B71 /* DEPTH_TEST */ => ctx_obj.depth_state.enabled = false,
        0x0BE2 /* BLEND */ => ctx_obj.blend_state.enabled = false,
        0x0B90 /* STENCIL_TEST */ => ctx_obj.stencil_state.enabled = false,
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
        0x0C23 => {
            // COLOR_WRITEMASK
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut u8, 4) };
            dest[0] = ctx_obj.color_mask.r as u8;
            dest[1] = ctx_obj.color_mask.g as u8;
            dest[2] = ctx_obj.color_mask.b as u8;
            dest[3] = ctx_obj.color_mask.a as u8;
            ERR_OK
        }
        0x0B72 => {
            // DEPTH_WRITEMASK
            if dest_len < 1 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut u8, 1) };
            dest[0] = ctx_obj.depth_state.mask as u8;
            ERR_OK
        }
        0x0B98 => {
            // STENCIL_WRITEMASK
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = ctx_obj.stencil_state.front.write_mask as i32;
            ERR_OK
        }
        0x8CA5 => {
            // STENCIL_BACK_WRITEMASK
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = ctx_obj.stencil_state.back.write_mask as i32;
            ERR_OK
        }
        0x0B74 => {
            // DEPTH_FUNC
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = ctx_obj.depth_state.func as i32;
            ERR_OK
        }
        0x0B92 => {
            // STENCIL_FUNC
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = ctx_obj.stencil_state.front.func as i32;
            ERR_OK
        }
        0x0B93 => {
            // STENCIL_VALUE_MASK
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = ctx_obj.stencil_state.front.mask as i32;
            ERR_OK
        }
        0x0B97 => {
            // STENCIL_REF
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = ctx_obj.stencil_state.front.ref_val as i32;
            ERR_OK
        }
        0x8800 => {
            // STENCIL_BACK_FUNC
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = ctx_obj.stencil_state.back.func as i32;
            ERR_OK
        }
        0x8CA4 => {
            // STENCIL_BACK_VALUE_MASK
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = ctx_obj.stencil_state.back.mask as i32;
            ERR_OK
        }
        0x8CA3 => {
            // STENCIL_BACK_REF
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = ctx_obj.stencil_state.back.ref_val as i32;
            ERR_OK
        }
        0x0B94 => {
            // STENCIL_FAIL
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = ctx_obj.stencil_state.front.fail as i32;
            ERR_OK
        }
        0x0B95 => {
            // STENCIL_PASS_DEPTH_FAIL
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = ctx_obj.stencil_state.front.zfail as i32;
            ERR_OK
        }
        0x0B96 => {
            // STENCIL_PASS_DEPTH_PASS
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = ctx_obj.stencil_state.front.zpass as i32;
            ERR_OK
        }
        0x8801 => {
            // STENCIL_BACK_FAIL
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = ctx_obj.stencil_state.back.fail as i32;
            ERR_OK
        }
        0x8802 => {
            // STENCIL_BACK_PASS_DEPTH_FAIL
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = ctx_obj.stencil_state.back.zfail as i32;
            ERR_OK
        }
        0x8803 => {
            // STENCIL_BACK_PASS_DEPTH_PASS
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = ctx_obj.stencil_state.back.zpass as i32;
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
