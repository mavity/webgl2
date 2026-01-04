//! WebGPU Bind Group management

use super::adapter::with_context;
use wgpu_types as wgt;

/// Create a bind group layout
pub fn create_bind_group_layout(ctx_handle: u32, device_handle: u32, entries_data: &[u32]) -> u32 {
    with_context(ctx_handle, |ctx| {
        let device_id = match ctx.devices.get(&device_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let mut entries = Vec::new();
        let mut cursor = 0;

        // Format: [count, binding, visibility, type, ...]
        if cursor < entries_data.len() {
            let count = entries_data[cursor];
            cursor += 1;

            for _ in 0..count {
                if cursor + 3 > entries_data.len() {
                    break;
                }
                let binding = entries_data[cursor];
                let visibility = wgt::ShaderStages::from_bits_truncate(entries_data[cursor + 1]);
                let ty_id = entries_data[cursor + 2];
                cursor += 3;

                let ty = match ty_id {
                    0 => wgt::BindingType::Buffer {
                        // Uniform
                        ty: wgt::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    1 => wgt::BindingType::Texture {
                        sample_type: wgt::TextureSampleType::Float { filterable: true },
                        view_dimension: wgt::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    2 => wgt::BindingType::Sampler(wgt::SamplerBindingType::Filtering),
                    _ => wgt::BindingType::Buffer {
                        ty: wgt::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                };

                entries.push(wgt::BindGroupLayoutEntry {
                    binding,
                    visibility,
                    ty,
                    count: None,
                });
            }
        }

        let desc = wgpu_core::binding_model::BindGroupLayoutDescriptor {
            label: None,
            entries: std::borrow::Cow::Owned(entries),
        };

        let (layout_id, error) = ctx
            .global
            .device_create_bind_group_layout(device_id, &desc, None);

        if let Some(e) = error {
            crate::error::set_error(
                crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                super::WEBGPU_ERROR_VALIDATION,
                format!("Failed to create bind group layout: {}", e),
            );
            return super::NULL_HANDLE;
        }

        let handle = ctx.next_bind_group_layout_id;
        ctx.next_bind_group_layout_id += 1;
        ctx.bind_group_layouts.insert(handle, layout_id);

        handle
    })
}

/// Create a bind group
pub fn create_bind_group(
    ctx_handle: u32,
    device_handle: u32,
    layout_handle: u32,
    entries_data: &[u32],
) -> u32 {
    with_context(ctx_handle, |ctx| {
        let device_id = match ctx.devices.get(&device_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let layout_id = match ctx.bind_group_layouts.get(&layout_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let mut entries = Vec::new();
        let mut cursor = 0;

        // Format: [count, binding, resource_type, resource_handle, ...]
        if cursor < entries_data.len() {
            let count = entries_data[cursor];
            cursor += 1;

            for _ in 0..count {
                if cursor + 3 > entries_data.len() {
                    break;
                }
                let binding = entries_data[cursor];
                let res_type = entries_data[cursor + 1];
                let res_handle = entries_data[cursor + 2];
                cursor += 3;

                let resource = match res_type {
                    0 => {
                        // Buffer
                        if let Some(id) = ctx.buffers.get(&res_handle) {
                            wgpu_core::binding_model::BindingResource::Buffer(
                                wgpu_core::binding_model::BufferBinding {
                                    buffer: *id,
                                    offset: 0,
                                    size: None,
                                },
                            )
                        } else {
                            continue;
                        }
                    }
                    1 => {
                        // TextureView
                        if let Some(id) = ctx.texture_views.get(&res_handle) {
                            wgpu_core::binding_model::BindingResource::TextureView(*id)
                        } else {
                            continue;
                        }
                    }
                    2 => {
                        // Sampler
                        if let Some(id) = ctx.samplers.get(&res_handle) {
                            wgpu_core::binding_model::BindingResource::Sampler(*id)
                        } else {
                            // Create a default sampler if handle is 0 or invalid?
                            // For now, assume we have samplers.
                            // Wait, I haven't implemented createSampler yet.
                            // Let's assume handle 0 is a default sampler if needed, or fail.
                            continue;
                        }
                    }
                    _ => continue,
                };

                entries.push(wgpu_core::binding_model::BindGroupEntry { binding, resource });
            }
        }

        let desc = wgpu_core::binding_model::BindGroupDescriptor {
            label: None,
            layout: layout_id,
            entries: std::borrow::Cow::Owned(entries),
        };

        let (bg_id, error) = ctx.global.device_create_bind_group(device_id, &desc, None);

        if let Some(e) = error {
            crate::error::set_error(
                crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                super::WEBGPU_ERROR_VALIDATION,
                format!("Failed to create bind group: {}", e),
            );
            return super::NULL_HANDLE;
        }

        let handle = ctx.next_bind_group_id;
        ctx.next_bind_group_id += 1;
        ctx.bind_groups.insert(handle, bg_id);

        handle
    })
}
