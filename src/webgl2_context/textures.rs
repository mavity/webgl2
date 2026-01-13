use super::registry::{clear_last_error, get_registry, set_last_error};
use super::types::*;
use std::collections::BTreeMap;

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
            levels: BTreeMap::new(),
            min_filter: 0x2705, // GL_NEAREST_MIPMAP_LINEAR (default)
            mag_filter: 0x2601, // GL_LINEAR (default)
            wrap_s: 0x2901,     // GL_REPEAT (default)
            wrap_t: 0x2901,     // GL_REPEAT (default)
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

/// Set texture parameters.
/// Returns errno.
pub fn ctx_tex_parameter_i(ctx: u32, target: u32, pname: u32, param: i32) -> u32 {
    clear_last_error();
    // Only TEXTURE_2D (0x0DE1) is supported for now
    if target != 0x0DE1 {
        set_last_error("invalid texture target");
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

    let tex_handle = match ctx_obj.bound_texture {
        Some(h) => h,
        None => {
            set_last_error("no texture bound");
            return ERR_INVALID_ARGS;
        }
    };

    let tex = match ctx_obj.textures.get_mut(&tex_handle) {
        Some(t) => t,
        None => {
            set_last_error("texture not found");
            return ERR_INVALID_HANDLE;
        }
    };

    match pname {
        GL_TEXTURE_MIN_FILTER => tex.min_filter = param as u32,
        GL_TEXTURE_MAG_FILTER => tex.mag_filter = param as u32,
        GL_TEXTURE_WRAP_S => tex.wrap_s = param as u32,
        GL_TEXTURE_WRAP_T => tex.wrap_t = param as u32,
        _ => {
            set_last_error("invalid texture parameter");
            return ERR_INVALID_ARGS;
        }
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
#[allow(clippy::too_many_arguments)]
pub fn ctx_tex_image_2d(
    ctx: u32,
    _target: u32,
    level: i32,
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
        let level_data = MipLevel {
            width,
            height,
            data: pixel_data,
        };
        tex.levels.insert(level as usize, level_data);
        ERR_OK
    } else {
        set_last_error("texture not found");
        ERR_INVALID_HANDLE
    }
}

/// Generate mipmaps for the bound texture.
pub fn ctx_generate_mipmap(ctx: u32, target: u32) -> u32 {
    clear_last_error();
    if target != GL_TEXTURE_2D {
        set_last_error("invalid texture target");
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

    let unit = ctx_obj.active_texture_unit as usize;
    if unit >= ctx_obj.texture_units.len() {
        set_last_error("active texture unit out of bounds");
        return ERR_INVALID_OPERATION;
    }

    let tex_handle = match ctx_obj.texture_units[unit] {
        Some(h) => h,
        None => {
            set_last_error("no texture bound");
            return ERR_INVALID_OPERATION;
        }
    };

    let tex = match ctx_obj.textures.get_mut(&tex_handle) {
        Some(t) => t,
        None => {
            set_last_error("texture not found");
            return ERR_INTERNAL;
        }
    };

    if let Some(base) = tex.levels.get(&0) {
        let mut width = base.width;
        let mut height = base.height;
        let mut current_level_idx = 0;
        let mut prev_level_data = base.data.clone();

        while width > 1 || height > 1 {
            let next_width = std::cmp::max(1, width / 2);
            let next_height = std::cmp::max(1, height / 2);
            let next_level_idx = current_level_idx + 1;

            let mut next_data = Vec::with_capacity((next_width * next_height * 4) as usize);

            for y in 0..next_height {
                for x in 0..next_width {
                    let src_x = x * 2;
                    let src_y = y * 2;
                    let mut r_sum = 0u32;
                    let mut g_sum = 0u32;
                    let mut b_sum = 0u32;
                    let mut a_sum = 0u32;
                    let mut count = 0u32;

                    for dy in 0..2 {
                        for dx in 0..2 {
                            let sx = src_x + dx;
                            let sy = src_y + dy;
                            if sx < width && sy < height {
                                let idx = ((sy * width + sx) * 4) as usize;
                                r_sum += prev_level_data[idx] as u32;
                                g_sum += prev_level_data[idx + 1] as u32;
                                b_sum += prev_level_data[idx + 2] as u32;
                                a_sum += prev_level_data[idx + 3] as u32;
                                count += 1;
                            }
                        }
                    }

                    next_data.push((r_sum / count) as u8);
                    next_data.push((g_sum / count) as u8);
                    next_data.push((b_sum / count) as u8);
                    next_data.push((a_sum / count) as u8);
                }
            }

            tex.levels.insert(
                next_level_idx,
                MipLevel {
                    width: next_width,
                    height: next_height,
                    data: next_data.clone(),
                },
            );

            prev_level_data = next_data;
            width = next_width;
            height = next_height;
            current_level_idx = next_level_idx;
        }
    }
    ERR_OK
}

/// Copy pixels from framebuffer to texture image.
pub fn ctx_copy_tex_image_2d(
    ctx: u32,
    target: u32,
    level: i32,
    _internal_format: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    _border: i32,
) -> u32 {
    clear_last_error();
    if target != GL_TEXTURE_2D {
        set_last_error("invalid texture target");
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

    // 1. Identify Source & Extract Data
    let src_snapshot = if let Some(fb_handle) = ctx_obj.bound_framebuffer {
        if let Some(fb) = ctx_obj.framebuffers.get(&fb_handle) {
            // Check Color Attachment
            if let Some(att) = fb.color_attachment {
                match att {
                    Attachment::Texture(tex_id) => {
                        if let Some(t) = ctx_obj.textures.get(&tex_id) {
                            if let Some(l0) = t.levels.get(&0) {
                                Some((l0.data.clone(), l0.width, l0.height, GL_RGBA8))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    Attachment::Renderbuffer(rb_id) => {
                        if let Some(rb) = ctx_obj.renderbuffers.get(&rb_id) {
                            Some((rb.data.clone(), rb.width, rb.height, rb.internal_format))
                        } else {
                            None
                        }
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let (src_data, src_w, src_h, src_fmt) = match src_snapshot {
        Some(s) => s,
        None => (vec![0u8; 4], 1, 1, GL_RGBA8),
    };

    // 2. Perform Copy
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    let mut dst_off = 0;

    for dy in 0..height {
        for dx in 0..width {
            let sx = x + dx;
            let sy = y + dy;

            let mut r = 0;
            let mut g = 0;
            let mut b = 0;
            let mut a = 0;

            if sx >= 0 && sx < src_w as i32 && sy >= 0 && sy < src_h as i32 {
                match src_fmt {
                    GL_RGBA8 => {
                        let idx = ((sy as u32 * src_w + sx as u32) * 4) as usize;
                        if idx + 3 < src_data.len() {
                            r = src_data[idx];
                            g = src_data[idx + 1];
                            b = src_data[idx + 2];
                            a = src_data[idx + 3];
                        }
                    }
                    GL_RGBA4 => {
                        let idx = ((sy as u32 * src_w + sx as u32) * 2) as usize;
                        if idx + 1 < src_data.len() {
                            let val = u16::from_le_bytes([src_data[idx], src_data[idx + 1]]);
                            r = (((val >> 12) & 0xF) as u8) * 17;
                            g = (((val >> 8) & 0xF) as u8) * 17;
                            b = (((val >> 4) & 0xF) as u8) * 17;
                            a = ((val & 0xF) as u8) * 17;
                        }
                    }
                    GL_RGB565 => {
                        let idx = ((sy as u32 * src_w + sx as u32) * 2) as usize;
                        if idx + 1 < src_data.len() {
                            let val = u16::from_le_bytes([src_data[idx], src_data[idx + 1]]);
                            let rv = ((val >> 11) & 0x1F) as u8;
                            let gv = ((val >> 5) & 0x3F) as u8;
                            let bv = (val & 0x1F) as u8;
                            r = (rv << 3) | (rv >> 2);
                            g = (gv << 2) | (gv >> 4);
                            b = (bv << 3) | (bv >> 2);
                            a = 255;
                        }
                    }
                    GL_RGB5_A1 => {
                        let idx = ((sy as u32 * src_w + sx as u32) * 2) as usize;
                        if idx + 1 < src_data.len() {
                            let val = u16::from_le_bytes([src_data[idx], src_data[idx + 1]]);
                            let rv = ((val >> 11) & 0x1F) as u8;
                            let gv = ((val >> 6) & 0x1F) as u8;
                            let bv = ((val >> 1) & 0x1F) as u8;
                            let av = (val & 1) as u8;
                            r = (rv << 3) | (rv >> 2);
                            g = (gv << 3) | (gv >> 2);
                            b = (bv << 3) | (bv >> 2);
                            a = if av == 1 { 255 } else { 0 };
                        }
                    }
                    _ => {}
                }
            }

            pixels[dst_off] = r;
            pixels[dst_off + 1] = g;
            pixels[dst_off + 2] = b;
            pixels[dst_off + 3] = a;
            dst_off += 4;
        }
    }

    let unit = ctx_obj.active_texture_unit as usize;
    if unit >= ctx_obj.texture_units.len() {
        set_last_error("active texture unit out of bounds");
        return ERR_INVALID_OPERATION;
    }

    let tex_handle = match ctx_obj.texture_units[unit] {
        Some(h) => h,
        None => {
            set_last_error("no texture bound");
            return ERR_INVALID_OPERATION;
        }
    };

    if let Some(tex) = ctx_obj.textures.get_mut(&tex_handle) {
        let level_data = MipLevel {
            width: width as u32,
            height: height as u32,
            data: pixels,
        };
        tex.levels.insert(level as usize, level_data);
        ERR_OK
    } else {
        set_last_error("texture not found");
        ERR_INVALID_HANDLE
    }
}
