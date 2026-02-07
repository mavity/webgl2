use crate::webgl2_context::registry::get_registry;
use crate::webgl2_context::types::*;

pub fn ctx_create_transform_feedback(ctx: u32) -> u32 {
    let mut reg = get_registry().borrow_mut();
    if let Some(ctx) = reg.contexts.get_mut(&ctx) {
        let handle = ctx.next_transform_feedback_handle;
        ctx.next_transform_feedback_handle += 1;
        ctx.transform_feedbacks.insert(
            handle,
            TransformFeedback {
                active: false,
                paused: false,
                buffer_bindings: vec![None; 16],
            },
        );
        handle
    } else {
        0
    }
}

pub fn ctx_is_transform_feedback(ctx: u32, handle: u32) -> bool {
    let reg = get_registry().borrow();
    if let Some(ctx) = reg.contexts.get(&ctx) {
        ctx.transform_feedbacks.contains_key(&handle)
    } else {
        false
    }
}

pub fn ctx_delete_transform_feedback(ctx: u32, handle: u32) -> u32 {
    if handle == 0 {
        return ERR_OK;
    }
    let mut reg = get_registry().borrow_mut();
    if let Some(ctx) = reg.contexts.get_mut(&ctx) {
        if ctx.bound_transform_feedback == Some(handle) {
            ctx.bound_transform_feedback = Some(0);
        }
        ctx.transform_feedbacks.remove(&handle);
        ERR_OK
    } else {
        ERR_INVALID_HANDLE
    }
}

pub fn ctx_bind_transform_feedback(ctx: u32, target: u32, handle: u32) -> u32 {
    if target != GL_TRANSFORM_FEEDBACK {
        return ERR_INVALID_ENUM;
    }
    let mut reg = get_registry().borrow_mut();
    if let Some(ctx) = reg.contexts.get_mut(&ctx) {
        if handle != 0 && !ctx.transform_feedbacks.contains_key(&handle) {
            return ERR_INVALID_OPERATION;
        }
        ctx.bound_transform_feedback = Some(handle);
        ERR_OK
    } else {
        ERR_INVALID_HANDLE
    }
}

pub fn ctx_begin_transform_feedback(ctx: u32, _primitive_mode: u32) -> u32 {
    let mut reg = get_registry().borrow_mut();
    if let Some(ctx) = reg.contexts.get_mut(&ctx) {
        let tf_handle = ctx.bound_transform_feedback.unwrap_or(0);
        if let Some(tf) = ctx.transform_feedbacks.get_mut(&tf_handle) {
            if tf.active {
                return ERR_INVALID_OPERATION;
            }
            tf.active = true;
            tf.paused = false;
            // In a real implementation, we would set up the rasterizer to capture varyings here.
            // For the software emulator, we might just set a flag.
            ERR_OK
        } else {
            ERR_INVALID_OPERATION
        }
    } else {
        ERR_INVALID_HANDLE
    }
}

pub fn ctx_end_transform_feedback(ctx: u32) -> u32 {
    let mut reg = get_registry().borrow_mut();
    if let Some(ctx) = reg.contexts.get_mut(&ctx) {
        let tf_handle = ctx.bound_transform_feedback.unwrap_or(0);
        if let Some(tf) = ctx.transform_feedbacks.get_mut(&tf_handle) {
            if !tf.active {
                return ERR_INVALID_OPERATION;
            }
            tf.active = false;
            tf.paused = false;
            ERR_OK
        } else {
            ERR_INVALID_OPERATION
        }
    } else {
        ERR_INVALID_HANDLE
    }
}

pub fn ctx_pause_transform_feedback(ctx: u32) -> u32 {
    let mut reg = get_registry().borrow_mut();
    if let Some(ctx) = reg.contexts.get_mut(&ctx) {
        let tf_handle = ctx.bound_transform_feedback.unwrap_or(0);
        if let Some(tf) = ctx.transform_feedbacks.get_mut(&tf_handle) {
            if !tf.active || tf.paused {
                return ERR_INVALID_OPERATION;
            }
            tf.paused = true;
            ERR_OK
        } else {
            ERR_INVALID_OPERATION
        }
    } else {
        ERR_INVALID_HANDLE
    }
}

