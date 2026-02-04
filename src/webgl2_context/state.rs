use super::registry::{clear_last_error, get_registry, set_last_error};
use super::types::*;
use wgpu_types::TextureFormat;

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
        let fb_draw_buffers = if let Some(fb_handle) = ctx_obj.bound_draw_framebuffer {
            ctx_obj
                .framebuffers
                .get(&fb_handle)
                .map(|fb| fb.draw_buffers)
        } else {
            None
        };

        if let Some(draw_buffers) = fb_draw_buffers {
            for &mode in draw_buffers.iter().take(8) {
                if mode >= 0x8CE0 {
                    // GL_COLOR_ATTACHMENTi
                    let idx = (mode - 0x8CE0) as usize;
                    let handle = if let Some(fb_handle) = ctx_obj.bound_draw_framebuffer {
                        ctx_obj.framebuffers.get(&fb_handle).and_then(|fb| {
                            if idx < fb.color_attachments.len() {
                                fb.color_attachments[idx].and_then(|att| match att {
                                    Attachment::Texture(t) => ctx_obj
                                        .textures
                                        .get(&t)
                                        .and_then(|tex| tex.levels.get(&0))
                                        .map(|l| l.gpu_handle),
                                    Attachment::Renderbuffer(r) => {
                                        ctx_obj.renderbuffers.get(&r).map(|rb| rb.gpu_handle)
                                    }
                                })
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    };

                    if let Some(h) = handle {
                        if h.is_valid() {
                            if ctx_obj.scissor_test_enabled {
                                let (sx, sy, sw, sh) = ctx_obj.scissor_box;
                                ctx_obj
                                    .kernel
                                    .clear_rect(h, ctx_obj.clear_color, sx, sy, sw, sh);
                            } else {
                                ctx_obj.kernel.clear(h, ctx_obj.clear_color);
                            }
                        }
                    }
                }
            }
        } else {
            // Default framebuffer
            if ctx_obj.default_draw_buffers[0] != 0 {
                // GL_NONE
                let handle = ctx_obj.default_framebuffer.gpu_handle;
                if ctx_obj.scissor_test_enabled {
                    let (sx, sy, sw, sh) = ctx_obj.scissor_box;
                    ctx_obj
                        .kernel
                        .clear_rect(handle, ctx_obj.clear_color, sx, sy, sw, sh);
                } else {
                    ctx_obj.kernel.clear(handle, ctx_obj.clear_color);
                }
            }
        }
    }

    if (mask & 0x00000100) != 0 {
        // GL_DEPTH_BUFFER_BIT
        if ctx_obj.bound_draw_framebuffer.is_none() {
            ctx_obj
                .default_framebuffer
                .clear_depth(1.0, ctx_obj.depth_state.mask);
        }
    }

    if (mask & 0x00000400) != 0 {
        // GL_STENCIL_BUFFER_BIT
        if ctx_obj.bound_draw_framebuffer.is_none() {
            let write_mask = ctx_obj.stencil_state.front.write_mask;
            let clear_val = 0; // TODO: get from state
            ctx_obj
                .default_framebuffer
                .clear_stencil(clear_val as u8, write_mask as u8);
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
    ctx_obj.default_framebuffer =
        crate::wasm_gl_emu::OwnedFramebuffer::new(&mut ctx_obj.kernel, width, height);
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
        GL_SCISSOR_TEST => ctx_obj.scissor_test_enabled = true,
        GL_DEPTH_TEST => ctx_obj.depth_state.enabled = true,
        GL_BLEND => ctx_obj.blend_state.enabled = true,
        GL_STENCIL_TEST => ctx_obj.stencil_state.enabled = true,
        GL_CULL_FACE => ctx_obj.cull_face_enabled = true,
        _ => {
            set_last_error("unsupported capability");
            return ERR_NOT_IMPLEMENTED;
        }
    }
    ERR_OK
}

/// Check if a capability is enabled.
pub fn ctx_is_enabled(ctx: u32, cap: u32) -> i32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return 0,
    };
    let val = match cap {
        GL_SCISSOR_TEST => ctx_obj.scissor_test_enabled,
        GL_DEPTH_TEST => ctx_obj.depth_state.enabled,
        GL_BLEND => ctx_obj.blend_state.enabled,
        GL_STENCIL_TEST => ctx_obj.stencil_state.enabled,
        GL_CULL_FACE => ctx_obj.cull_face_enabled,
        _ => false,
    };
    if val {
        1
    } else {
        0
    }
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
        GL_SCISSOR_TEST => ctx_obj.scissor_test_enabled = false,
        GL_DEPTH_TEST => ctx_obj.depth_state.enabled = false,
        GL_BLEND => ctx_obj.blend_state.enabled = false,
        GL_STENCIL_TEST => ctx_obj.stencil_state.enabled = false,
        GL_CULL_FACE => ctx_obj.cull_face_enabled = false,
        _ => {
            set_last_error("unsupported capability");
            return ERR_NOT_IMPLEMENTED;
        }
    }
    ERR_OK
}

pub fn ctx_cull_face(ctx: u32, mode: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    ctx_obj.cull_face_mode = mode;
    ERR_OK
}

