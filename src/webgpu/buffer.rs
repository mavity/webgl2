//! WebGPU Buffer management

use super::adapter::{with_context, with_context_val};
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

        let mut size = size;
        if mapped_at_creation {
            size = (size + 3) & !3;
        }

        let desc = wgt::BufferDescriptor {
            label: None,
            size,
            usage: wgt::BufferUsages::from_bits_truncate(usage),
            mapped_at_creation,
        };

        let (buffer_id, error) = ctx.global.device_create_buffer(device_id, &desc, None);
        if let Some(e) = error {
            crate::error::set_error(
                crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                super::WEBGPU_ERROR_INVALID_HANDLE,
                e,
            );
            // TODO: Return poisoned handle
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
            crate::error::set_error(
                crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                super::WEBGPU_ERROR_INVALID_HANDLE,
                "Invalid buffer handle",
            );
            super::WEBGPU_ERROR_INVALID_HANDLE
        }
    })
}

/// Map a buffer asynchronously
pub fn buffer_map_async(
    ctx_handle: u32,
    device_handle: u32,
    buffer_handle: u32,
    mode: u32,
    offset: u64,
    size: u64,
) -> u32 {
    with_context(ctx_handle, |ctx| {
        let device_id = match ctx.devices.get(&device_handle) {
            Some(id) => *id,
            None => return super::WEBGPU_ERROR_INVALID_HANDLE,
        };

        let buffer_id = match ctx.buffers.get(&buffer_handle) {
            Some(id) => *id,
            None => return super::WEBGPU_ERROR_INVALID_HANDLE,
        };

        let host = if mode == 1 {
            // Read
            wgpu_core::device::HostMap::Read
        } else {
            // Write
            wgpu_core::device::HostMap::Write
        };

        let op = wgpu_core::resource::BufferMapOperation {
            host,
            callback: None,
        };

        match ctx
            .global
            .buffer_map_async(buffer_id, offset, Some(size), op)
        {
            Ok(_) => {
                // In a single-threaded WASM environment, we must manually poll the device
                // to process the mapping operation. wgpu-core is lazy and won't transition
                // the buffer state to "Mapped" until poll is called. Since we don't have
                // background threads, we force the update here to make the operation
                // effectively synchronous from the JS perspective.
                if let Err(e) = ctx.global.device_poll(device_id, wgt::PollType::Poll) {
                    crate::error::set_error(
                        crate::error::ErrorSource::WebGPU(
                            crate::error::WebGPUErrorFilter::Internal,
                        ),
                        super::WEBGPU_ERROR_INVALID_HANDLE,
                        e,
                    );
                    return super::WEBGPU_ERROR_INVALID_HANDLE;
                }
                super::WEBGPU_SUCCESS
            }
            Err(e) => {
                crate::error::set_error(
                    crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                    super::WEBGPU_ERROR_INVALID_HANDLE,
                    e,
                );
                super::WEBGPU_ERROR_INVALID_HANDLE
            }
        }
    })
}

/// Get mapped range of a buffer
pub fn buffer_get_mapped_range(
    ctx_handle: u32,
    buffer_handle: u32,
    offset: u64,
    size: u64,
) -> *mut u8 {
    with_context_val(ctx_handle, std::ptr::null_mut(), |ctx| {
        let buffer_id = match ctx.buffers.get(&buffer_handle) {
            Some(id) => *id,
            None => return std::ptr::null_mut(),
        };

        match ctx
            .global
            .buffer_get_mapped_range(buffer_id, offset, Some(size))
        {
            Ok((ptr, _len)) => ptr.as_ptr(),
            Err(e) => {
                crate::error::set_error(
                    crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                    super::WEBGPU_ERROR_INVALID_HANDLE,
                    e,
                );
                std::ptr::null_mut()
            }
        }
    })
}

/// Unmap a buffer
pub fn buffer_unmap(ctx_handle: u32, buffer_handle: u32) -> u32 {
    with_context(ctx_handle, |ctx| {
        let buffer_id = match ctx.buffers.get(&buffer_handle) {
            Some(id) => *id,
            None => {
                crate::error::set_error(
                    crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                    super::WEBGPU_ERROR_INVALID_HANDLE,
                    "Invalid buffer handle",
                );
                return super::WEBGPU_ERROR_INVALID_HANDLE;
            }
        };

        match ctx.global.buffer_unmap(buffer_id) {
            Ok(_) => super::WEBGPU_SUCCESS,
            Err(e) => {
                crate::error::set_error(
                    crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                    super::WEBGPU_ERROR_INVALID_HANDLE,
                    e,
                );
                super::WEBGPU_ERROR_INVALID_HANDLE
            }
        }
    })
}