pub fn ctx_resume_transform_feedback(ctx: u32) -> u32 {
    let mut reg = get_registry().borrow_mut();
    if let Some(ctx) = reg.contexts.get_mut(&ctx) {
        let tf_handle = ctx.bound_transform_feedback.unwrap_or(0);
        if let Some(tf) = ctx.transform_feedbacks.get_mut(&tf_handle) {
            if !tf.active || !tf.paused {
                return ERR_INVALID_OPERATION;
            }
            tf.paused = false;
            ERR_OK
        } else {
            ERR_INVALID_OPERATION
        }
    } else {
        ERR_INVALID_HANDLE
    }
}

pub fn ctx_transform_feedback_varyings(
    ctx: u32,
    program: u32,
    varyings: Vec<String>,
    buffer_mode: u32,
) -> u32 {
    let mut reg = get_registry().borrow_mut();
    if let Some(ctx) = reg.contexts.get_mut(&ctx) {
        if let Some(p) = ctx.programs.get_mut(&program) {
            p.tf_varyings = varyings;
            p.tf_buffer_mode = buffer_mode;
            ERR_OK
        } else {
            ERR_INVALID_HANDLE
        }
    } else {
        ERR_INVALID_HANDLE
    }
}

/// Get active transform feedback varying info.
/// Returns a pointer to an ephemeral payload:
/// - bytes [ptr+0 .. ptr+3]: `size: i32`
/// - bytes [ptr+4 .. ptr+7]: `type: u32`
/// - bytes [ptr+8 .. ptr+8+len-1]: `name: [u8]`
///
/// # Ephemeral Pointer
///
/// The returned pointer is **ephemeral** and valid only until the next call into WASM
/// on the same context. JavaScript must copy the data out of linear memory immediately
/// and must not assume the payload lifetime beyond the next WASM call.
///
/// # Header
///
/// The 16 bytes immediately preceding the returned pointer contain the header:
/// - bytes [ptr-16 .. ptr-13]: `len: u32` (total payload length in bytes)
/// - bytes [ptr-12 .. ptr-1]: `reserved: 12 bytes` (zero)
///
/// Returns 0 on failure (check last error).
pub fn ctx_get_transform_feedback_varying(ctx_handle: u32, program: u32, index: u32) -> u32 {
    let mut reg = get_registry().borrow_mut();
    let ctx = match reg.contexts.get_mut(&ctx_handle) {
        Some(c) => c,
        None => return 0,
    };

    if let Some(p) = ctx.programs.get(&program) {
        if index >= p.tf_varyings.len() as u32 {
            return 0;
        }

        let name = p.tf_varyings[index as usize].clone();
        // Look up type info from linked varying info
        let (type_code, components) = p.varying_types.get(&name).cloned().unwrap_or((0, 1));

        // Map internal type info back to GL enum
        let gl_type = match (type_code, components) {
            (0, 1) => 0x1406, // FLOAT
            (0, 2) => 0x8B50, // FLOAT_VEC2
            (0, 3) => 0x8B51, // FLOAT_VEC3
            (0, 4) => 0x8B52, // FLOAT_VEC4
            (1, 1) => 0x1404, // INT
            (2, 1) => 0x1405, // UNSIGNED_INT
            _ => 0x1406,
        };

        let name_bytes = name.as_bytes();
        let payload_len = 8 + name_bytes.len() as u32;

        let ptr = if payload_len <= 128 {
            ctx.alloc_small(payload_len)
        } else {
            ctx.alloc_blob(payload_len)
        };

        unsafe {
            *(ptr as *mut i32) = 1; // Assuming size 1 for now
            *((ptr + 4) as *mut u32) = gl_type;
            let dest = std::slice::from_raw_parts_mut((ptr + 8) as *mut u8, name_bytes.len());
            dest.copy_from_slice(name_bytes);
        }

        ptr
    } else {
        0
    }
}
