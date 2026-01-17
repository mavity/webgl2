use super::registry::{clear_last_error, get_registry, set_last_error};
use super::types::*;

/// Check if object is a buffer.
pub fn ctx_is_buffer(ctx: u32, handle: u32) -> bool {
    clear_last_error();
    if handle == 0 {
        return false;
    }
    let reg = get_registry().borrow();
    if let Some(c) = reg.contexts.get(&ctx) {
        c.buffers.contains_key(&handle)
    } else {
        false
    }
}

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

    let buffer_handle = match ctx_obj.get_buffer_handle_for_target(target) {
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
        GL_BUFFER_SIZE => {
            if let Some(gb) = ctx_obj.kernel.get_buffer(buffer.gpu_handle) {
                gb.data.len() as i32
            } else {
                0
            }
        }
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
    let gpu_handle = ctx_obj.kernel.create_buffer_blob(0);
    ctx_obj.buffers.insert(
        buf_id,
        Buffer {
            gpu_handle,
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
    if let Some(b) = ctx_obj.buffers.remove(&buf) {
        ctx_obj.kernel.destroy_buffer(b.gpu_handle);
    }
    
    // Unbind from all targets
    for val in ctx_obj.buffer_bindings.values_mut() {
        if *val == Some(buf) {
            *val = None;
        }
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

/// Copy data between buffers.
pub fn ctx_copy_buffer_sub_data(
    ctx: u32,
    read_target: u32,
    write_target: u32,
    read_offset: u32,
    write_offset: u32,
    size: u32,
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

    let read_buf_handle = match ctx_obj.get_buffer_handle_for_target(read_target) {
        Some(h) => h,
        None => {
            set_last_error("no source buffer bound");
            return ERR_INVALID_OPERATION;
        }
    };
    let write_buf_handle = match ctx_obj.get_buffer_handle_for_target(write_target) {
        Some(h) => h,
        None => {
            set_last_error("no destination buffer bound");
            return ERR_INVALID_OPERATION;
        }
    };

    let src_gpu = match ctx_obj.buffers.get(&read_buf_handle) {
        Some(b) => b.gpu_handle,
        None => return ERR_INVALID_HANDLE,
    };
    let dst_gpu = match ctx_obj.buffers.get(&write_buf_handle) {
        Some(b) => b.gpu_handle,
        None => return ERR_INVALID_HANDLE,
    };

    ctx_obj.kernel.copy_blob(
        src_gpu,
        dst_gpu,
        read_offset as usize,
        write_offset as usize,
        size as usize,
    );

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

    if target == GL_ELEMENT_ARRAY_BUFFER {
        if let Some(vao) = ctx_obj.vertex_arrays.get_mut(&ctx_obj.bound_vertex_array) {
            vao.element_array_buffer = if buf == 0 { None } else { Some(buf) };
        } else {
            set_last_error("current vertex array not found");
            return ERR_INVALID_OPERATION;
        }
    } else {
        ctx_obj
            .buffer_bindings
            .insert(target, if buf == 0 { None } else { Some(buf) });
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

    let buf_handle = match ctx_obj.get_buffer_handle_for_target(target) {
        Some(h) => h,
        None => {
            set_last_error("no buffer bound to target");
            return ERR_INVALID_ARGS;
        }
    };

    let src_slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };

    if let Some(buf) = ctx_obj.buffers.get_mut(&buf_handle) {
        ctx_obj.kernel.destroy_buffer(buf.gpu_handle);
        buf.gpu_handle = ctx_obj.kernel.create_buffer_blob(len as usize);
        if let Some(gpu_buf) = ctx_obj.kernel.get_buffer_mut(buf.gpu_handle) {
            gpu_buf.data.copy_from_slice(src_slice);
        }
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

    let buf_handle = match ctx_obj.get_buffer_handle_for_target(target) {
        Some(h) => h,
        None => {
            set_last_error("no buffer bound to target");
            return ERR_INVALID_ARGS;
        }
    };

    let src_slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };

    if let Some(buf) = ctx_obj.buffers.get_mut(&buf_handle) {
        let gpu_handle = buf.gpu_handle;
        if let Some(gpu_buf) = ctx_obj.kernel.get_buffer_mut(gpu_handle) {
            if (offset as usize + len as usize) > gpu_buf.data.len() {
                set_last_error("buffer overflow");
                return ERR_INVALID_ARGS;
            }
            gpu_buf.data[offset as usize..offset as usize + len as usize].copy_from_slice(src_slice);
            ERR_OK
        } else {
            set_last_error("internal resource lost");
            ERR_INTERNAL
        }
    } else {
        set_last_error("buffer not found");
        ERR_INVALID_HANDLE
    }
}