pub fn ctx_front_face(ctx: u32, mode: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    ctx_obj.front_face = mode;
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
        0x8CA6 | 0x8CAA => {
            // DRAW_FRAMEBUFFER_BINDING or READ_FRAMEBUFFER_BINDING
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = if pname == 0x8CA6 {
                ctx_obj.bound_draw_framebuffer.unwrap_or(0) as i32
            } else {
                ctx_obj.bound_read_framebuffer.unwrap_or(0) as i32
            };
            ERR_OK
        }
        0x8CA7 => {
            // RENDERBUFFER_BINDING
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = ctx_obj.bound_renderbuffer.unwrap_or(0) as i32;
            ERR_OK
        }
        0x8824 => {
            // MAX_DRAW_BUFFERS
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = 8;
            ERR_OK
        }
        0x8CDF => {
            // MAX_COLOR_ATTACHMENTS
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = 8;
            ERR_OK
        }
        0x8825..=0x882C => {
            // DRAW_BUFFER0..7
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let idx = (pname - 0x8825) as usize;
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            if let Some(fb_handle) = ctx_obj.bound_draw_framebuffer {
                if let Some(fb) = ctx_obj.framebuffers.get(&fb_handle) {
                    dest[0] = fb.draw_buffers.get(idx).cloned().unwrap_or(0) as i32;
                } else {
                    dest[0] = 0;
                }
            } else {
                dest[0] = ctx_obj.default_draw_buffers.get(idx).cloned().unwrap_or(0) as i32;
            }
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
        ctx_obj.set_error(error);
        ERR_OK
    } else {
        ERR_INVALID_HANDLE
    }
}

pub fn ctx_clear_buffer_fv(ctx: u32, buffer: u32, drawbuffer: i32, ptr: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if buffer == GL_COLOR {
        if !(0..8).contains(&drawbuffer) {
            return GL_INVALID_VALUE;
        }
        let handle = ctx_obj.get_color_attachment_handle_at(drawbuffer as usize);
        if handle.is_valid() {
            let (vw, vh) = ctx_obj.get_attachment_size(handle);
            let values = unsafe { std::slice::from_raw_parts(ptr as *const f32, 4) };
            ctx_obj.kernel.clear_rect(
                handle,
                [values[0], values[1], values[2], values[3]],
                0,
                0,
                vw,
                vh,
            );
        }
    }
    ERR_OK
}

pub fn ctx_clear_buffer_iv(ctx: u32, buffer: u32, drawbuffer: i32, ptr: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if buffer == GL_COLOR {
        if !(0..8).contains(&drawbuffer) {
            return GL_INVALID_VALUE;
        }
        let handle = ctx_obj.get_color_attachment_handle_at(drawbuffer as usize);
        if handle.is_valid() {
            let (vw, vh) = ctx_obj.get_attachment_size(handle);
            let buf_format = ctx_obj.kernel.get_buffer(handle).map(|b| b.format);
            let bpp = buf_format
                .and_then(|f| f.block_copy_size(None))
                .unwrap_or(4) as usize;

            let values = unsafe { std::slice::from_raw_parts(ptr as *const i32, 4) };
            let mut pixel_bytes = vec![0u8; bpp];

            if bpp == 16 {
                // RGBA32I
                for i in 0..4 {
                    pixel_bytes[i * 4..(i + 1) * 4].copy_from_slice(&values[i].to_ne_bytes());
                }
            } else if bpp == 8 {
                // RGBA16I or RG32I?
                // Assume 4 components if it's GL_COLOR
                for i in 0..4.min(bpp / 2) {
                    let val = values[i] as i16;
                    pixel_bytes[i * 2..(i + 1) * 2].copy_from_slice(&val.to_ne_bytes());
                }
            } else if bpp == 4 {
                // RGBA8I or R32I?
                if let Some(TextureFormat::R32Sint) = buf_format {
                    pixel_bytes.copy_from_slice(&values[0].to_ne_bytes());
                } else {
                    for i in 0..4.min(bpp) {
                        pixel_bytes[i] = values[i] as u8;
                    }
                }
            }

            ctx_obj
                .kernel
                .clear_rect_raw(handle, &pixel_bytes, 0, 0, vw, vh);
        }
    }
    ERR_OK
}

pub fn ctx_clear_buffer_uiv(ctx: u32, buffer: u32, drawbuffer: i32, ptr: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if buffer == GL_COLOR {
        if !(0..8).contains(&drawbuffer) {
            return GL_INVALID_VALUE;
        }
        let handle = ctx_obj.get_color_attachment_handle_at(drawbuffer as usize);
        if handle.is_valid() {
            let (vw, vh) = ctx_obj.get_attachment_size(handle);
            let buf_format = ctx_obj.kernel.get_buffer(handle).map(|b| b.format);
            let bpp = buf_format
                .and_then(|f| f.block_copy_size(None))
                .unwrap_or(4) as usize;

            let values = unsafe { std::slice::from_raw_parts(ptr as *const u32, 4) };
            let mut pixel_bytes = vec![0u8; bpp];

            if bpp == 16 {
                // RGBA32UI
                for i in 0..4 {
                    pixel_bytes[i * 4..(i + 1) * 4].copy_from_slice(&values[i].to_ne_bytes());
                }
            } else if bpp == 8 {
                // RGBA16UI or RG32UI
                if let Some(TextureFormat::Rg32Uint) = buf_format {
                    pixel_bytes[0..4].copy_from_slice(&values[0].to_ne_bytes());
                    pixel_bytes[4..8].copy_from_slice(&values[1].to_ne_bytes());
                } else {
                    for i in 0..4.min(bpp / 2) {
                        let val = values[i] as u16;
                        pixel_bytes[i * 2..(i + 1) * 2].copy_from_slice(&val.to_ne_bytes());
                    }
                }
            } else if bpp == 4 {
                // RGBA8UI or R32UI
                if let Some(TextureFormat::R32Uint) = buf_format {
                    pixel_bytes.copy_from_slice(&values[0].to_ne_bytes());
                } else {
                    for i in 0..4.min(bpp) {
                        pixel_bytes[i] = values[i] as u8;
                    }
                }
            }

            ctx_obj
                .kernel
                .clear_rect_raw(handle, &pixel_bytes, 0, 0, vw, vh);
        }
    }
    ERR_OK
}
