//! WebGPU Pipeline management

use super::adapter::with_context;
use std::borrow::Cow;
use wgpu_core::pipeline;
use wgpu_types as wgt;

/// Create a new render pipeline (simplified for Phase 1)
pub fn create_render_pipeline(
    ctx_handle: u32,
    device_handle: u32,
    vertex_module_handle: u32,
    vertex_entry: &str,
    fragment_module_handle: u32,
    fragment_entry: &str,
) -> u32 {
    with_context(ctx_handle, |ctx| {
        let device_id = match ctx.devices.get(&device_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let v_module = match ctx.shader_modules.get(&vertex_module_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let f_module = match ctx.shader_modules.get(&fragment_module_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let desc = pipeline::RenderPipelineDescriptor {
            label: None,
            layout: None, // Auto-layout
            vertex: pipeline::VertexState {
                stage: pipeline::ProgrammableStageDescriptor {
                    module: v_module,
                    entry_point: Some(Cow::Borrowed(vertex_entry)),
                    constants: Default::default(),
                    zero_initialize_workgroup_memory: true,
                },
                buffers: Cow::Borrowed(&[]),
            },
            primitive: wgt::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgt::MultisampleState::default(),
            fragment: Some(pipeline::FragmentState {
                stage: pipeline::ProgrammableStageDescriptor {
                    module: f_module,
                    entry_point: Some(Cow::Borrowed(fragment_entry)),
                    constants: Default::default(),
                    zero_initialize_workgroup_memory: true,
                },
                targets: Cow::Borrowed(&[Some(wgt::ColorTargetState {
                    format: wgt::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgt::ColorWrites::ALL,
                })]),
            }),
            multiview_mask: None,
            cache: None,
        };

        let (pipeline_id, error) = ctx
            .global
            .device_create_render_pipeline(device_id, &desc, None);
        if let Some(e) = error {
            crate::js_log(0, &format!("Failed to create render pipeline: {:?}", e));
            return super::NULL_HANDLE;
        }

        let handle = ctx.next_render_pipeline_id;
        ctx.next_render_pipeline_id += 1;
        ctx.render_pipelines.insert(handle, pipeline_id);

        handle
    })
}
