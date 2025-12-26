use super::registry::{get_registry, set_last_error, clear_last_error};
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

    let bound_buffer = ctx_obj.bound_array_buffer;

    if let Some(vao) = ctx_obj.vertex_arrays.get_mut(&ctx_obj.bound_vertex_array) {
        if (index as usize) < vao.attributes.len() {
            let attr = &mut vao.attributes[index as usize];
            attr.size = size;
            attr.type_ = type_;
            attr.normalized = normalized;
            attr.stride = stride;
            attr.offset = offset;
            attr.buffer = bound_buffer;
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
            vao.attributes[index as usize].default_value = [v0, v1, v2, v3];
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
