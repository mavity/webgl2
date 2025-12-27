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

/// Copy texture to buffer
pub fn command_encoder_copy_texture_to_buffer(
    ctx_handle: u32,
    encoder_handle: u32,
    source_texture_handle: u32,
    dest_buffer_handle: u32,
    dest_offset: u64,
    dest_bytes_per_row: u32,
    dest_rows_per_image: u32,
    size_width: u32,
    size_height: u32,
    size_depth: u32,
) -> u32 {
    with_context(ctx_handle, |ctx| {
        let encoder_id = match ctx.command_encoders.get(&encoder_handle) {
            Some(id) => *id,
            None => return super::WEBGPU_ERROR_INVALID_HANDLE,
        };

        let texture_id = match ctx.textures.get(&source_texture_handle) {
            Some(id) => *id,
            None => return super::WEBGPU_ERROR_INVALID_HANDLE,
        };

        let buffer_id = match ctx.buffers.get(&dest_buffer_handle) {
            Some(id) => *id,
            None => return super::WEBGPU_ERROR_INVALID_HANDLE,
        };

        let source = wgt::TexelCopyTextureInfo {
            texture: texture_id,
            mip_level: 0,
            origin: wgt::Origin3d::ZERO,
            aspect: wgt::TextureAspect::All,
        };

        let dest = wgt::TexelCopyBufferInfo {
            buffer: buffer_id,
            layout: wgt::TexelCopyBufferLayout {
                offset: dest_offset,
                bytes_per_row: if dest_bytes_per_row > 0 { Some(dest_bytes_per_row) } else { None },
                rows_per_image: if dest_rows_per_image > 0 { Some(dest_rows_per_image) } else { None },
            },
        };

        let size = wgt::Extent3d {
            width: size_width,
            height: size_height,
            depth_or_array_layers: size_depth,
        };

        if let Err(e) = ctx.global.command_encoder_copy_texture_to_buffer(
            encoder_id,
            &source,
            &dest,
            &size,
        ) {
            crate::js_log(0, &format!("Failed to copy texture to buffer: {:?}", e));
            return super::WEBGPU_ERROR_OPERATION_FAILED;
        }
        
        super::WEBGPU_SUCCESS
    })
}

/// Begin render pass (simplified for 1 color attachment)
pub fn command_encoder_begin_render_pass_1_color(
    ctx_handle: u32,
    encoder_handle: u32,
    view_handle: u32,
    load_op: u32, // 0: Load, 1: Clear
    store_op: u32, // 0: Store, 1: Discard
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
) -> u32 {
    with_context(ctx_handle, |ctx| {
        let encoder_id = match ctx.command_encoders.get(&encoder_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let view_id = match ctx.texture_views.get(&view_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let load = match load_op {
            0 => wgpu_core::command::LoadOp::Load,
            _ => wgpu_core::command::LoadOp::Clear(wgt::Color {
                r: clear_r,
                g: clear_g,
                b: clear_b,
                a: clear_a,
            }),
        };

        let store = match store_op {
            0 => wgt::StoreOp::Store,
            _ => wgt::StoreOp::Discard,
        };

        let color_attachment = wgpu_core::command::RenderPassColorAttachment {
            view: view_id,
            resolve_target: None,
            load_op: load,
            store_op: store,
            depth_slice: None,
        };

        let desc = wgpu_core::command::RenderPassDescriptor {
            label: None,
            color_attachments: vec![Some(color_attachment)].into(),
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        };

        let (mut pass, err) = ctx.global.command_encoder_begin_render_pass(encoder_id, &desc);
        if let Some(e) = err {
             crate::js_log(0, &format!("Failed to begin render pass: {:?}", e));
             return super::WEBGPU_ERROR_OPERATION_FAILED;
        }
        
        // End immediately to simulate "one-shot" pass for clearing
        if let Err(e) = ctx.global.render_pass_end(&mut pass) {
             crate::js_log(0, &format!("Failed to end render pass: {:?}", e));
             return super::WEBGPU_ERROR_OPERATION_FAILED;
        }
        
        super::WEBGPU_SUCCESS
    })
}


