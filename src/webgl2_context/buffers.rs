use super::registry::{clear_last_error, get_registry, set_last_error};
use super::types::*;

/// Get buffer parameter.
pub fn ctx_get_buffer_parameter(ctx: u32, target: u32, pname: u32) -> i32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return -1;
        }
    };

    let buffer_handle = match target {
        GL_ARRAY_BUFFER => ctx_obj.bound_array_buffer,
        GL_ELEMENT_ARRAY_BUFFER => {
            if let Some(vao) = ctx_obj.vertex_arrays.get(&ctx_obj.bound_vertex_array) {
                vao.element_array_buffer
            } else {
                None
            }
        }
        _ => {
            set_last_error("invalid buffer target");
            return -1;
        }
    };

    let buffer_handle = match buffer_handle {
        Some(h) => h,
        None => {
            set_last_error("no buffer bound to target");
            return -1;
        }
    };

    let buffer = match ctx_obj.buffers.get(&buffer_handle) {
        Some(b) => b,
        None => {
            set_last_error("buffer not found");
            return -1;
        }
    };

    match pname {
        GL_BUFFER_SIZE => buffer.data.len() as i32,
        _ => {
            set_last_error("invalid parameter name");
            -1
        }
    }
}

// ============================================================================
// Buffer Operations
// ============================================================================

/// Create a buffer in the given context.
pub fn ctx_create_buffer(ctx: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return 0;
        }
    };
    let buf_id = ctx_obj.allocate_buffer_handle();
    ctx_obj.buffers.insert(
        buf_id,
        Buffer {
            data: Vec::new(),
            usage: 0,
        },
    );
    buf_id
}

/// Delete a buffer.
pub fn ctx_delete_buffer(ctx: u32, buf: u32) -> u32 {
    clear_last_error();
    if buf == INVALID_HANDLE {
        return ERR_OK; // Deleting 0 is a no-op
    }
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };
    ctx_obj.buffers.remove(&buf);
    if ctx_obj.bound_array_buffer == Some(buf) {
        ctx_obj.bound_array_buffer = None;
    }

    // Unbind from all VAOs
    for vao in ctx_obj.vertex_arrays.values_mut() {
        if vao.element_array_buffer == Some(buf) {
            vao.element_array_buffer = None;
        }
        for attr in &mut vao.attributes {
            if attr.buffer == Some(buf) {
                attr.buffer = None;
            }
        }
    }
    ERR_OK
}

/// Bind a buffer to a target.
pub fn ctx_bind_buffer(ctx: u32, target: u32, buf: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if buf != 0 && !ctx_obj.buffers.contains_key(&buf) {
        set_last_error("buffer not found");
        return ERR_INVALID_HANDLE;
    }

    match target {
        GL_ARRAY_BUFFER => ctx_obj.bound_array_buffer = if buf == 0 { None } else { Some(buf) },
        GL_ELEMENT_ARRAY_BUFFER => {
            if let Some(vao) = ctx_obj.vertex_arrays.get_mut(&ctx_obj.bound_vertex_array) {
                vao.element_array_buffer = if buf == 0 { None } else { Some(buf) };
            } else {
                // Should not happen if bound_vertex_array is valid
                set_last_error("current vertex array not found");
                return ERR_INVALID_OPERATION;
            }
        }
        _ => {
            set_last_error("invalid buffer target");
            return ERR_INVALID_ARGS;
        }
    }
    ERR_OK
}

/// Upload data to the bound buffer.
pub fn ctx_buffer_data(ctx: u32, target: u32, ptr: u32, len: u32, usage: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    let buf_handle = match target {
        GL_ARRAY_BUFFER => ctx_obj.bound_array_buffer,
        GL_ELEMENT_ARRAY_BUFFER => {
            if let Some(vao) = ctx_obj.vertex_arrays.get(&ctx_obj.bound_vertex_array) {
                vao.element_array_buffer
            } else {
                None
            }
        }
        _ => {
            set_last_error("invalid buffer target");
            return ERR_INVALID_ARGS;
        }
    };

    let buf_handle = match buf_handle {
        Some(h) => h,
        None => {
            set_last_error("no buffer bound to target");
            return ERR_INVALID_ARGS;
        }
    };

    let src_slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };

    if len >= 4 {
        crate::js_log(0, &format!("Buffer Data Upload: len={}, first 4 bytes: {:02x} {:02x} {:02x} {:02x}", len, src_slice[0], src_slice[1], src_slice[2], src_slice[3]));
    }

    if let Some(buf) = ctx_obj.buffers.get_mut(&buf_handle) {
        buf.data = src_slice.to_vec();
        buf.usage = usage;
        ERR_OK
    } else {
        set_last_error("buffer not found");
        ERR_INVALID_HANDLE
    }
}

/// Update a subset of the bound buffer's data.
pub fn ctx_buffer_sub_data(ctx: u32, target: u32, offset: u32, ptr: u32, len: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    let buf_handle = match target {
        GL_ARRAY_BUFFER => ctx_obj.bound_array_buffer,
        GL_ELEMENT_ARRAY_BUFFER => {
            if let Some(vao) = ctx_obj.vertex_arrays.get(&ctx_obj.bound_vertex_array) {
                vao.element_array_buffer
            } else {
                None
            }
        }
        _ => {
            set_last_error("invalid buffer target");
            return ERR_INVALID_ARGS;
        }
    };

    let buf_handle = match buf_handle {
        Some(h) => h,
        None => {
            set_last_error("no buffer bound to target");
            return ERR_INVALID_ARGS;
        }
    };

    let src_slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };

    if let Some(buf) = ctx_obj.buffers.get_mut(&buf_handle) {
        if (offset as usize + len as usize) > buf.data.len() {
            set_last_error("buffer overflow");
            return ERR_INVALID_ARGS;
        }
        buf.data[offset as usize..offset as usize + len as usize].copy_from_slice(src_slice);
        ERR_OK
    } else {
        set_last_error("buffer not found");
        ERR_INVALID_HANDLE
    }
}
