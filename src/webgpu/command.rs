//! WebGPU Command Encoder and Queue management

use super::adapter::with_context;
use std::num::NonZero;
use wgpu_types as wgt;

/// Create a new command encoder
pub fn create_command_encoder(ctx_handle: u32, device_handle: u32) -> u32 {
    with_context(ctx_handle, |ctx| {
        let device_id = match ctx.devices.get(&device_handle) {
            Some(id) => *id,
            None => {
                crate::error::set_error(
                    crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                    super::WEBGPU_ERROR_INVALID_HANDLE,
                    "Invalid device handle",
                );
                return super::NULL_HANDLE;
            }
        };

        let desc = wgt::CommandEncoderDescriptor { label: None };

        let (encoder_id, error) = ctx
            .global
            .device_create_command_encoder(device_id, &desc, None);
        if let Some(e) = error {
            crate::error::set_error(
                crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                super::WEBGPU_ERROR_INVALID_HANDLE,
                e.to_string(),
            );
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
            None => {
                crate::error::set_error(
                    crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                    super::WEBGPU_ERROR_INVALID_HANDLE,
                    "Invalid command encoder handle",
                );
                return super::NULL_HANDLE;
            }
        };

        let desc = wgt::CommandBufferDescriptor { label: None };

        let (buffer_id, error) = ctx.global.command_encoder_finish(encoder_id, &desc, None);
        if let Some(e) = error {
            crate::error::set_error(
                crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                super::WEBGPU_ERROR_INVALID_HANDLE,
                format!("{}: {}", e.0, e.1),
            );
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
            None => {
                crate::error::set_error(
                    crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                    super::WEBGPU_ERROR_INVALID_HANDLE,
                    "Invalid command encoder handle",
                );
                return super::NULL_HANDLE;
            }
        };

        let source_id = match ctx.buffers.get(&source_handle) {
            Some(id) => *id,
            None => {
                crate::error::set_error(
                    crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                    super::WEBGPU_ERROR_INVALID_HANDLE,
                    "Invalid source buffer handle",
                );
                return super::NULL_HANDLE;
            }
        };

        let dest_id = match ctx.buffers.get(&dest_handle) {
            Some(id) => *id,
            None => {
                crate::error::set_error(
                    crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                    super::WEBGPU_ERROR_INVALID_HANDLE,
                    "Invalid destination buffer handle",
                );
                return super::NULL_HANDLE;
            }
        };

        if let Err(e) = ctx.global.command_encoder_copy_buffer_to_buffer(
            encoder_id,
            source_id,
            source_offset,
            dest_id,
            dest_offset,
            Some(size),
        ) {
            crate::error::set_error(
                crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                super::WEBGPU_ERROR_OPERATION_FAILED,
                e,
            );
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
                crate::error::set_error(
                    crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                    super::WEBGPU_ERROR_OPERATION_FAILED,
                    format!("Submission index {}: {}", e.0, e.1),
                );
                super::WEBGPU_ERROR_OPERATION_FAILED
            }
        }
    })
}

pub struct CopyTextureToBufferConfig {
    pub source_texture_handle: u32,
    pub dest_buffer_handle: u32,
    pub dest_offset: u64,
    pub dest_bytes_per_row: u32,
    pub dest_rows_per_image: u32,
    pub size_width: u32,
    pub size_height: u32,
    pub size_depth: u32,
}

/// Copy texture to buffer
pub fn command_encoder_copy_texture_to_buffer(
    ctx_handle: u32,
    encoder_handle: u32,
    config: CopyTextureToBufferConfig,
) -> u32 {
    with_context(ctx_handle, |ctx| {
        let encoder_id = match ctx.command_encoders.get(&encoder_handle) {
            Some(id) => *id,
            None => return super::WEBGPU_ERROR_INVALID_HANDLE,
        };

        let texture_id = match ctx.textures.get(&config.source_texture_handle) {
            Some(id) => *id,
            None => return super::WEBGPU_ERROR_INVALID_HANDLE,
        };

        let buffer_id = match ctx.buffers.get(&config.dest_buffer_handle) {
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
                offset: config.dest_offset,
                bytes_per_row: if config.dest_bytes_per_row > 0 {
                    Some(config.dest_bytes_per_row)
                } else {
                    None
                },
                rows_per_image: if config.dest_rows_per_image > 0 {
                    Some(config.dest_rows_per_image)
                } else {
                    None
                },
            },
        };

        let size = wgt::Extent3d {
            width: config.size_width,
            height: config.size_height,
            depth_or_array_layers: config.size_depth,
        };

        if let Err(e) = ctx
            .global
            .command_encoder_copy_texture_to_buffer(encoder_id, &source, &dest, &size)
        {
            crate::error::set_error(
                crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                super::WEBGPU_ERROR_OPERATION_FAILED,
                e,
            );
            return super::WEBGPU_ERROR_OPERATION_FAILED;
        }

        super::WEBGPU_SUCCESS
    })
}

