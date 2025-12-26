use super::registry::{clear_last_error, get_registry, set_last_error};
use super::types::*;

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
