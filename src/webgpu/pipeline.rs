//! WebGPU Pipeline management

use super::adapter::with_context;
use std::borrow::Cow;
use wgpu_core::pipeline;
use wgpu_types as wgt;

pub struct RenderPipelineConfig<'a> {
    pub vertex_module_handle: u32,
    pub vertex_entry: &'a str,
    pub fragment_module_handle: u32,
    pub fragment_entry: &'a str,
    pub layout_data: &'a [u32],
    pub pipeline_layout_handle: u32,
    pub primitive_topology: u32,
    pub depth_format: u32,
    pub depth_write_enabled: bool,
    pub depth_compare: u32,
    pub blend_enabled: bool,
    pub color_blend_src: u32,
    pub color_blend_dst: u32,
    pub color_blend_op: u32,
    pub alpha_blend_src: u32,
    pub alpha_blend_dst: u32,
    pub alpha_blend_op: u32,
}

/// Create a new render pipeline
pub fn create_render_pipeline(
    ctx_handle: u32,
    device_handle: u32,
    config: RenderPipelineConfig,
) -> u32 {
    with_context(ctx_handle, |ctx| {
        let device_id = match ctx.devices.get(&device_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let v_module = match ctx.shader_modules.get(&config.vertex_module_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let f_module = match ctx.shader_modules.get(&config.fragment_module_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let layout_id = if config.pipeline_layout_handle != 0 {
            match ctx.pipeline_layouts.get(&config.pipeline_layout_handle) {
                Some(id) => Some(*id),
                None => {
                    return super::NULL_HANDLE;
                }
            }
        } else {
            None
        };

        // Parse vertex buffer layout
        let mut vertex_buffers = Vec::new();
        let mut cursor = 0;
        if cursor < config.layout_data.len() {
            let count = config.layout_data[cursor];
            cursor += 1;

            for _ in 0..count {
                if cursor + 3 > config.layout_data.len() {
                    break;
                }
                let array_stride = config.layout_data[cursor] as u64;
                let step_mode = if config.layout_data[cursor + 1] == 1 {
                    wgt::VertexStepMode::Instance
                } else {
                    wgt::VertexStepMode::Vertex
                };
                let attr_count = config.layout_data[cursor + 2];
                cursor += 3;

                let mut attributes = Vec::new();
                for _ in 0..attr_count {
                    if cursor + 3 > config.layout_data.len() {
                        break;
                    }
                    let format_id = config.layout_data[cursor];
                    let offset = config.layout_data[cursor + 1] as u64;
                    let shader_location = config.layout_data[cursor + 2];
                    cursor += 3;

                    let format = match format_id {
                        1 => wgt::VertexFormat::Float32x3,
                        2 => wgt::VertexFormat::Float32x2,
                        3 => wgt::VertexFormat::Float32x4,
                        _ => wgt::VertexFormat::Float32x3, // Default/Fallback
                    };

                    attributes.push(wgt::VertexAttribute {
                        format,
                        offset,
                        shader_location,
                    });
                }

                vertex_buffers.push(wgpu_core::pipeline::VertexBufferLayout {
                    array_stride,
                    step_mode,
                    attributes: Cow::Owned(attributes),
                });
            }
        }

        let primitive = wgt::PrimitiveState {
            topology: match config.primitive_topology {
                1 => wgt::PrimitiveTopology::PointList,
                2 => wgt::PrimitiveTopology::LineList,
                3 => wgt::PrimitiveTopology::LineStrip,
                4 => wgt::PrimitiveTopology::TriangleList,
                5 => wgt::PrimitiveTopology::TriangleStrip,
                _ => wgt::PrimitiveTopology::TriangleList,
            },
            ..Default::default()
        };

        let depth_stencil = if config.depth_format != 0 {
            let format = match config.depth_format {
                1 => wgt::TextureFormat::Depth32Float,
                2 => wgt::TextureFormat::Depth24Plus,
                3 => wgt::TextureFormat::Depth24PlusStencil8,
                _ => wgt::TextureFormat::Depth32Float,
            };

            let compare = match config.depth_compare {
                1 => wgt::CompareFunction::Never,
                2 => wgt::CompareFunction::Less,
                3 => wgt::CompareFunction::Equal,
                4 => wgt::CompareFunction::LessEqual,
                5 => wgt::CompareFunction::Greater,
                6 => wgt::CompareFunction::NotEqual,
                7 => wgt::CompareFunction::GreaterEqual,
                8 => wgt::CompareFunction::Always,
                _ => wgt::CompareFunction::Less,
            };

            Some(wgt::DepthStencilState {
                format,
                depth_write_enabled: config.depth_write_enabled,
                depth_compare: compare,
                stencil: wgt::StencilState::default(),
                bias: wgt::DepthBiasState::default(),
            })
        } else {
            None
        };

        let blend = if config.blend_enabled {
            let map_factor = |f| match f {
                0 => wgt::BlendFactor::Zero,
                1 => wgt::BlendFactor::One,
                2 => wgt::BlendFactor::Src,
                3 => wgt::BlendFactor::OneMinusSrc,
                4 => wgt::BlendFactor::SrcAlpha,
                5 => wgt::BlendFactor::OneMinusSrcAlpha,
                6 => wgt::BlendFactor::Dst,
                7 => wgt::BlendFactor::OneMinusDst,
                8 => wgt::BlendFactor::DstAlpha,
                9 => wgt::BlendFactor::OneMinusDstAlpha,
                _ => wgt::BlendFactor::One,
            };
            let map_op = |o| match o {
                0 => wgt::BlendOperation::Add,
                1 => wgt::BlendOperation::Subtract,
                2 => wgt::BlendOperation::ReverseSubtract,
                3 => wgt::BlendOperation::Min,
                4 => wgt::BlendOperation::Max,
                _ => wgt::BlendOperation::Add,
            };

            Some(wgt::BlendState {
                color: wgt::BlendComponent {
                    src_factor: map_factor(config.color_blend_src),
                    dst_factor: map_factor(config.color_blend_dst),
                    operation: map_op(config.color_blend_op),
                },
                alpha: wgt::BlendComponent {
                    src_factor: map_factor(config.alpha_blend_src),
                    dst_factor: map_factor(config.alpha_blend_dst),
                    operation: map_op(config.alpha_blend_op),
                },
            })
        } else {
            None
        };

        let desc = pipeline::RenderPipelineDescriptor {
            label: None,
            layout: layout_id,
            vertex: pipeline::VertexState {
                stage: pipeline::ProgrammableStageDescriptor {
                    module: v_module,
                    entry_point: Some(Cow::Borrowed(config.vertex_entry)),
                    constants: Default::default(),
                    zero_initialize_workgroup_memory: true,
                },
                buffers: Cow::Owned(vertex_buffers),
            },
            primitive,
            depth_stencil,
            multisample: wgt::MultisampleState::default(),
            fragment: Some(pipeline::FragmentState {
                stage: pipeline::ProgrammableStageDescriptor {
                    module: f_module,
                    entry_point: Some(Cow::Borrowed(config.fragment_entry)),
                    constants: Default::default(),
                    zero_initialize_workgroup_memory: true,
                },
                targets: Cow::Borrowed(&[Some(wgt::ColorTargetState {
                    format: wgt::TextureFormat::Rgba8Unorm,
                    blend,
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
            crate::error::set_error(
                crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                super::WEBGPU_ERROR_INVALID_HANDLE,
                e,
            );
            return super::NULL_HANDLE;
        }

        let handle = ctx.next_render_pipeline_id;
        ctx.next_render_pipeline_id += 1;
        ctx.render_pipelines.insert(handle, pipeline_id);

        handle
    })
}

/// Create a pipeline layout
///
/// # Safety
///
/// This function is unsafe because it takes raw pointers.
pub unsafe fn create_pipeline_layout(
    ctx_handle: u32,
    device_handle: u32,
    bind_group_layouts_ptr: *const u32,
    bind_group_layouts_len: usize,
) -> u32 {
    let bgl_handles = std::slice::from_raw_parts(bind_group_layouts_ptr, bind_group_layouts_len);

    with_context(ctx_handle, |ctx| {
        let device_id = match ctx.devices.get(&device_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let mut bind_group_layouts = Vec::with_capacity(bind_group_layouts_len);
        for handle in bgl_handles {
            if let Some(id) = ctx.bind_group_layouts.get(handle) {
                bind_group_layouts.push(*id);
            } else {
                return super::NULL_HANDLE;
            }
        }

        let desc = wgpu_core::binding_model::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: Cow::Borrowed(&bind_group_layouts),
            immediate_size: 0,
        };

        let (layout_id, error) = ctx
            .global
            .device_create_pipeline_layout(device_id, &desc, None);
        if let Some(e) = error {
            crate::error::set_error(
                crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                super::WEBGPU_ERROR_INVALID_HANDLE,
                e,
            );
            return super::NULL_HANDLE;
        }

        let handle = ctx.next_pipeline_layout_id;
        ctx.next_pipeline_layout_id += 1;
        ctx.pipeline_layouts.insert(handle, layout_id);

        handle
    })
}
