//! WebGPU Command Encoder and Queue management

use super::adapter::with_context;
use wgpu_types as wgt;

/// Create a new command encoder
pub fn create_command_encoder(ctx_handle: u32, device_handle: u32) -> u32 {
    with_context(ctx_handle, |ctx| {
        let device_id = match ctx.devices.get(&device_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let desc = wgt::CommandEncoderDescriptor { label: None };

        let (encoder_id, error) = ctx
            .global
            .device_create_command_encoder(device_id, &desc, None);
        if let Some(e) = error {
            crate::js_log(0, &format!("Failed to create command encoder: {:?}", e));
            return super::NULL_HANDLE;
        }

        let handle = ctx.next_command_encoder_id;
        ctx.next_command_encoder_id += 1;
        ctx.command_encoders.insert(handle, encoder_id);

        handle
    })
}

/// Finish encoding and return a command buffer
pub fn command_encoder_finish(ctx_handle: u32, encoder_handle: u32) -> u32 {
    with_context(ctx_handle, |ctx| {
        let encoder_id = match ctx.command_encoders.remove(&encoder_handle) {
            Some(id) => id,
            None => return super::NULL_HANDLE,
        };

        let desc = wgt::CommandBufferDescriptor { label: None };

        let (buffer_id, error) = ctx.global.command_encoder_finish(encoder_id, &desc, None);
        if let Some(e) = error {
            crate::js_log(0, &format!("Failed to finish command encoder: {:?}", e));
            return super::NULL_HANDLE;
        }

        let handle = ctx.next_command_buffer_id;
        ctx.next_command_buffer_id += 1;
        ctx.command_buffers.insert(handle, buffer_id);

        handle
    })
}

/// Copy buffer to buffer
pub fn command_encoder_copy_buffer_to_buffer(
    ctx_handle: u32,
    encoder_handle: u32,
    source_handle: u32,
    source_offset: u64,
    dest_handle: u32,
    dest_offset: u64,
    size: u64,
) -> u32 {
    with_context(ctx_handle, |ctx| {
        let encoder_id = match ctx.command_encoders.get(&encoder_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let source_id = match ctx.buffers.get(&source_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let dest_id = match ctx.buffers.get(&dest_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        if let Err(e) = ctx.global.command_encoder_copy_buffer_to_buffer(
            encoder_id,
            source_id,
            source_offset,
            dest_id,
            dest_offset,
            Some(size),
        ) {
            crate::js_log(0, &format!("Failed to copy buffer to buffer: {:?}", e));
            return super::NULL_HANDLE;
        }
        
        0 // Success
    })
}

/// Submit command buffers to the queue
pub fn queue_submit(ctx_handle: u32, device_handle: u32, cb_handles: &[u32]) -> u32 {
    with_context(ctx_handle, |ctx| {
        let device_id = match ctx.devices.get(&device_handle) {
            Some(id) => *id,
            None => return super::WEBGPU_ERROR_INVALID_HANDLE,
        };

        let mut cb_ids = Vec::with_capacity(cb_handles.len());
        for &h in cb_handles {
            if let Some(id) = ctx.command_buffers.remove(&h) {
                cb_ids.push(id);
            } else {
                return super::WEBGPU_ERROR_INVALID_HANDLE;
            }
        }

        let queue_id = match ctx.queues.get(&device_handle) {
            Some(id) => *id,
            None => return super::WEBGPU_ERROR_OPERATION_FAILED,
        };

        match ctx.global.queue_submit(queue_id, &cb_ids) {
            Ok(_) => {
                // Synchronous execution model: poll immediately
                let _ = ctx.global.device_poll(
                    device_id,
                    wgt::PollType::Wait {
                        submission_index: None,
                        timeout: None,
                    },
                );
                super::WEBGPU_SUCCESS
            }
            Err(e) => {
                crate::js_log(0, &format!("Failed to submit queue: {:?}", e));
                super::WEBGPU_ERROR_OPERATION_FAILED
            }
        }
    })
}