pub struct RenderPassConfig {
    pub view_handle: u32,
    pub load_op: u32,
    pub store_op: u32,
    pub clear_r: f64,
    pub clear_g: f64,
    pub clear_b: f64,
    pub clear_a: f64,
}

/// Begin render pass (simplified for 1 color attachment)
pub fn command_encoder_begin_render_pass_1_color(
    ctx_handle: u32,
    encoder_handle: u32,
    config: RenderPassConfig,
) -> u32 {
    // Deprecated in favor of run_render_pass, but kept for compatibility if needed
    command_encoder_run_render_pass(ctx_handle, encoder_handle, config, &[])
}

/// Run a render pass with buffered commands
pub fn command_encoder_run_render_pass(
    ctx_handle: u32,
    encoder_handle: u32,
    config: RenderPassConfig,
    commands: &[u32],
) -> u32 {
    with_context(ctx_handle, |ctx| {
        let encoder_id = match ctx.command_encoders.get(&encoder_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let view_id = match ctx.texture_views.get(&config.view_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let load = match config.load_op {
            0 => wgpu_core::command::LoadOp::Load,
            _ => wgpu_core::command::LoadOp::Clear(wgt::Color {
                r: config.clear_r,
                g: config.clear_g,
                b: config.clear_b,
                a: config.clear_a,
            }),
        };

        let store = match config.store_op {
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

        let (mut pass, err) = ctx
            .global
            .command_encoder_begin_render_pass(encoder_id, &desc);
        if let Some(e) = err {
            crate::error::set_error(
                crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                super::WEBGPU_ERROR_OPERATION_FAILED,
                e,
            );
            return super::WEBGPU_ERROR_OPERATION_FAILED;
        }

        // Execute commands
        let mut cursor = 0;
        while cursor < commands.len() {
            let op = commands[cursor];
            cursor += 1;

            match op {
                1 => {
                    // SetPipeline
                    if cursor >= commands.len() {
                        break;
                    }
                    let pipeline_handle = commands[cursor];
                    cursor += 1;
                    if let Some(id) = ctx.render_pipelines.get(&pipeline_handle) {
                        let _ = ctx.global.render_pass_set_pipeline(&mut pass, *id);
                    }
                }
                2 => {
                    // SetVertexBuffer
                    if cursor + 3 >= commands.len() {
                        break;
                    }
                    let slot = commands[cursor];
                    let buffer_handle = commands[cursor + 1];
                    let offset = commands[cursor + 2] as u64;
                    let size = commands[cursor + 3] as u64;
                    cursor += 4;

                    if let Some(id) = ctx.buffers.get(&buffer_handle) {
                        let _ = ctx.global.render_pass_set_vertex_buffer(
                            &mut pass,
                            slot,
                            *id,
                            offset,
                            NonZero::new(size),
                        );
                    }
                }
                3 => {
                    // Draw
                    if cursor + 3 >= commands.len() {
                        break;
                    }
                    let vertex_count = commands[cursor];
                    let instance_count = commands[cursor + 1];
                    let first_vertex = commands[cursor + 2];
                    let first_instance = commands[cursor + 3];
                    cursor += 4;

                    if let Err(e) = ctx.global.render_pass_draw(
                        &mut pass,
                        vertex_count,
                        instance_count,
                        first_vertex,
                        first_instance,
                    ) {
                        crate::error::set_error(
                            crate::error::ErrorSource::WebGPU(
                                crate::error::WebGPUErrorFilter::Validation,
                            ),
                            super::WEBGPU_ERROR_OPERATION_FAILED,
                            e,
                        );
                    }
                }
                4 => {
                    // SetBindGroup
                    if cursor + 1 >= commands.len() {
                        break;
                    }
                    let index = commands[cursor];
                    let bg_handle = commands[cursor + 1];
                    cursor += 2;

                    if let Some(id) = ctx.bind_groups.get(&bg_handle) {
                        if let Err(e) =
                            ctx.global
                                .render_pass_set_bind_group(&mut pass, index, Some(*id), &[])
                        {
                            crate::error::set_error(
                                crate::error::ErrorSource::WebGPU(
                                    crate::error::WebGPUErrorFilter::Validation,
                                ),
                                super::WEBGPU_ERROR_OPERATION_FAILED,
                                e,
                            );
                        }
                    }
                }
                _ => {
                    // TODO: handle properly, propagate error
                    break;
                }
            }
        }

        if let Err(e) = ctx.global.render_pass_end(&mut pass) {
            crate::error::set_error(
                crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                super::WEBGPU_ERROR_OPERATION_FAILED,
                e,
            );
            return super::WEBGPU_ERROR_OPERATION_FAILED;
        }

        super::WEBGPU_SUCCESS
    })
}
