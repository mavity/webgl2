use super::registry::{clear_last_error, get_registry, set_last_error};
use super::types::*;

// ============================================================================
// Vertex Array Object Operations
// ============================================================================

/// Create a vertex array object.
pub fn ctx_create_vertex_array(ctx: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return 0;
        }
    };
    let vao_id = ctx_obj.allocate_vertex_array_handle();
    ctx_obj.vertex_arrays.insert(vao_id, VertexArray::default());
    vao_id
}

/// Delete a vertex array object.
pub fn ctx_delete_vertex_array(ctx: u32, vao: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if vao == 0 {
        return ERR_OK; // Silent ignore for 0
    }

    if ctx_obj.vertex_arrays.remove(&vao).is_some() {
        // If deleted VAO is bound, bind default VAO (0)
        if ctx_obj.bound_vertex_array == vao {
            ctx_obj.bound_vertex_array = 0;
        }
        ERR_OK
    } else {
        set_last_error("vertex array not found");
        ERR_INVALID_HANDLE
    }
}

/// Bind a vertex array object.
pub fn ctx_bind_vertex_array(ctx: u32, vao: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    if ctx_obj.vertex_arrays.contains_key(&vao) {
        ctx_obj.bound_vertex_array = vao;
        ERR_OK
    } else {
        set_last_error("vertex array not found");
        ERR_INVALID_HANDLE
    }
}

/// Is vertex array.
pub fn ctx_is_vertex_array(ctx: u32, vao: u32) -> u32 {
    clear_last_error();
    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => return 0,
    };
    // 0 is not a user VAO object in WebGL terms usually, but here we track it.
    // WebGL spec says isVertexArray returns false for deleted VAOs and 0.
    if vao == 0 {
        return 0;
    }
    if ctx_obj.vertex_arrays.contains_key(&vao) {
        1
    } else {
        0
    }
}

/// Enable vertex attribute array.
pub fn ctx_enable_vertex_attrib_array(ctx: u32, index: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if let Some(vao) = ctx_obj.vertex_arrays.get_mut(&ctx_obj.bound_vertex_array) {
        if (index as usize) < vao.attributes.len() {
            vao.attributes[index as usize].enabled = true;
            ERR_OK
        } else {
            set_last_error("index out of range");
            ERR_INVALID_ARGS
        }
    } else {
        set_last_error("current vertex array not found");
        ERR_INVALID_OPERATION
    }
}

/// Disable vertex attribute array.
pub fn ctx_disable_vertex_attrib_array(ctx: u32, index: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if let Some(vao) = ctx_obj.vertex_arrays.get_mut(&ctx_obj.bound_vertex_array) {
        if (index as usize) < vao.attributes.len() {
            vao.attributes[index as usize].enabled = false;
            ERR_OK
        } else {
            set_last_error("index out of range");
            ERR_INVALID_ARGS
        }
    } else {
        set_last_error("current vertex array not found");
        ERR_INVALID_OPERATION
    }
}

/// Vertex attribute pointer.
fn get_type_size(type_: u32) -> u32 {
    match type_ {
        GL_BYTE | GL_UNSIGNED_BYTE => 1,
        GL_SHORT | GL_UNSIGNED_SHORT | GL_HALF_FLOAT => 2,
        GL_INT | GL_UNSIGNED_INT | GL_FLOAT => 4,
        _ => 1,
    }
}

pub fn ctx_vertex_attrib_pointer(
    ctx: u32,
    index: u32,
    size: i32,
    type_: u32,
    normalized: bool,
    stride: i32,
    offset: u32,
) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    let bound_buffer = ctx_obj.get_buffer_handle_for_target(GL_ARRAY_BUFFER);

    if bound_buffer.is_none() && offset != 0 {
        set_last_error("offset is non-zero but no buffer is bound to ARRAY_BUFFER");
        ctx_obj.set_error(GL_INVALID_OPERATION);
        return ERR_GL;
    }

    if stride < 0 || stride > 255 {
        set_last_error("stride out of range");
        ctx_obj.set_error(GL_INVALID_VALUE);
        return ERR_GL;
    }

    if size < 1 || size > 4 {
        set_last_error("size out of range");
        ctx_obj.set_error(GL_INVALID_VALUE);
        return ERR_GL;
    }

    if let Some(vao) = ctx_obj.vertex_arrays.get_mut(&ctx_obj.bound_vertex_array) {
        if (index as usize) < vao.attributes.len() {
            let attr = &mut vao.attributes[index as usize];
            attr.size = size;
            attr.type_ = type_;
            attr.normalized = normalized;
            attr.stride = stride;
            attr.offset = offset;
            attr.buffer = bound_buffer;
            attr.is_integer = false;
            ERR_OK
        } else {
            set_last_error("index out of range");
            ERR_INVALID_ARGS
        }
    } else {
        set_last_error("current vertex array not found");
        ERR_INVALID_OPERATION
    }
}

