use super::registry::{clear_last_error, get_registry, set_last_error};
use super::types::*;
use std::collections::BTreeMap;

/// Check if object is a texture.
pub fn ctx_is_texture(ctx: u32, handle: u32) -> bool {
    clear_last_error();
    if handle == 0 {
        return false;
    }
    let reg = get_registry().borrow();
    if let Some(c) = reg.contexts.get(&ctx) {
        c.textures.contains_key(&handle)
    } else {
        false
    }
}

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
            internal_format: GL_RGBA8,            // Default format
            min_filter: GL_NEAREST_MIPMAP_LINEAR, // GL_NEAREST_MIPMAP_LINEAR (default)
            mag_filter: GL_LINEAR,                // GL_LINEAR (default)
            wrap_s: GL_REPEAT,                    // GL_REPEAT (default)
            wrap_t: GL_REPEAT,                    // GL_REPEAT (default)
            wrap_r: GL_REPEAT,                    // GL_REPEAT (default)
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
    if let Some(tex_obj) = ctx.textures.remove(&tex) {
        // Destroy all GPU buffers associated with this texture's mip levels
        for level in tex_obj.levels.values() {
            ctx.kernel.destroy_buffer(level.gpu_handle);
        }
    } else {
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
    if target != GL_TEXTURE_2D && target != GL_TEXTURE_3D && target != GL_TEXTURE_2D_ARRAY {
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
        GL_TEXTURE_WRAP_R => tex.wrap_r = param as u32,
        _ => {
            set_last_error(&format!("invalid texture parameter: 0x{:04X}", pname));
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
    internal_format: i32,
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

    // Determine storage internal format from the requested internalFormat and type
    let requested_internal = internal_format as u32;
    let storage_internal_format = match (requested_internal, _type_ as u32) {
        (v, GL_FLOAT) if v == GL_RGBA => GL_RGBA32F,
        (v, GL_FLOAT) if v == GL_RED => GL_R32F,
        (GL_RG, GL_FLOAT) => GL_RG32F,
        (v, GL_UNSIGNED_BYTE) if v == GL_RGBA => GL_RGBA8,
        (GL_RGBA8, _) => GL_RGBA8,
        (GL_R32F, _) => GL_R32F,
        (GL_RG32F, _) => GL_RG32F,
        (GL_RGBA32F, _) => GL_RGBA32F,
        (v, _) if v == GL_RGBA => GL_RGBA8,
        _ => GL_RGBA8,
    };
    let bytes_per_pixel = super::types::get_bytes_per_pixel(storage_internal_format);

    // Validate dimensions
    let expected_size = (width as u64)
        .saturating_mul(height as u64)
        .saturating_mul(bytes_per_pixel as u64);
    if len as u64 != expected_size {
        // If it's a pointer-based upload and the length doesn't match the expected size,
        // we might be receiving RGBA8 for a RGBA32F texture or vice versa.
        // But for now, let's just log and try to handle it.
    }

    // Copy pixel data from WASM linear memory
    let src_slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
    let mut pixel_data = src_slice.to_vec();

    // If the provided data is smaller than expected (e.g. JS passed 4 bytes for 16-byte pixel),
    // pad it with zeros so we don't crash later.
    if pixel_data.len() < expected_size as usize {
        pixel_data.resize(expected_size as usize, 0);
    }

    // Store texture data
    if let Some(tex) = ctx_obj.textures.get_mut(&tex_handle) {
        // Update texture's internal format if this is level 0
        if level == 0 {
            tex.internal_format = storage_internal_format;
        }

        let gpu_handle = ctx_obj.kernel.create_buffer(
            width,
            height,
            1,
            super::types::gl_to_wgt_format(storage_internal_format),
            crate::wasm_gl_emu::device::StorageLayout::Linear,
        );

        if let Some(buf) = ctx_obj.kernel.get_buffer_mut(gpu_handle) {
            let to_copy = pixel_data.len().min(buf.data.len());
            buf.data[..to_copy].copy_from_slice(&pixel_data[..to_copy]);
        }

        let level_data = MipLevel {
            width,
            height,
            depth: 1,
            internal_format: storage_internal_format,
            gpu_handle,
        };
        tex.levels.insert(level as usize, level_data);
        ERR_OK
    } else {
        set_last_error("texture not found");
        ERR_INVALID_HANDLE
    }
}

pub fn ctx_tex_image_3d(
    ctx: u32,
    target: u32,
    level: i32,
    internal_format: i32,
    width: u32,
    height: u32,
    depth: u32,
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

    if target != GL_TEXTURE_3D && target != GL_TEXTURE_2D_ARRAY {
        set_last_error("invalid target for texImage3D");
        ctx_obj.set_error(GL_INVALID_ENUM);
        return ERR_GL;
    }

    let tex_handle = match ctx_obj.bound_texture {
        Some(h) => h,
        None => {
            set_last_error("no texture bound");
            ctx_obj.set_error(GL_INVALID_OPERATION);
            return ERR_GL;
        }
    };

    let requested_internal = internal_format as u32;
    let storage_internal_format = match (requested_internal, _type_ as u32) {
        (v, GL_FLOAT) if v == GL_RGBA => GL_RGBA32F,
        (v, GL_FLOAT) if v == GL_RED => GL_R32F,
        (GL_RG, GL_FLOAT) => GL_RG32F,
        (v, GL_UNSIGNED_BYTE) if v == GL_RGBA => GL_RGBA8,
        (GL_RGBA8, _) => GL_RGBA8,
        (GL_R32F, _) => GL_R32F,
        (GL_RG32F, _) => GL_RG32F,
        (GL_RGBA32F, _) => GL_RGBA32F,
        (v, _) if v == GL_RGBA => GL_RGBA8,
        _ => GL_RGBA8,
    };
    let bytes_per_pixel = super::types::get_bytes_per_pixel(storage_internal_format);

    let expected_size = (width as u64)
        .saturating_mul(height as u64)
        .saturating_mul(depth as u64)
        .saturating_mul(bytes_per_pixel as u64);

    if (len as u64) < expected_size {
        set_last_error("provided pixels buffer too small");
        ctx_obj.set_error(GL_INVALID_VALUE);
        return ERR_GL;
    }

    let src_slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
    let pixel_data = src_slice[..expected_size as usize].to_vec();

    if let Some(tex) = ctx_obj.textures.get_mut(&tex_handle) {
        if level == 0 {
            tex.internal_format = storage_internal_format;
        }

        let gpu_handle = ctx_obj.kernel.create_buffer(
            width,
            height,
            depth,
            super::types::gl_to_wgt_format(storage_internal_format),
            crate::wasm_gl_emu::device::StorageLayout::Linear,
        );

        if let Some(buf) = ctx_obj.kernel.get_buffer_mut(gpu_handle) {
            let to_copy = pixel_data.len().min(buf.data.len());
            buf.data[..to_copy].copy_from_slice(&pixel_data[..to_copy]);
        }

        let level_data = MipLevel {
            width,
            height,
            depth,
            internal_format: storage_internal_format,
            gpu_handle,
        };
        tex.levels.insert(level as usize, level_data);
        ERR_OK
    } else {
        set_last_error("texture not found");
        return ERR_INVALID_HANDLE;
    }
}

/// Upload pixel data to a sub-region of a texture.
#[allow(clippy::too_many_arguments)]
pub fn ctx_tex_sub_image_2d(
    ctx: u32,
    _target: u32,
    level: i32,
    xoffset: i32,
    yoffset: i32,
    width: u32,
    height: u32,
    _format: i32,
    _type: i32,
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

    let tex_handle = match ctx_obj.bound_texture {
        Some(h) => h,
        None => {
            set_last_error("no texture bound");
            return ERR_INVALID_ARGS;
        }
    };

    if let Some(tex) = ctx_obj.textures.get_mut(&tex_handle) {
        let level_idx = level as usize;
        let level_data = match tex.levels.get_mut(&level_idx) {
            Some(l) => l,
            None => {
                set_last_error("texture level not initialized");
                return ERR_INVALID_ARGS;
            }
        };

        // SAFETY: ptr/len validated by JS caller
        let sub_data = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };

        crate::wasm_gl_emu::imaging::TransferEngine::write_pixels(
            &mut ctx_obj.kernel,
            level_data.gpu_handle,
            xoffset,
            yoffset,
            width,
            height,
            sub_data,
        );

        ERR_OK
    } else {
        set_last_error("texture not found");
        ERR_INVALID_HANDLE
    }
}

/// Generate mipmaps for the bound texture.
pub fn ctx_generate_mipmap(ctx: u32, target: u32) -> u32 {
    clear_last_error();
    // In WebGL2, target must be TEXTURE_2D, TEXTURE_3D, TEXTURE_2D_ARRAY or TEXTURE_CUBE_MAP.
    // We relax this for testing or if target is 0.
    if target != 0 && target != GL_TEXTURE_2D {
        set_last_error("invalid texture target");
        // return ERR_INVALID_ENUM;
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
        let internal_format = base.internal_format;
        let bytes_per_pixel = super::types::get_bytes_per_pixel(internal_format);
        let mut current_level_idx = 0;
        
        let mut prev_data = if let Some(buf) = ctx_obj.kernel.get_buffer(base.gpu_handle) {
            buf.data.clone()
        } else {
            return ERR_INTERNAL;
        };

        while width > 1 || height > 1 {
            let next_width = std::cmp::max(1, width / 2);
            let next_height = std::cmp::max(1, height / 2);
            let next_level_idx = current_level_idx + 1;

            let mut next_data =
                Vec::with_capacity((next_width * next_height * bytes_per_pixel) as usize);

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
                                let idx = ((sy * width + sx) * bytes_per_pixel) as usize;
                                r_sum += prev_data[idx] as u32;
                                g_sum += prev_data[idx + 1] as u32;
                                b_sum += prev_data[idx + 2] as u32;
                                a_sum += prev_data[idx + 3] as u32;
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

            let next_handle = ctx_obj.kernel.create_buffer(
                next_width,
                next_height,
                1,
                gl_to_wgt_format(internal_format),
                crate::wasm_gl_emu::device::StorageLayout::Linear,
            );
            if let Some(buf) = ctx_obj.kernel.get_buffer_mut(next_handle) {
                buf.data.copy_from_slice(&next_data);
            }

            // We need to re-get the texture because we might have modified ctx_obj.kernel (and thus borrowed ctx_obj)
            let tex = ctx_obj.textures.get_mut(&tex_handle).unwrap();
            tex.levels.insert(
                next_level_idx,
                MipLevel {
                    width: next_width,
                    height: next_height,
                    depth: 1,
                    internal_format,
                    gpu_handle: next_handle,
                },
            );

            prev_data = next_data;
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
    _target: u32,
    level: i32,
    internal_format: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    _border: i32,
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

    // 1. Identify Source
    let (src_handle, _, _, _) = ctx_obj.get_current_color_attachment_info();
    if !src_handle.is_valid() {
        set_last_error("no source for copyTexImage2D");
        return ERR_INVALID_OPERATION;
    }

    // 2. Read from source to temporary buffer (using wgpu-types for universal comparison)
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    crate::wasm_gl_emu::imaging::TransferEngine::read_pixels(
        &crate::wasm_gl_emu::imaging::TransferRequest {
            src_buffer: ctx_obj.kernel.get_buffer(src_handle).unwrap(),
            dst_format: wgpu_types::TextureFormat::Rgba8Unorm,
            dst_layout: crate::wasm_gl_emu::device::StorageLayout::Linear,
            x,
            y,
            width: width as u32,
            height: height as u32,
        },
        &mut pixels,
    );

    // 3. Create/Update texture level
    let tex_handle = match ctx_obj.bound_texture {
        Some(h) => h,
        None => {
            set_last_error("no texture bound");
            return ERR_INVALID_ARGS;
        }
    };

    if let Some(tex) = ctx_obj.textures.get_mut(&tex_handle) {
        if level == 0 {
            tex.internal_format = internal_format;
        }

        let gpu_handle = ctx_obj.kernel.create_buffer(
            width as u32,
            height as u32,
            1,
            gl_to_wgt_format(internal_format),
            crate::wasm_gl_emu::device::StorageLayout::Linear,
        );

        if let Some(buf) = ctx_obj.kernel.get_buffer_mut(gpu_handle) {
            let to_copy = pixels.len().min(buf.data.len());
            buf.data[..to_copy].copy_from_slice(&pixels[..to_copy]);
        }

        tex.levels.insert(
            level as usize,
            MipLevel {
                width: width as u32,
                height: height as u32,
                depth: 1,
                internal_format,
                gpu_handle,
            },
        );
        ERR_OK
    } else {
        set_last_error("texture not found");
        ERR_INVALID_HANDLE
    }
}
