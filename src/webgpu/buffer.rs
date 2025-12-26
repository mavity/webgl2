//! WebGPU Buffer management

use super::adapter::with_context;
use wgpu_types as wgt;

/// Create a new buffer
pub fn create_buffer(
    ctx_handle: u32,
    device_handle: u32,
    size: u64,
    usage: u32,
    mapped_at_creation: bool,
) -> u32 {
    with_context(ctx_handle, |ctx| {
        let device_id = match ctx.devices.get(&device_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let desc = wgt::BufferDescriptor {
            label: None,
            size,
            usage: wgt::BufferUsages::from_bits_truncate(usage),
            mapped_at_creation,
        };

        let (buffer_id, error) = ctx.global.device_create_buffer(device_id, &desc, None);
        if let Some(e) = error {
            crate::js_log(0, &format!("Failed to create buffer: {:?}", e));
            return super::NULL_HANDLE;
        }

        let handle = ctx.next_buffer_id;
        ctx.next_buffer_id += 1;
        ctx.buffers.insert(handle, buffer_id);

        handle
    })
}

/// Destroy a buffer
pub fn destroy_buffer(ctx_handle: u32, buffer_handle: u32) -> u32 {
    with_context(ctx_handle, |ctx| {
        if let Some(id) = ctx.buffers.remove(&buffer_handle) {
            ctx.global.buffer_destroy(id);
            super::WEBGPU_SUCCESS
        } else {
            super::WEBGPU_ERROR_INVALID_HANDLE
        }
    })
}