/// Vertex attribute integer pointer.
pub fn ctx_vertex_attrib_ipointer(
    ctx: u32,
    index: u32,
    size: i32,
    type_: u32,
    stride: i32,
    offset: u32,
) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    let bound_buffer = ctx_obj.get_buffer_handle_for_target(GL_ARRAY_BUFFER);

    if bound_buffer.is_none() && offset != 0 {
        set_last_error("offset is non-zero but no buffer is bound to ARRAY_BUFFER");
        ctx_obj.set_error(GL_INVALID_OPERATION);
        return ERR_GL;
    }

    if stride < 0 || stride > 255 {
        set_last_error("stride out of range");
        ctx_obj.set_error(GL_INVALID_VALUE);
        return ERR_GL;
    }

    if size < 1 || size > 4 {
        set_last_error("size out of range");
        ctx_obj.set_error(GL_INVALID_VALUE);
        return ERR_GL;
    }

    // Check type and alignment (IPointer specific)
    match type_ {
        GL_BYTE | GL_UNSIGNED_BYTE | GL_SHORT | GL_UNSIGNED_SHORT | GL_INT | GL_UNSIGNED_INT => {
            let type_size = get_type_size(type_);
            if offset % type_size != 0 {
                set_last_error("offset must be a multiple of the type size");
                ctx_obj.set_error(GL_INVALID_OPERATION);
                return ERR_GL;
            }
            if stride > 0 && (stride as u32 % type_size != 0) {
                set_last_error("stride must be a multiple of the type size");
                ctx_obj.set_error(GL_INVALID_OPERATION);
                return ERR_GL;
            }
        }
        _ => {
            set_last_error("invalid type for vertexAttribIPointer");
            ctx_obj.set_error(GL_INVALID_ENUM);
            return ERR_GL;
        }
    }

    if let Some(vao) = ctx_obj.vertex_arrays.get_mut(&ctx_obj.bound_vertex_array) {
        if (index as usize) < vao.attributes.len() {
            let attr = &mut vao.attributes[index as usize];
            attr.size = size;
            attr.type_ = type_;
            attr.normalized = false; // Integer attributes are never normalized
            attr.stride = stride;
            attr.offset = offset;
            attr.buffer = bound_buffer;
            attr.is_integer = true;
            ERR_OK
        } else {
            set_last_error("index out of range");
            ERR_INVALID_ARGS
        }
    } else {
        set_last_error("current vertex array not found");
        ERR_INVALID_OPERATION
    }
}

/// Vertex attribute divisor.
pub fn ctx_vertex_attrib_divisor(ctx: u32, index: u32, divisor: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if let Some(vao) = ctx_obj.vertex_arrays.get_mut(&ctx_obj.bound_vertex_array) {
        if (index as usize) < vao.attributes.len() {
            let attr = &mut vao.attributes[index as usize];
            attr.divisor = divisor;
            ERR_OK
        } else {
            set_last_error("index out of range");
            ERR_INVALID_ARGS
        }
    } else {
        set_last_error("current vertex array not found");
        ERR_INVALID_OPERATION
    }
}

/// Set vertex attribute default value (1f).
pub fn ctx_vertex_attrib1f(ctx: u32, index: u32, v0: f32) -> u32 {
    ctx_vertex_attrib4f(ctx, index, v0, 0.0, 0.0, 1.0)
}

/// Set vertex attribute default value (2f).
pub fn ctx_vertex_attrib2f(ctx: u32, index: u32, v0: f32, v1: f32) -> u32 {
    ctx_vertex_attrib4f(ctx, index, v0, v1, 0.0, 1.0)
}

/// Set vertex attribute default value (3f).
pub fn ctx_vertex_attrib3f(ctx: u32, index: u32, v0: f32, v1: f32, v2: f32) -> u32 {
    ctx_vertex_attrib4f(ctx, index, v0, v1, v2, 1.0)
}

/// Set vertex attribute default value (4f).
pub fn ctx_vertex_attrib4f(ctx: u32, index: u32, v0: f32, v1: f32, v2: f32, v3: f32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if let Some(vao) = ctx_obj.vertex_arrays.get_mut(&ctx_obj.bound_vertex_array) {
        if (index as usize) < vao.attributes.len() {
            vao.attributes[index as usize].default_value =
                [v0.to_bits(), v1.to_bits(), v2.to_bits(), v3.to_bits()];
            vao.attributes[index as usize].is_integer = false;
            vao.attributes[index as usize].current_value_type = GL_FLOAT;
            ERR_OK
        } else {
            ctx_obj.gl_error = GL_INVALID_VALUE;
            ERR_GL
        }
    } else {
        set_last_error("current vertex array not found");
        ERR_INVALID_OPERATION
    }
}

/// Set vertex attribute default value (I4i).
pub fn ctx_vertex_attrib_i4i(ctx: u32, index: u32, v0: i32, v1: i32, v2: i32, v3: i32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if let Some(vao) = ctx_obj.vertex_arrays.get_mut(&ctx_obj.bound_vertex_array) {
        if (index as usize) < vao.attributes.len() {
            vao.attributes[index as usize].default_value =
                [v0 as u32, v1 as u32, v2 as u32, v3 as u32];
            vao.attributes[index as usize].is_integer = true;
            vao.attributes[index as usize].current_value_type = GL_INT;
            ERR_OK
        } else {
            ctx_obj.gl_error = GL_INVALID_VALUE;
            ERR_GL
        }
    } else {
        set_last_error("current vertex array not found");
        ERR_INVALID_OPERATION
    }
}

/// Set vertex attribute default value (I4ui).
pub fn ctx_vertex_attrib_i4ui(ctx: u32, index: u32, v0: u32, v1: u32, v2: u32, v3: u32) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if let Some(vao) = ctx_obj.vertex_arrays.get_mut(&ctx_obj.bound_vertex_array) {
        if (index as usize) < vao.attributes.len() {
            vao.attributes[index as usize].default_value = [v0, v1, v2, v3];
            vao.attributes[index as usize].is_integer = true;
            vao.attributes[index as usize].current_value_type = GL_UNSIGNED_INT;
            ERR_OK
        } else {
            ctx_obj.gl_error = GL_INVALID_VALUE;
            ERR_GL
        }
    } else {
        set_last_error("current vertex array not found");
        ERR_INVALID_OPERATION
    }
}

// Force rebuild
/// Get vertex attribute parameter.
pub fn ctx_get_vertex_attrib_v4(
    ctx: u32,
    index: u32,
    pname: u32,
    dest_ptr: u32,
    dest_len: u32,
) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let ctx_obj = match reg.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    if index >= 16 {
        ctx_obj.gl_error = GL_INVALID_VALUE;
        return ERR_GL;
    }

    // Debug: check index
    // return index + 100;

    let bound_vao = ctx_obj.bound_vertex_array;
    let vao = match ctx_obj.vertex_arrays.get(&bound_vao) {
        Some(v) => v,
        None => {
            set_last_error("current vertex array not found");
            return ERR_INVALID_OPERATION;
        }
    };

    // Double check index against VAO size
    if (index as usize) >= vao.attributes.len() {
        set_last_error("index out of range (VAO check)");
        return ERR_INVALID_ARGS;
    }

    let attr = &vao.attributes[index as usize];

    match pname {
        GL_VERTEX_ATTRIB_ARRAY_ENABLED => {
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = if attr.enabled { 1 } else { 0 };
            ERR_OK
        }
        GL_VERTEX_ATTRIB_ARRAY_SIZE => {
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = attr.size;
            ERR_OK
        }
        GL_VERTEX_ATTRIB_ARRAY_STRIDE => {
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = attr.stride;
            ERR_OK
        }
        GL_VERTEX_ATTRIB_ARRAY_TYPE => {
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = attr.type_ as i32;
            ERR_OK
        }
        GL_VERTEX_ATTRIB_ARRAY_NORMALIZED => {
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = if attr.normalized { 1 } else { 0 };
            ERR_OK
        }
        GL_VERTEX_ATTRIB_ARRAY_INTEGER => {
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = if attr.is_integer { 1 } else { 0 };
            ERR_OK
        }
        GL_VERTEX_ATTRIB_ARRAY_DIVISOR => {
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = attr.divisor as i32;
            ERR_OK
        }
        GL_VERTEX_ATTRIB_ARRAY_POINTER => {
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = attr.offset as i32;
            ERR_OK
        }
        GL_VERTEX_ATTRIB_ARRAY_BUFFER_BINDING => {
            if dest_len < 4 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut i32, 1) };
            dest[0] = attr.buffer.unwrap_or(0) as i32;
            ERR_OK
        }
        GL_CURRENT_VERTEX_ATTRIB => {
            if dest_len < 16 {
                return ERR_INVALID_ARGS;
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut u32, 5) };
            dest[0] = attr.default_value[0];
            dest[1] = attr.default_value[1];
            dest[2] = attr.default_value[2];
            dest[3] = attr.default_value[3];
            if dest_len >= 20 {
                dest[4] = attr.current_value_type;
            }
            ERR_OK
        }
        _ => {
            if ctx_obj.gl_error == GL_NO_ERROR {
                ctx_obj.gl_error = GL_INVALID_ENUM;
            }
            ERR_GL
        }
    }
}
