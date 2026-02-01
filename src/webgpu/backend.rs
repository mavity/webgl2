use crate::naga_wasm_backend::WasmModule;
use crate::wasm_gl_emu;
use std::any::Any;
use std::collections::HashMap;
use std::num::NonZero;
use std::sync::{Arc, Mutex};

use crate::webgl2_context::types::*;
use std::time::Duration;
use wgpu_hal as hal;
use wgpu_types as wgt;

type VertexBufferEntry = Option<(Arc<Mutex<Vec<u8>>>, wgt::BufferAddress)>;
type IndexBufferEntry = Option<(Arc<Mutex<Vec<u8>>>, wgt::BufferAddress, wgt::IndexFormat)>;

macro_rules! impl_dyn_resource {
    ($($type:ty),*) => {
        $(
            impl hal::DynResource for $type {
                fn as_any(&self) -> &dyn Any {
                    self
                }
                fn as_any_mut(&mut self) -> &mut dyn Any {
                    self
                }
            }
        )*
    };
}

// The "Soft GPU" backend API entry point
#[derive(Clone, Debug)]
pub struct SoftApi;

impl hal::Api for SoftApi {
    type Instance = SoftInstance;
    type Surface = SoftSurface;
    type Adapter = SoftAdapter;
    type Device = SoftDevice;
    type Queue = SoftQueue;
    type CommandEncoder = SoftCommandEncoder;
    type CommandBuffer = SoftCommandBuffer;
    type Buffer = SoftBuffer;
    type Texture = SoftTexture;
    type SurfaceTexture = SoftTexture;
    type TextureView = SoftTextureView;
    type Sampler = SoftSampler;
    type QuerySet = SoftQuerySet;
    type Fence = SoftFence;
    type PipelineLayout = SoftPipelineLayout;
    type RenderPipeline = SoftRenderPipeline;
    type ComputePipeline = SoftComputePipeline;
    type ShaderModule = SoftShaderModule;
    type BindGroupLayout = SoftBindGroupLayout;
    type BindGroup = SoftBindGroup;
    type AccelerationStructure = SoftAccelerationStructure;
    type PipelineCache = SoftPipelineLayout; // Reuse for now or create SoftPipelineCache

    const VARIANT: wgt::Backend = wgt::Backend::Noop;
}

pub struct SoftInstance {
    pub attribute_buffer: Vec<u8>,
    pub varying_buffer: Vec<u8>,
    pub private_buffer: Vec<u8>,
    pub texture_metadata: Vec<u8>,
    pub frame_stack: Vec<u8>,
}

impl SoftInstance {
    pub fn new() -> Self {
        Self {
            attribute_buffer: vec![0u8; 1024],
            varying_buffer: vec![0u8; 131072],
            private_buffer: vec![0u8; 16384],
            texture_metadata: vec![0u8; 16384],
            frame_stack: vec![0u8; 131072],
        }
    }
}

impl Default for SoftInstance {
    fn default() -> Self {
        Self::new()
    }
}

impl hal::Instance for SoftInstance {
    type A = SoftApi;

    unsafe fn init(_desc: &hal::InstanceDescriptor) -> Result<Self, hal::InstanceError> {
        Ok(Self::new())
    }

    unsafe fn create_surface(
        &self,
        _display_handle: raw_window_handle::RawDisplayHandle,
        _window_handle: raw_window_handle::RawWindowHandle,
    ) -> Result<SoftSurface, hal::InstanceError> {
        Ok(SoftSurface)
    }

    unsafe fn enumerate_adapters(
        &self,
        _surface_hint: Option<&SoftSurface>,
    ) -> Vec<hal::ExposedAdapter<SoftApi>> {
        let adapter = SoftAdapter {
            memory: wasm_gl_emu::ShaderMemoryLayout {
                attr_ptr: self.attribute_buffer.as_ptr() as u32,
                uniform_ptr: 0, // Set per draw
                varying_ptr: self.varying_buffer.as_ptr() as u32,
                private_ptr: self.private_buffer.as_ptr() as u32,
                texture_ptr: self.texture_metadata.as_ptr() as u32,
                frame_sp: (self.frame_stack.as_ptr() as usize + self.frame_stack.len()) as u32,
            },
        };

        let info = wgt::AdapterInfo {
            name: "WASM Soft-GPU".to_string(),
            vendor: 0,
            device: 0,
            device_type: wgt::DeviceType::Cpu,
            driver: "soft-gpu".to_string(),
            driver_info: "WASM Software Rasterizer".to_string(),
            backend: wgt::Backend::Noop,
            device_pci_bus_id: 0.to_string(),
            subgroup_max_size: 0,
            subgroup_min_size: 0,
            transient_saves_memory: false,
        };

        vec![hal::ExposedAdapter {
            adapter,
            info,
            features: wgt::Features::empty(),
            capabilities: hal::Capabilities {
                limits: wgt::Limits::default(),
                downlevel: wgt::DownlevelCapabilities::default(),
                alignments: hal::Alignments {
                    buffer_copy_offset: NonZero::new(1).unwrap(),
                    buffer_copy_pitch: NonZero::new(1).unwrap(),
                    uniform_bounds_check_alignment: NonZero::new(1).unwrap(),
                    raw_tlas_instance_size: 0,
                    ray_tracing_scratch_buffer_alignment: 0,
                },
                cooperative_matrix_properties: vec![],
            },
        }]
    }
}

#[derive(Debug)]
pub struct SoftSurface;

impl hal::Surface for SoftSurface {
    type A = SoftApi;

    unsafe fn configure(
        &self,
        _device: &SoftDevice,
        _config: &hal::SurfaceConfiguration,
    ) -> Result<(), hal::SurfaceError> {
        Ok(())
    }

    unsafe fn unconfigure(&self, _device: &SoftDevice) {}

    unsafe fn acquire_texture(
        &self,
        _timeout: Option<std::time::Duration>,
        _fence: &SoftFence,
    ) -> Result<Option<hal::AcquiredSurfaceTexture<SoftApi>>, hal::SurfaceError> {
        // For now, return nothing or a dummy texture
        Ok(None)
    }

    unsafe fn discard_texture(&self, _texture: SoftTexture) {}
}

#[derive(Debug)]
pub struct SoftAdapter {
    pub memory: wasm_gl_emu::ShaderMemoryLayout,
}

impl hal::Adapter for SoftAdapter {
    type A = SoftApi;

    unsafe fn open(
        &self,
        _features: wgt::Features,
        _limits: &wgt::Limits,
        _memory_hints: &wgt::MemoryHints,
    ) -> Result<hal::OpenDevice<SoftApi>, hal::DeviceError> {
        let device = SoftDevice {
            // TODO: no need for sync in single-threaded!!
            _mem_allocator: Arc::new(Mutex::new(0)), // Simple allocator
            memory: self.memory,
        };
        let queue = SoftQueue {
            memory: self.memory,
        };
        Ok(hal::OpenDevice { device, queue })
    }

    unsafe fn texture_format_capabilities(
        &self,
        _format: wgt::TextureFormat,
    ) -> hal::TextureFormatCapabilities {
        hal::TextureFormatCapabilities::all()
    }

    unsafe fn surface_capabilities(
        &self,
        _surface: &SoftSurface,
    ) -> Option<hal::SurfaceCapabilities> {
        Some(hal::SurfaceCapabilities {
            formats: vec![wgt::TextureFormat::Rgba8Unorm],
            present_modes: vec![wgt::PresentMode::Fifo],
            composite_alpha_modes: vec![wgt::CompositeAlphaMode::Opaque],
            maximum_frame_latency: 2..=3, // RangeInclusive
            current_extent: None,
            usage: wgt::TextureUses::COLOR_TARGET,
        })
    }

    unsafe fn get_presentation_timestamp(&self) -> wgt::PresentationTimestamp {
        wgt::PresentationTimestamp::INVALID_TIMESTAMP
    }
}

#[derive(Debug)]
pub struct SoftDevice {
    // In a real implementation, this would manage memory
    // TODO: in single threaded there is no need for sync!!!
    _mem_allocator: Arc<Mutex<u32>>,
    pub memory: wasm_gl_emu::ShaderMemoryLayout,
}

impl SoftDevice {
    fn compile_stage(
        &self,
        module: &Arc<naga::Module>,
        entry_point: &str,
        stage: naga::ShaderStage,
    ) -> Result<SoftShaderStage, hal::PipelineError> {
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );

        let info = validator.validate(module).map_err(|e| {
            hal::PipelineError::Linkage(
                match stage {
                    naga::ShaderStage::Vertex => wgt::ShaderStages::VERTEX,
                    naga::ShaderStage::Fragment => wgt::ShaderStages::FRAGMENT,
                    naga::ShaderStage::Compute => wgt::ShaderStages::COMPUTE,
                    _ => wgt::ShaderStages::empty(),
                },
                format!("Naga validation failed: {:?}", e),
            )
        })?;

        let backend_config = crate::naga_wasm_backend::WasmBackendConfig {
            debug_shaders: false,
            ..Default::default()
        };

        let backend = crate::naga_wasm_backend::WasmBackend::new(backend_config);

        let config = crate::naga_wasm_backend::CompileConfig {
            module,
            info: &info,
            source: "",
            stage,
            entry_point: Some(entry_point),
            attribute_locations: &HashMap::new(),
            uniform_locations: &HashMap::new(),
            varying_locations: &HashMap::new(),
            varying_types: &HashMap::new(),
            uniform_types: &HashMap::new(),
            attribute_types: &HashMap::new(),
        };

        let mut wasm_module = backend.compile(config, Some(entry_point)).map_err(|e| {
            hal::PipelineError::Linkage(
                match stage {
                    naga::ShaderStage::Vertex => wgt::ShaderStages::VERTEX,
                    naga::ShaderStage::Fragment => wgt::ShaderStages::FRAGMENT,
                    naga::ShaderStage::Compute => wgt::ShaderStages::COMPUTE,
                    _ => wgt::ShaderStages::empty(),
                },
                format!("WASM backend compilation failed: {:?}", e),
            )
        })?;

        // Register with host to get table index
        wasm_module.table_index = crate::js_register_shader(&wasm_module.wasm_bytes);

        let uniform_map =
            crate::naga_wasm_backend::output_layout::get_uniform_map(module, &info, stage);

        Ok(SoftShaderStage {
            module: module.clone(),
            entry_point: entry_point.to_string(),
            wasm_module,
            uniform_map,
        })
    }
}

impl hal::Device for SoftDevice {
    type A = SoftApi;

    unsafe fn create_buffer(
        &self,
        desc: &hal::BufferDescriptor,
    ) -> Result<SoftBuffer, hal::DeviceError> {
        // Allocate memory for the buffer
        let size = desc.size as usize;
        let data = vec![0; size];

        Ok(SoftBuffer {
            data: Arc::new(Mutex::new(data)),
            size: desc.size,
            usage: desc.usage,
        })
    }

    unsafe fn destroy_buffer(&self, _buffer: SoftBuffer) {
        // Arc drops automatically
    }

    unsafe fn map_buffer(
        &self,
        buffer: &SoftBuffer,
        range: std::ops::Range<wgt::BufferAddress>,
    ) -> Result<hal::BufferMapping, hal::DeviceError> {
        let mut data = buffer.data.lock().unwrap();
        let ptr = data.as_mut_ptr().add(range.start as usize);

        Ok(hal::BufferMapping {
            ptr: std::ptr::NonNull::new(ptr).unwrap(),
            is_coherent: true,
        })
    }

    unsafe fn unmap_buffer(&self, _buffer: &SoftBuffer) {}

    unsafe fn flush_mapped_ranges<I>(&self, _buffer: &SoftBuffer, _ranges: I)
    where
        I: Iterator<Item = std::ops::Range<wgt::BufferAddress>>,
    {
        // Coherent memory, no flush needed
    }

    unsafe fn invalidate_mapped_ranges<I>(&self, _buffer: &SoftBuffer, _ranges: I)
    where
        I: Iterator<Item = std::ops::Range<wgt::BufferAddress>>,
    {
        // Coherent memory, no invalidate needed
    }

    unsafe fn create_command_encoder(
        &self,
        _desc: &hal::CommandEncoderDescriptor<SoftQueue>,
    ) -> Result<SoftCommandEncoder, hal::DeviceError> {
        Ok(SoftCommandEncoder {
            commands: Vec::new(),
            current_render_pass: None,
        })
    }

    unsafe fn create_bind_group_layout(
        &self,
        desc: &hal::BindGroupLayoutDescriptor,
    ) -> Result<SoftBindGroupLayout, hal::DeviceError> {
        Ok(SoftBindGroupLayout {
            entries: desc.entries.to_vec(),
        })
    }

    unsafe fn create_pipeline_layout(
        &self,
        _desc: &hal::PipelineLayoutDescriptor<SoftBindGroupLayout>,
    ) -> Result<SoftPipelineLayout, hal::DeviceError> {
        Ok(SoftPipelineLayout)
    }

    unsafe fn create_shader_module(
        &self,
        _desc: &hal::ShaderModuleDescriptor,
        shader: hal::ShaderInput,
    ) -> Result<SoftShaderModule, hal::ShaderError> {
        match shader {
            hal::ShaderInput::Naga(module) => Ok(SoftShaderModule {
                module: Arc::new(module.module.into_owned()),
            }),
            _ => Err(hal::ShaderError::Compilation(
                "SPIR-V or other non-WGSL inputs are not supported by the Soft backend".to_string(),
            )),
        }
    }

    unsafe fn create_render_pipeline(
        &self,
        desc: &hal::RenderPipelineDescriptor<
            SoftPipelineLayout,
            SoftShaderModule,
            SoftPipelineLayout,
        >,
    ) -> Result<SoftRenderPipeline, hal::PipelineError> {
        let (vertex_stage, vertex_layouts) = match &desc.vertex_processor {
            hal::VertexProcessor::Standard {
                vertex_stage,
                vertex_buffers,
            } => {
                let vs = self.compile_stage(
                    &vertex_stage.module.module,
                    vertex_stage.entry_point,
                    naga::ShaderStage::Vertex,
                )?;
                let layouts = vertex_buffers
                    .iter()
                    .map(|vb| SoftVertexBufferLayout {
                        array_stride: vb.array_stride,
                        step_mode: vb.step_mode,
                        attributes: vb.attributes.to_vec(),
                    })
                    .collect();
                (vs, layouts)
            }
            _ => {
                return Err(hal::PipelineError::Linkage(
                    wgt::ShaderStages::VERTEX,
                    "Mesh shaders not supported".into(),
                ))
            }
        };

        let fragment_stage = match &desc.fragment_stage {
            Some(fs) => self.compile_stage(
                &fs.module.module,
                fs.entry_point,
                naga::ShaderStage::Fragment,
            )?,
            None => {
                return Err(hal::PipelineError::Linkage(
                    wgt::ShaderStages::FRAGMENT,
                    "Fragment stage is mandatory for Soft backend for now".into(),
                ))
            }
        };

        Ok(SoftRenderPipeline {
            vertex_stage,
            fragment_stage,
            vertex_layouts,
            color_targets: desc.color_targets.to_vec(),
            depth_stencil: desc.depth_stencil.clone(),
            primitive: desc.primitive,
            multisample: desc.multisample,
        })
    }

    unsafe fn create_compute_pipeline(
        &self,
        _desc: &hal::ComputePipelineDescriptor<
            SoftPipelineLayout,
            SoftShaderModule,
            SoftPipelineLayout,
        >,
    ) -> Result<SoftComputePipeline, hal::PipelineError> {
        Ok(SoftComputePipeline)
    }

    unsafe fn create_bind_group(
        &self,
        desc: &hal::BindGroupDescriptor<
            SoftBindGroupLayout,
            SoftBuffer,
            SoftSampler,
            SoftTextureView,
            SoftAccelerationStructure,
        >,
    ) -> Result<SoftBindGroup, hal::DeviceError> {
        let mut resources = HashMap::new();

        let mut buffer_idx = 0;
        let mut sampler_idx = 0;
        let mut texture_idx = 0;

        for entry in &desc.layout.entries {
            match entry.ty {
                wgt::BindingType::Buffer { .. } => {
                    let b = &desc.buffers[buffer_idx];
                    resources.insert(
                        entry.binding,
                        SoftResource::Buffer {
                            buffer: b.buffer.clone(),
                            offset: b.offset,
                            size: b.size,
                        },
                    );
                    buffer_idx += 1;
                }
                wgt::BindingType::Sampler { .. } => {
                    resources.insert(
                        entry.binding,
                        SoftResource::Sampler((*desc.samplers[sampler_idx]).clone()),
                    );
                    sampler_idx += 1;
                }
                wgt::BindingType::Texture { .. } => {
                    resources.insert(
                        entry.binding,
                        SoftResource::TextureView(desc.textures[texture_idx].view.clone()),
                    );
                    texture_idx += 1;
                }
                _ => {}
            }
        }

        Ok(SoftBindGroup { resources })
    }

    unsafe fn create_texture(
        &self,
        desc: &hal::TextureDescriptor,
    ) -> Result<SoftTexture, hal::DeviceError> {
        let block_size = desc.format.block_copy_size(None).unwrap_or(4);
        let size =
            (desc.size.width * desc.size.height * desc.size.depth_or_array_layers * block_size)
                as usize;
        let data = vec![0; size];

        Ok(SoftTexture {
            data: Arc::new(Mutex::new(data)),
            desc: desc.into(),
        })
    }

    unsafe fn destroy_texture(&self, _texture: SoftTexture) {}

    unsafe fn create_texture_view(
        &self,
        texture: &SoftTexture,
        desc: &hal::TextureViewDescriptor,
    ) -> Result<SoftTextureView, hal::DeviceError> {
        Ok(SoftTextureView {
            texture: texture.data.clone(),
            desc: hal::TextureViewDescriptor {
                label: None,
                format: desc.format,
                dimension: desc.dimension,
                usage: desc.usage,
                range: desc.range,
            },
            texture_desc: texture.desc.clone(),
        })
    }

    unsafe fn create_sampler(
        &self,
        desc: &hal::SamplerDescriptor,
    ) -> Result<SoftSampler, hal::DeviceError> {
        Ok(SoftSampler {
            desc: hal::SamplerDescriptor {
                label: None,
                address_modes: desc.address_modes,
                mag_filter: desc.mag_filter,
                min_filter: desc.min_filter,
                mipmap_filter: desc.mipmap_filter,
                lod_clamp: desc.lod_clamp.clone(),
                compare: desc.compare,
                anisotropy_clamp: desc.anisotropy_clamp,
                border_color: desc.border_color,
            },
        })
    }

    unsafe fn create_query_set(
        &self,
        _desc: &wgt::QuerySetDescriptor<hal::Label>,
    ) -> Result<SoftQuerySet, hal::DeviceError> {
        Ok(SoftQuerySet)
    }

    unsafe fn create_fence(&self) -> Result<SoftFence, hal::DeviceError> {
        Ok(SoftFence {
            value: Arc::new(Mutex::new(0)),
        })
    }

    unsafe fn get_fence_value(
        &self,
        fence: &SoftFence,
    ) -> Result<hal::FenceValue, hal::DeviceError> {
        Ok(*fence.value.lock().unwrap())
    }

    unsafe fn wait(
        &self,
        fence: &SoftFence,
        value: hal::FenceValue,
        _timeout_ms: Option<Duration>,
    ) -> Result<bool, hal::DeviceError> {
        Ok(*fence.value.lock().unwrap() >= value)
    }

    // Missing methods implementation
    unsafe fn add_raw_buffer(&self, _buffer: &SoftBuffer) {
        todo!()
    }
    unsafe fn add_raw_texture(&self, _texture: &SoftTexture) {
        todo!()
    }
    unsafe fn destroy_texture_view(&self, _view: SoftTextureView) {}
    unsafe fn destroy_sampler(&self, _sampler: SoftSampler) {}
    unsafe fn destroy_bind_group_layout(&self, _bg_layout: SoftBindGroupLayout) {}
    unsafe fn destroy_pipeline_layout(&self, _pipeline_layout: SoftPipelineLayout) {}
    unsafe fn destroy_bind_group(&self, _group: SoftBindGroup) {}
    unsafe fn destroy_shader_module(&self, _module: SoftShaderModule) {}
    unsafe fn destroy_render_pipeline(&self, _pipeline: SoftRenderPipeline) {}
    unsafe fn destroy_compute_pipeline(&self, _pipeline: SoftComputePipeline) {}
    unsafe fn create_pipeline_cache(
        &self,
        _desc: &hal::PipelineCacheDescriptor,
    ) -> Result<SoftPipelineLayout, hal::PipelineCacheError> {
        Ok(SoftPipelineLayout)
    }
    unsafe fn destroy_pipeline_cache(&self, _cache: SoftPipelineLayout) {}
    unsafe fn destroy_query_set(&self, _set: SoftQuerySet) {}
    unsafe fn destroy_fence(&self, _fence: SoftFence) {}
    unsafe fn start_graphics_debugger_capture(&self) -> bool {
        false
    }
    unsafe fn stop_graphics_debugger_capture(&self) {}
    unsafe fn create_acceleration_structure(
        &self,
        _desc: &hal::AccelerationStructureDescriptor,
    ) -> Result<SoftAccelerationStructure, hal::DeviceError> {
        Ok(SoftAccelerationStructure)
    }
    unsafe fn get_acceleration_structure_build_sizes(
        &self,
        _desc: &hal::GetAccelerationStructureBuildSizesDescriptor<SoftBuffer>,
    ) -> hal::AccelerationStructureBuildSizes {
        hal::AccelerationStructureBuildSizes::default()
    }
    unsafe fn get_acceleration_structure_device_address(
        &self,
        _as: &SoftAccelerationStructure,
    ) -> wgt::BufferAddress {
        0
    }
    unsafe fn destroy_acceleration_structure(&self, _as: SoftAccelerationStructure) {}
    fn tlas_instance_to_bytes(&self, _instance: hal::TlasInstance) -> Vec<u8> {
        vec![]
    }
    fn get_internal_counters(&self) -> wgt::HalCounters {
        wgt::HalCounters::default()
    }
    fn check_if_oom(&self) -> Result<(), hal::DeviceError> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum SoftCommand {
    CopyBufferToBuffer {
        src: Arc<Mutex<Vec<u8>>>,
        dst: Arc<Mutex<Vec<u8>>>,
        regions: Vec<hal::BufferCopy>,
    },
    CopyTextureToBuffer {
        src: Arc<Mutex<Vec<u8>>>,
        dst: Arc<Mutex<Vec<u8>>>,
        regions: Vec<hal::BufferTextureCopy>,
        texture_desc: SoftTextureDescriptor,
    },
    CopyBufferToTexture {
        src: Arc<Mutex<Vec<u8>>>,
        dst: Arc<Mutex<Vec<u8>>>,
        regions: Vec<hal::BufferTextureCopy>,
        texture_desc: SoftTextureDescriptor,
    },
    RenderPass {
        desc: SoftRenderPassDescriptor,
        commands: Vec<SoftRenderCommand>,
    },
}

#[derive(Debug, Clone)]
pub struct SoftRenderPassDescriptor {
    pub color_attachments: Vec<Option<SoftRenderPassColorAttachment>>,
    pub depth_stencil_attachment: Option<SoftRenderPassDepthStencilAttachment>,
}

#[derive(Debug, Clone)]
pub struct SoftRenderPassColorAttachment {
    pub view: SoftTextureView,
    pub resolve_target: Option<SoftTextureView>,
    pub load_op: wgt::LoadOp<wgt::Color>,
    pub store_op: wgt::StoreOp,
    pub clear_value: wgt::Color,
}

#[derive(Debug, Clone)]
pub struct SoftRenderPassDepthStencilAttachment {
    pub view: SoftTextureView,
    pub depth_load_op: Option<wgt::LoadOp<f32>>,
    pub depth_store_op: Option<wgt::StoreOp>,
    pub depth_clear_value: f32,
    pub stencil_load_op: Option<wgt::LoadOp<u32>>,
    pub stencil_store_op: Option<wgt::StoreOp>,
    pub stencil_clear_value: u32,
}

#[derive(Debug, Clone)]
pub enum SoftRenderCommand {
    SetPipeline(Box<SoftRenderPipeline>),
    SetBindGroup {
        index: u32,
        group: SoftBindGroup,
        dynamic_offsets: Vec<u32>,
    },
    SetVertexBuffer {
        index: u32,
        buffer: SoftBuffer,
        offset: wgt::BufferAddress,
        size: Option<wgt::BufferSize>,
    },
    SetIndexBuffer {
        buffer: SoftBuffer,
        offset: wgt::BufferAddress,
        size: Option<wgt::BufferSize>,
        format: wgt::IndexFormat,
    },
    Draw {
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    },
    DrawIndexed {
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        base_vertex: i32,
        first_instance: u32,
    },
    SetViewport {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    },
    SetScissorRect {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
}

#[derive(Debug)]
pub struct SoftQueue {
    pub memory: wasm_gl_emu::ShaderMemoryLayout,
}

impl SoftQueue {
    fn scalarize_into(
        &self,
        uniform_data: &mut [u8],
        bind_groups: &[Option<SoftBindGroup>],
        stages: &[&SoftShaderStage],
    ) {
        let mut current_offset = crate::naga_wasm_backend::output_layout::CONTEXT_BLOCK_SIZE;
        let mut processed_bindings = std::collections::HashMap::new();

        for stage in stages {
            for (&(group_idx, binding_idx), &(context_offset, _layout)) in &stage.uniform_map {
                // Determine if we've already packed this resource's data
                // In WebGPU, same (group, binding) across stages refers to the same resource.
                let resource_key = (group_idx, binding_idx);

                if let Some(Some(bg)) = bind_groups.get(group_idx as usize) {
                    if let Some(res) = bg.resources.get(&binding_idx) {
                        match res {
                            SoftResource::Buffer {
                                buffer,
                                offset: buf_offset,
                                size,
                            } => {
                                // 1. Write the pointer to the context block ALWAYS (stage-specific offset)
                                let context_ptr = context_offset as usize;

                                // We need to know where the data for this resource lives.
                                // If already processed, reuse the existing offset.
                                let data_offset = if let Some(&existing_offset) =
                                    processed_bindings.get(&resource_key)
                                {
                                    existing_offset
                                } else {
                                    // Pack new data
                                    let data = buffer.data.lock().unwrap();
                                    let start = *buf_offset as usize;
                                    let end = if let Some(s) = size {
                                        start + s.get() as usize
                                    } else {
                                        data.len()
                                    };

                                    if start < data.len() {
                                        let actual_end = end.min(data.len());
                                        let bytes = &data[start..actual_end];

                                        // Align current_offset to 16 bytes for safety
                                        current_offset = (current_offset + 15) & !15;

                                        let copy_len = bytes.len();
                                        if current_offset + copy_len as u32
                                            <= uniform_data.len() as u32
                                        {
                                            let target_offset = current_offset;
                                            uniform_data[target_offset as usize
                                                ..target_offset as usize + copy_len]
                                                .copy_from_slice(bytes);

                                            current_offset += copy_len as u32;
                                            processed_bindings.insert(resource_key, target_offset);
                                            target_offset
                                        } else {
                                            0 // Should not happen with 4KB buffer
                                        }
                                    } else {
                                        0
                                    }
                                };

                                if context_ptr + 4 <= uniform_data.len() {
                                    uniform_data[context_ptr..context_ptr + 4]
                                        .copy_from_slice(&data_offset.to_le_bytes());
                                }
                            }
                            SoftResource::TextureView(_) | SoftResource::Sampler(_) => {
                                // Handled via texture_ptr context block in prepare_textures
                            }
                        }
                    }
                }
            }
        }
    }
}

impl hal::Queue for SoftQueue {
    type A = SoftApi;

    unsafe fn submit(
        &self,
        command_buffers: &[&SoftCommandBuffer],
        _surface_textures: &[&SoftTexture],
        fence: (&mut SoftFence, hal::FenceValue),
    ) -> Result<(), hal::DeviceError> {
        // Pass 1: Execute all copy commands first
        for cmd_buf in command_buffers {
            for cmd in &cmd_buf.commands {
                match cmd {
                    SoftCommand::CopyBufferToBuffer { src, dst, regions } => {
                        let src_data = src.lock().unwrap();
                        let mut dst_data = dst.lock().unwrap();
                        for region in regions {
                            let src_start = region.src_offset as usize;
                            let dst_start = region.dst_offset as usize;
                            let size = region.size.get() as usize;

                            if src_start + size <= src_data.len()
                                && dst_start + size <= dst_data.len()
                            {
                                dst_data[dst_start..dst_start + size]
                                    .copy_from_slice(&src_data[src_start..src_start + size]);
                            }
                        }
                    }

                    // TODO: why was this removed???
                    // SoftCommand::CopyTextureToBuffer {
                    //     src,
                    //     dst,
                    //     regions,
                    //     texture_desc,
                    // } => {
                    //     let src_data = src.lock().unwrap();
                    //     let mut dst_data = dst.lock().unwrap();
                    //     for region in regions {
                    //         let bytes_per_pixel = 4;
                    //         let width = region.size.width;
                    //         let height = region.size.height;
                    //         let depth = region.size.depth;
                    //         let row_pitch = width * bytes_per_pixel;
                    //         let slice_pitch = row_pitch * height;
                    //         let src_origin = region.texture_base.origin;
                    //         let src_offset =
                    //             (src_origin.z * texture_desc.size.height * texture_desc.size.width
                    //                 + src_origin.y * texture_desc.size.width
                    //                 + src_origin.x)
                    //                 * bytes_per_pixel;
                    //         let dst_offset = region.buffer_layout.offset;

                    //         for z in 0..depth {
                    //             for y in 0..height {
                    //                 let src_idx =
                    //                     (src_offset + (z * slice_pitch) + (y * row_pitch)) as usize;
                    //                 let dst_idx = (dst_offset
                    //                     + (z as u64 * slice_pitch as u64)
                    //                     + (y as u64 * row_pitch as u64))
                    //                     as usize;
                    //                 let len = row_pitch as usize;
                    //                 if src_idx + len <= src_data.len()
                    //                     && dst_idx + len <= dst_data.len()
                    //                 {
                    //                     dst_data[dst_idx..dst_idx + len]
                    //                         .copy_from_slice(&src_data[src_idx..src_idx + len]);
                    //                 }
                    //             }
                    //         }
                    //     }
                    // }
                    SoftCommand::CopyBufferToTexture {
                        src,
                        dst,
                        regions,
                        texture_desc,
                    } => {
                        let src_data = src.lock().unwrap();
                        let mut dst_data = dst.lock().unwrap();
                        for region in regions {
                            let bytes_per_pixel = match texture_desc.format {
                                wgt::TextureFormat::R32Float => 4,
                                wgt::TextureFormat::Rg32Float => 8,
                                wgt::TextureFormat::Rgba32Float => 16,
                                _ => 4,
                            };
                            let width = region.size.width;
                            let height = region.size.height;
                            let depth = region.size.depth;
                            let buffer_row_pitch = region
                                .buffer_layout
                                .bytes_per_row
                                .unwrap_or(width * bytes_per_pixel);
                            let buffer_slice_pitch =
                                region.buffer_layout.rows_per_image.unwrap_or(height)
                                    * buffer_row_pitch;
                            let texture_row_pitch = texture_desc.size.width * bytes_per_pixel;
                            let texture_slice_pitch = texture_desc.size.height * texture_row_pitch;
                            let src_offset = region.buffer_layout.offset;
                            let dst_origin = region.texture_base.origin;
                            let dst_offset = (dst_origin.z * texture_slice_pitch
                                + dst_origin.y * texture_row_pitch
                                + dst_origin.x * bytes_per_pixel)
                                as u64;

                            for z in 0..depth {
                                for y in 0..height {
                                    let src_idx = (src_offset
                                        + (z as u64 * buffer_slice_pitch as u64)
                                        + (y as u64 * buffer_row_pitch as u64))
                                        as usize;
                                    let dst_idx = (dst_offset
                                        + (z as u64 * texture_slice_pitch as u64)
                                        + (y as u64 * texture_row_pitch as u64))
                                        as usize;
                                    let len = (width * bytes_per_pixel) as usize;
                                    if src_idx + len <= src_data.len()
                                        && dst_idx + len <= dst_data.len()
                                    {
                                        dst_data[dst_idx..dst_idx + len]
                                            .copy_from_slice(&src_data[src_idx..src_idx + len]);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // TODO: why was this removed??
        // // Pass 2: Execute render passes
        // for cmd_buf in command_buffers {
        //     for cmd in &cmd_buf.commands {
        //         match cmd {
        //             _ => {}
        //         }
        //     }
        // }

        // Pass 2: Execute render passes and other non-copy commands
        for cmd_buf in command_buffers {
            for cmd in &cmd_buf.commands {
                if let SoftCommand::RenderPass { desc, commands } = cmd {
                    // 1. Handle LoadOps (Clearing)
                    for att in desc.color_attachments.iter().flatten() {
                        if let wgt::LoadOp::Clear(color) = att.load_op {
                            let mut data = att.view.texture.lock().unwrap();
                            let format = att.view.texture_desc.format;

                            // TODO: Handle other formats properly
                            match format {
                                wgt::TextureFormat::Rgba8Unorm | wgt::TextureFormat::Bgra8Unorm => {
                                    let r = (color.r * 255.0) as u8;
                                    let g = (color.g * 255.0) as u8;
                                    let b = (color.b * 255.0) as u8;
                                    let a = (color.a * 255.0) as u8;
                                    let pixel = [r, g, b, a];

                                    for chunk in data.chunks_mut(4) {
                                        if chunk.len() == 4 {
                                            chunk.copy_from_slice(&pixel);
                                        }
                                    }
                                }
                                _ => {
                                    eprintln!(
                                        "SoftGPU: Unsupported format for clear: {:?}",
                                        format
                                    );
                                }
                            }
                        }
                    }

                    if let Some(att) = &desc.depth_stencil_attachment {
                        let _data = att.view.texture.lock().unwrap();
                        // TODO: Implement depth/stencil clearing
                        // This requires knowing the depth/stencil layout in memory
                    }

                    // 2. Execute commands
                    let mut current_pipeline: Option<&SoftRenderPipeline> = None;
                    let mut vertex_buffers: Vec<VertexBufferEntry> = vec![None; 16];
                    let mut index_buffer: IndexBufferEntry = None;
                    let mut bind_groups: Vec<Option<SoftBindGroup>> = vec![None; 4];

                    let mut viewport = (0i32, 0i32, 0u32, 0u32);
                    let mut scissor = (0i32, 0i32, 0u32, 0u32);
                    let mut scissor_enabled = false;

                    if let Some(Some(att)) = desc.color_attachments.first() {
                        viewport = (
                            0,
                            0,
                            att.view.texture_desc.size.width,
                            att.view.texture_desc.size.height,
                        );
                        scissor = (
                            0,
                            0,
                            att.view.texture_desc.size.width,
                            att.view.texture_desc.size.height,
                        );
                    }

                    for command in commands {
                        match command {
                            SoftRenderCommand::SetViewport {
                                x,
                                y,
                                width,
                                height,
                                ..
                            } => {
                                viewport = (*x as i32, *y as i32, *width as u32, *height as u32);
                            }
                            SoftRenderCommand::SetScissorRect {
                                x,
                                y,
                                width,
                                height,
                            } => {
                                scissor = (*x as i32, *y as i32, *width, *height);
                                scissor_enabled = true;
                            }
                            SoftRenderCommand::SetPipeline(pipeline) => {
                                current_pipeline = Some(pipeline);
                            }
                            SoftRenderCommand::SetBindGroup {
                                index,
                                group,
                                dynamic_offsets: _,
                            } => {
                                if (*index as usize) < bind_groups.len() {
                                    bind_groups[*index as usize] = Some(group.clone());
                                }
                            }
                            SoftRenderCommand::SetVertexBuffer {
                                index,
                                buffer,
                                offset,
                                size: _,
                            } => {
                                if (*index as usize) < vertex_buffers.len() {
                                    vertex_buffers[*index as usize] =
                                        Some((buffer.data.clone(), *offset));
                                }
                            }
                            SoftRenderCommand::SetIndexBuffer {
                                buffer,
                                offset,
                                size: _,
                                format,
                            } => {
                                index_buffer = Some((buffer.data.clone(), *offset, *format));
                            }
                            SoftRenderCommand::Draw {
                                vertex_count,
                                instance_count,
                                first_vertex,
                                first_instance,
                            } => {
                                if let Some(pipeline) = current_pipeline {
                                    // Only handle the first color attachment for now
                                    if let Some(Some(att)) = desc.color_attachments.first() {
                                        let mut data = att.view.texture.lock().unwrap();
                                        let width = att.view.texture_desc.size.width;
                                        let height = att.view.texture_desc.size.height;

                                        let internal_format = match att.view.texture_desc.format {
                                            wgt::TextureFormat::R32Float => 0x822E,    // GL_R32F
                                            wgt::TextureFormat::Rg32Float => 0x8230,   // GL_RG32F
                                            wgt::TextureFormat::Rgba32Float => 0x8814, // GL_RGBA32F
                                            wgt::TextureFormat::Rgba8Unorm
                                            | wgt::TextureFormat::Bgra8Unorm => 0x8058, // GL_RGBA8
                                            _ => 0x8058,
                                        };

                                        let mut dummy_depth = vec![1.0; (width * height) as usize];
                                        let mut dummy_stencil =
                                            vec![0u8; (width * height) as usize];

                                        let rasterizer = wasm_gl_emu::Rasterizer::new();

                                        let fetcher = SoftVertexFetcher {
                                            vertex_buffers: &vertex_buffers,
                                            vertex_layouts: &pipeline.vertex_layouts,
                                        };

                                        let mut combined_uniforms = vec![0u8; 4096];
                                        self.scalarize_into(
                                            &mut combined_uniforms,
                                            &bind_groups,
                                            &[&pipeline.vertex_stage, &pipeline.fragment_stage],
                                        );

                                        // Pre-lock textures to keep pointers valid during draw
                                        type TexMetadata = (
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                        );
                                        let mut texture_metas: HashMap<u32, TexMetadata> =
                                            HashMap::new();
                                        let mut _texture_guards = Vec::new();
                                        let mut _already_locked = HashMap::new();

                                        for stage in
                                            &[&pipeline.vertex_stage, &pipeline.fragment_stage]
                                        {
                                            for (&(group, binding), &(offset, _)) in
                                                &stage.uniform_map
                                            {
                                                if let Some(Some(bg)) =
                                                    bind_groups.get(group as usize)
                                                {
                                                    match bg.resources.get(&binding) {
                                                        Some(SoftResource::TextureView(view)) => {
                                                            let tex_ptr =
                                                                Arc::as_ptr(&view.texture) as usize;
                                                            let data_ptr = if let Some(&p) =
                                                                _already_locked.get(&tex_ptr)
                                                            {
                                                                p
                                                            } else {
                                                                let guard =
                                                                    view.texture.lock().unwrap();
                                                                let p = guard.as_ptr() as u32;
                                                                _texture_guards.push(guard);
                                                                _already_locked.insert(tex_ptr, p);
                                                                p
                                                            };

                                                            let format = match view
                                                                .texture_desc
                                                                .format
                                                            {
                                                                wgt::TextureFormat::R32Float => {
                                                                    0x822E
                                                                }
                                                                wgt::TextureFormat::Rg32Float => {
                                                                    0x8230
                                                                }
                                                                wgt::TextureFormat::Rgba32Float => {
                                                                    0x8814
                                                                }
                                                                wgt::TextureFormat::Rgba8Unorm
                                                                | wgt::TextureFormat::Bgra8Unorm => {
                                                                    0x8058
                                                                }
                                                                _ => 0x8058,
                                                            };
                                                            let bpp = match format {
                                                                0x822E => 4,
                                                                0x8230 => 8,
                                                                0x8814 => 16,
                                                                _ => 4,
                                                            };

                                                            texture_metas.insert(
                                                                offset,
                                                                (
                                                                    view.texture_desc.size.width,
                                                                    view.texture_desc.size.height,
                                                                    data_ptr,
                                                                    view.texture_desc
                                                                        .size
                                                                        .depth_or_array_layers,
                                                                    format,
                                                                    bpp,
                                                                    0x2901, // GL_REPEAT
                                                                    0x2901, // GL_REPEAT
                                                                    0x2901, // Default wrap_r
                                                                    0,      // Layout: Linear
                                                                    0x2601, // min_filter: Linear
                                                                    0x2601, // mag_filter: Linear
                                                                ),
                                                            );
                                                        }
                                                        Some(SoftResource::Sampler(sampler)) => {
                                                            let wrap_s = match sampler
                                                                .desc
                                                                .address_modes[0]
                                                            {
                                                                wgt::AddressMode::Repeat => 0x2901,
                                                                wgt::AddressMode::MirrorRepeat => {
                                                                    0x8370
                                                                }
                                                                wgt::AddressMode::ClampToEdge => {
                                                                    0x812F
                                                                }
                                                                _ => 0x2901,
                                                            };
                                                            let wrap_t = match sampler
                                                                .desc
                                                                .address_modes[1]
                                                            {
                                                                wgt::AddressMode::Repeat => 0x2901,
                                                                wgt::AddressMode::MirrorRepeat => {
                                                                    0x8370
                                                                }
                                                                wgt::AddressMode::ClampToEdge => {
                                                                    0x812F
                                                                }
                                                                _ => 0x2901,
                                                            };
                                                            let wrap_r = match sampler
                                                                .desc
                                                                .address_modes[2]
                                                            {
                                                                wgt::AddressMode::Repeat => 0x2901,
                                                                wgt::AddressMode::MirrorRepeat => {
                                                                    0x8370
                                                                }
                                                                _ => 0x2901,
                                                            };
                                                            let min_filter = match sampler
                                                                .desc
                                                                .min_filter
                                                            {
                                                                wgt::FilterMode::Nearest => 0x2600,
                                                                wgt::FilterMode::Linear => 0x2601,
                                                            };
                                                            let mag_filter = match sampler
                                                                .desc
                                                                .mag_filter
                                                            {
                                                                wgt::FilterMode::Nearest => 0x2600,
                                                                wgt::FilterMode::Linear => 0x2601,
                                                            };

                                                            texture_metas.insert(
                                                                offset,
                                                                (
                                                                    0, 0, 0, 0, 0, 0, wrap_s,
                                                                    wrap_t, wrap_r,
                                                                    0, // Layout
                                                                    min_filter, mag_filter,
                                                                ),
                                                            );
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }

                                        let prepare_textures = Box::new(
                                            move |mem: &wasm_gl_emu::ShaderMemoryLayout| {
                                                for (&offset, meta) in &texture_metas {
                                                    // Place metadata after the 256-byte context block in the texture region.
                                                    // Each binding gets 64 bytes for its metadata.
                                                    let data_offset = 256 + (offset / 4) * 64;
                                                    let data_addr = mem.texture_ptr + data_offset;
                                                    unsafe {
                                                        // 1. Write the pointer to the UNIFORM context block (where the shader looks)
                                                        let ctx_ptr =
                                                            (mem.uniform_ptr + offset) as *mut u32;
                                                        *ctx_ptr = data_addr;

                                                        // 2. Write metadata to the pointed-to location (in the texture region)
                                                        let base = data_addr as *mut i32;
                                                        *base.offset(0) = meta.0 as i32; // width
                                                        *base.offset(1) = meta.1 as i32; // height
                                                        *base.offset(2) = meta.2 as i32; // data_ptr
                                                        *base.offset(3) = meta.3 as i32; // depth
                                                        *base.offset(4) = meta.4 as i32; // format
                                                        *base.offset(5) = meta.5 as i32; // bpp
                                                        *base.offset(6) = meta.6 as i32; // wrap_s
                                                        *base.offset(7) = meta.7 as i32; // wrap_t
                                                        *base.offset(8) = meta.8 as i32; // wrap_r
                                                        *base.offset(9) = meta.9 as i32; // layout
                                                        *base.offset(10) = meta.10 as i32; // min_filter
                                                        *base.offset(11) = meta.11 as i32;
                                                        // mag_filter
                                                    }
                                                }
                                            },
                                        );

                                        let mut raster_pipeline =
                                            wasm_gl_emu::RasterPipeline::new();
                                        raster_pipeline.memory = self.memory;
                                        raster_pipeline.memory.uniform_ptr =
                                            combined_uniforms.as_ptr() as u32;
                                        raster_pipeline.vs_table_idx =
                                            Some(pipeline.vertex_stage.wasm_module.table_index);
                                        raster_pipeline.fs_table_idx =
                                            Some(pipeline.fragment_stage.wasm_module.table_index);

                                        let mut state = wasm_gl_emu::RenderState {
                                            ctx_handle: 0,
                                            memory: self.memory,
                                            viewport,
                                            scissor,
                                            scissor_enabled,
                                            uniform_data: &combined_uniforms,
                                            prepare_textures: Some(prepare_textures),
                                            blend: wasm_gl_emu::rasterizer::BlendState::default(),
                                            color_mask:
                                                wasm_gl_emu::rasterizer::ColorMaskState::default(),
                                            depth: wasm_gl_emu::rasterizer::DepthState::default(),
                                            stencil: wasm_gl_emu::rasterizer::StencilState::default(
                                            ),
                                            cull_face_enabled: false,
                                            cull_face_mode: GL_BACK,
                                            front_face: GL_CCW,
                                        };
                                        state.memory.uniform_ptr =
                                            combined_uniforms.as_ptr() as u32;

                                        // Map depth/blend state
                                        if let Some(ds) = &pipeline.depth_stencil {
                                            state.depth.enabled = true;
                                            state.depth.mask = ds.depth_write_enabled;
                                            state.depth.func = match ds.depth_compare {
                                                wgt::CompareFunction::Never => GL_NEVER,
                                                wgt::CompareFunction::Less => GL_LESS,
                                                wgt::CompareFunction::Equal => GL_EQUAL,
                                                wgt::CompareFunction::LessEqual => GL_LEQUAL,
                                                wgt::CompareFunction::Greater => GL_GREATER,
                                                wgt::CompareFunction::NotEqual => GL_NOTEQUAL,
                                                wgt::CompareFunction::GreaterEqual => GL_GEQUAL,
                                                wgt::CompareFunction::Always => GL_ALWAYS,
                                            };
                                        }

                                        if let Some(Some(target)) = pipeline.color_targets.first() {
                                            if let Some(blend) = &target.blend {
                                                state.blend.enabled = true;
                                                let map_factor = |f| match f {
                                                    wgt::BlendFactor::Zero => GL_ZERO,
                                                    wgt::BlendFactor::One => GL_ONE,
                                                    wgt::BlendFactor::Src => GL_SRC_COLOR,
                                                    wgt::BlendFactor::OneMinusSrc => {
                                                        GL_ONE_MINUS_SRC_COLOR
                                                    }
                                                    wgt::BlendFactor::SrcAlpha => GL_SRC_ALPHA,
                                                    wgt::BlendFactor::OneMinusSrcAlpha => {
                                                        GL_ONE_MINUS_SRC_ALPHA
                                                    }
                                                    wgt::BlendFactor::Dst => GL_DST_COLOR,
                                                    wgt::BlendFactor::OneMinusDst => {
                                                        GL_ONE_MINUS_DST_COLOR
                                                    }
                                                    wgt::BlendFactor::DstAlpha => GL_DST_ALPHA,
                                                    wgt::BlendFactor::OneMinusDstAlpha => {
                                                        GL_ONE_MINUS_DST_ALPHA
                                                    }
                                                    _ => GL_ONE,
                                                };
                                                let map_op = |o| match o {
                                                    wgt::BlendOperation::Add => GL_FUNC_ADD,
                                                    wgt::BlendOperation::Subtract => {
                                                        GL_FUNC_SUBTRACT
                                                    }
                                                    wgt::BlendOperation::ReverseSubtract => {
                                                        GL_FUNC_REVERSE_SUBTRACT
                                                    }
                                                    wgt::BlendOperation::Min => GL_MIN,
                                                    wgt::BlendOperation::Max => GL_MAX,
                                                };
                                                state.blend.src_rgb =
                                                    map_factor(blend.color.src_factor);
                                                state.blend.dst_rgb =
                                                    map_factor(blend.color.dst_factor);
                                                state.blend.eq_rgb = map_op(blend.color.operation);
                                                state.blend.src_alpha =
                                                    map_factor(blend.alpha.src_factor);
                                                state.blend.dst_alpha =
                                                    map_factor(blend.alpha.dst_factor);
                                                state.blend.eq_alpha =
                                                    map_op(blend.alpha.operation);
                                            }
                                        }

                                        let mut dummy_kernel = wasm_gl_emu::GpuKernel::new();
                                        rasterizer.draw(
                                                &mut dummy_kernel,
                                                wasm_gl_emu::rasterizer::DrawConfig {
                                                    color_targets: vec![
                                                        wasm_gl_emu::rasterizer::ColorTarget::Raw(
                                                            &mut data,
                                                        )
                                                    ],
                                                    depth_stencil_target:
                                                        wasm_gl_emu::rasterizer::DepthStencilTarget::Raw {
                                                            depth: &mut dummy_depth,
                                                            stencil: &mut dummy_stencil,
                                                        },
                                                    width,
                                                    height,
                                                    internal_formats: vec![internal_format],
                                                    pipeline: &raster_pipeline,
                                                    state: &state,
                                                    vertex_fetcher: &fetcher,
                                                    vertex_count: *vertex_count as usize,
                                                    instance_count: *instance_count as usize,
                                                    first_vertex: *first_vertex as usize,
                                                    first_instance: *first_instance as usize,
                                                    indices: None,
                                                    mode: match pipeline.primitive.topology {
                                                        wgt::PrimitiveTopology::PointList => {
                                                            GL_POINTS
                                                        }
                                                        wgt::PrimitiveTopology::LineList => {
                                                            GL_LINES
                                                        }
                                                        wgt::PrimitiveTopology::LineStrip => {
                                                            GL_LINE_STRIP
                                                        }
                                                        wgt::PrimitiveTopology::TriangleList => {
                                                            GL_TRIANGLES
                                                        }
                                                        wgt::PrimitiveTopology::TriangleStrip => {
                                                            GL_TRIANGLE_STRIP
                                                        }
                                                    },
                                                },
                                            );
                                    }
                                }
                            }
                            SoftRenderCommand::DrawIndexed {
                                index_count,
                                instance_count,
                                first_index,
                                base_vertex,
                                first_instance,
                            } => {
                                if let Some(pipeline) = current_pipeline {
                                    if let Some(Some(att)) = desc.color_attachments.first() {
                                        let mut data = att.view.texture.lock().unwrap();
                                        let width = att.view.texture_desc.size.width;
                                        let height = att.view.texture_desc.size.height;

                                        let internal_format = match att.view.texture_desc.format {
                                            wgt::TextureFormat::R32Float => 0x822E,
                                            wgt::TextureFormat::Rg32Float => 0x8230,
                                            wgt::TextureFormat::Rgba32Float => 0x8814,
                                            wgt::TextureFormat::Rgba8Unorm
                                            | wgt::TextureFormat::Bgra8Unorm => 0x8058,
                                            _ => 0x8058,
                                        };

                                        let mut dummy_depth = vec![1.0; (width * height) as usize];
                                        let mut dummy_stencil =
                                            vec![0u8; (width * height) as usize];

                                        let rasterizer = wasm_gl_emu::Rasterizer::new();

                                        let fetcher = SoftVertexFetcher {
                                            vertex_buffers: &vertex_buffers,
                                            vertex_layouts: &pipeline.vertex_layouts,
                                        };

                                        let mut combined_uniforms = vec![0u8; 4096];
                                        self.scalarize_into(
                                            &mut combined_uniforms,
                                            &bind_groups,
                                            &[&pipeline.vertex_stage, &pipeline.fragment_stage],
                                        );

                                        // Pre-lock textures to keep pointers valid during draw
                                        type TexMetadata = (
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                            u32,
                                        );
                                        let mut texture_metas: HashMap<u32, TexMetadata> =
                                            HashMap::new();
                                        let mut _texture_guards = Vec::new();
                                        let mut _already_locked = HashMap::new();

                                        for stage in
                                            &[&pipeline.vertex_stage, &pipeline.fragment_stage]
                                        {
                                            for (&(group, binding), &(offset, _)) in
                                                &stage.uniform_map
                                            {
                                                if let Some(Some(bg)) =
                                                    bind_groups.get(group as usize)
                                                {
                                                    match bg.resources.get(&binding) {
                                                        Some(SoftResource::TextureView(view)) => {
                                                            let tex_ptr =
                                                                Arc::as_ptr(&view.texture) as usize;
                                                            let data_ptr = if let Some(&p) =
                                                                _already_locked.get(&tex_ptr)
                                                            {
                                                                p
                                                            } else {
                                                                let guard =
                                                                    view.texture.lock().unwrap();
                                                                let p = guard.as_ptr() as u32;
                                                                _texture_guards.push(guard);
                                                                _already_locked.insert(tex_ptr, p);
                                                                p
                                                            };

                                                            let format = match view
                                                                .texture_desc
                                                                .format
                                                            {
                                                                wgt::TextureFormat::R32Float => {
                                                                    0x822E
                                                                }
                                                                wgt::TextureFormat::Rg32Float => {
                                                                    0x8230
                                                                }
                                                                wgt::TextureFormat::Rgba32Float => {
                                                                    0x8814
                                                                }
                                                                wgt::TextureFormat::Rgba8Unorm
                                                                | wgt::TextureFormat::Bgra8Unorm => {
                                                                    0x8058
                                                                }
                                                                _ => 0x8058,
                                                            };
                                                            let bpp = match format {
                                                                0x822E => 4,
                                                                0x8230 => 8,
                                                                0x8814 => 16,
                                                                _ => 4,
                                                            };

                                                            texture_metas.insert(
                                                                offset,
                                                                (
                                                                    view.texture_desc.size.width,
                                                                    view.texture_desc.size.height,
                                                                    data_ptr,
                                                                    view.texture_desc
                                                                        .size
                                                                        .depth_or_array_layers,
                                                                    format,
                                                                    bpp,
                                                                    0x2901, // GL_REPEAT
                                                                    0x2901, // GL_REPEAT
                                                                    0x2901, // Default wrap_r
                                                                    0,      // Layout: Linear
                                                                    0x2601, // min_filter: Linear
                                                                    0x2601, // mag_filter: Linear
                                                                ),
                                                            );
                                                        }
                                                        Some(SoftResource::Sampler(sampler)) => {
                                                            let wrap_s = match sampler
                                                                .desc
                                                                .address_modes[0]
                                                            {
                                                                wgt::AddressMode::Repeat => 0x2901,
                                                                wgt::AddressMode::MirrorRepeat => {
                                                                    0x8370
                                                                }
                                                                wgt::AddressMode::ClampToEdge => {
                                                                    0x812F
                                                                }
                                                                _ => 0x2901,
                                                            };
                                                            let wrap_t = match sampler
                                                                .desc
                                                                .address_modes[1]
                                                            {
                                                                wgt::AddressMode::Repeat => 0x2901,
                                                                wgt::AddressMode::MirrorRepeat => {
                                                                    0x8370
                                                                }
                                                                wgt::AddressMode::ClampToEdge => {
                                                                    0x812F
                                                                }
                                                                _ => 0x2901,
                                                            };
                                                            let wrap_r = match sampler
                                                                .desc
                                                                .address_modes[2]
                                                            {
                                                                wgt::AddressMode::Repeat => 0x2901,
                                                                wgt::AddressMode::MirrorRepeat => {
                                                                    0x8370
                                                                }
                                                                _ => 0x2901,
                                                            };
                                                            let min_filter = match sampler
                                                                .desc
                                                                .min_filter
                                                            {
                                                                wgt::FilterMode::Nearest => 0x2600,
                                                                wgt::FilterMode::Linear => 0x2601,
                                                            };
                                                            let mag_filter = match sampler
                                                                .desc
                                                                .mag_filter
                                                            {
                                                                wgt::FilterMode::Nearest => 0x2600,
                                                                wgt::FilterMode::Linear => 0x2601,
                                                            };

                                                            texture_metas.insert(
                                                                offset,
                                                                (
                                                                    0, 0, 0, 0, 0, 0, wrap_s,
                                                                    wrap_t, wrap_r,
                                                                    0, // Layout
                                                                    min_filter, mag_filter,
                                                                ),
                                                            );
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }

                                        let prepare_textures = Box::new(
                                            move |mem: &wasm_gl_emu::ShaderMemoryLayout| {
                                                for (&offset, meta) in &texture_metas {
                                                    // Place metadata after the 256-byte context block.
                                                    // Each binding gets 64 bytes for its metadata.
                                                    let data_offset = 256 + (offset / 4) * 64;
                                                    let data_addr = mem.texture_ptr + data_offset;
                                                    unsafe {
                                                        // 1. Write the pointer to the UNIFORM context block
                                                        let ctx_ptr =
                                                            (mem.uniform_ptr + offset) as *mut u32;
                                                        *ctx_ptr = data_addr;

                                                        // 2. Write metadata to the pointed-to location
                                                        let base = data_addr as *mut i32;
                                                        *base.offset(0) = meta.0 as i32; // width
                                                        *base.offset(1) = meta.1 as i32; // height
                                                        *base.offset(2) = meta.2 as i32; // data_ptr
                                                        *base.offset(3) = meta.3 as i32; // depth
                                                        *base.offset(4) = meta.4 as i32; // format
                                                        *base.offset(5) = meta.5 as i32; // bpp
                                                        *base.offset(6) = meta.6 as i32; // wrap_s
                                                        *base.offset(7) = meta.7 as i32; // wrap_t
                                                        *base.offset(8) = meta.8 as i32; // wrap_r
                                                        *base.offset(9) = meta.9 as i32; // layout
                                                        *base.offset(10) = meta.10 as i32; // min_filter
                                                        *base.offset(11) = meta.11 as i32;
                                                        // mag_filter
                                                    }
                                                }
                                            },
                                        );

                                        let mut state = wasm_gl_emu::RenderState {
                                            ctx_handle: 0,
                                            memory: self.memory,
                                            viewport,
                                            scissor,
                                            scissor_enabled,
                                            uniform_data: &combined_uniforms,
                                            prepare_textures: Some(prepare_textures),
                                            blend: wasm_gl_emu::rasterizer::BlendState::default(),
                                            color_mask:
                                                wasm_gl_emu::rasterizer::ColorMaskState::default(),
                                            depth: wasm_gl_emu::rasterizer::DepthState::default(),
                                            stencil: wasm_gl_emu::rasterizer::StencilState::default(
                                            ),
                                            cull_face_enabled: false,
                                            cull_face_mode: GL_BACK,
                                            front_face: GL_CCW,
                                        };
                                        state.memory.uniform_ptr =
                                            combined_uniforms.as_ptr() as u32;

                                        // Map depth/blend state (same as Draw)
                                        if let Some(ds) = &pipeline.depth_stencil {
                                            state.depth.enabled = true;
                                            state.depth.mask = ds.depth_write_enabled;
                                            state.depth.func = match ds.depth_compare {
                                                wgt::CompareFunction::Never => GL_NEVER,
                                                wgt::CompareFunction::Less => GL_LESS,
                                                wgt::CompareFunction::Equal => GL_EQUAL,
                                                wgt::CompareFunction::LessEqual => GL_LEQUAL,
                                                wgt::CompareFunction::Greater => GL_GREATER,
                                                wgt::CompareFunction::NotEqual => GL_NOTEQUAL,
                                                wgt::CompareFunction::GreaterEqual => GL_GEQUAL,
                                                wgt::CompareFunction::Always => GL_ALWAYS,
                                            };
                                        }

                                        // Map blend state from pipeline
                                        if let Some(Some(target)) = pipeline.color_targets.first() {
                                            if let Some(blend) = &target.blend {
                                                state.blend.enabled = true;
                                                let map_factor = |f| match f {
                                                    wgt::BlendFactor::Zero => GL_ZERO,
                                                    wgt::BlendFactor::One => GL_ONE,
                                                    wgt::BlendFactor::Src => GL_SRC_COLOR,
                                                    wgt::BlendFactor::OneMinusSrc => {
                                                        GL_ONE_MINUS_SRC_COLOR
                                                    }
                                                    wgt::BlendFactor::SrcAlpha => GL_SRC_ALPHA,
                                                    wgt::BlendFactor::OneMinusSrcAlpha => {
                                                        GL_ONE_MINUS_SRC_ALPHA
                                                    }
                                                    wgt::BlendFactor::Dst => GL_DST_COLOR,
                                                    wgt::BlendFactor::OneMinusDst => {
                                                        GL_ONE_MINUS_DST_COLOR
                                                    }
                                                    wgt::BlendFactor::DstAlpha => GL_DST_ALPHA,
                                                    wgt::BlendFactor::OneMinusDstAlpha => {
                                                        GL_ONE_MINUS_DST_ALPHA
                                                    }
                                                    _ => GL_ONE,
                                                };
                                                let map_op = |o| match o {
                                                    wgt::BlendOperation::Add => GL_FUNC_ADD,
                                                    wgt::BlendOperation::Subtract => {
                                                        GL_FUNC_SUBTRACT
                                                    }
                                                    wgt::BlendOperation::ReverseSubtract => {
                                                        GL_FUNC_REVERSE_SUBTRACT
                                                    }
                                                    wgt::BlendOperation::Min => GL_MIN,
                                                    wgt::BlendOperation::Max => GL_MAX,
                                                };
                                                state.blend.src_rgb =
                                                    map_factor(blend.color.src_factor);
                                                state.blend.dst_rgb =
                                                    map_factor(blend.color.dst_factor);
                                                state.blend.eq_rgb = map_op(blend.color.operation);
                                                state.blend.src_alpha =
                                                    map_factor(blend.alpha.src_factor);
                                                state.blend.dst_alpha =
                                                    map_factor(blend.alpha.dst_factor);
                                                state.blend.eq_alpha =
                                                    map_op(blend.alpha.operation);
                                            }
                                        }

                                        let mut raster_pipeline =
                                            wasm_gl_emu::RasterPipeline::new();
                                        raster_pipeline.memory = self.memory;
                                        raster_pipeline.memory.uniform_ptr =
                                            combined_uniforms.as_ptr() as u32;
                                        raster_pipeline.vs_table_idx =
                                            Some(pipeline.vertex_stage.wasm_module.table_index);
                                        raster_pipeline.fs_table_idx =
                                            Some(pipeline.fragment_stage.wasm_module.table_index);

                                        // Fetch index buffer data
                                        let indices = if let Some((buffer, offset, format)) =
                                            &index_buffer
                                        {
                                            let data = buffer.lock().unwrap();
                                            let start = *offset as usize;
                                            let count = *index_count as usize;
                                            let mut idxs = Vec::with_capacity(count);

                                            let (_index_size, start_offset) = match format {
                                                wgt::IndexFormat::Uint16 => {
                                                    (2, *first_index as usize * 2)
                                                }
                                                wgt::IndexFormat::Uint32 => {
                                                    (4, *first_index as usize * 4)
                                                }
                                            };
                                            let final_start = start + start_offset;

                                            match format {
                                                wgt::IndexFormat::Uint16 => {
                                                    for i in 0..count {
                                                        let pos = final_start + i * 2;
                                                        if pos + 2 <= data.len() {
                                                            let val = u16::from_le_bytes([
                                                                data[pos],
                                                                data[pos + 1],
                                                            ]);
                                                            idxs.push(
                                                                (val as i32 + *base_vertex) as u32,
                                                            );
                                                        }
                                                    }
                                                }
                                                wgt::IndexFormat::Uint32 => {
                                                    for i in 0..count {
                                                        let pos = final_start + i * 4;
                                                        if pos + 4 <= data.len() {
                                                            let val = u32::from_le_bytes([
                                                                data[pos],
                                                                data[pos + 1],
                                                                data[pos + 2],
                                                                data[pos + 3],
                                                            ]);
                                                            idxs.push(
                                                                (val as i32 + *base_vertex) as u32,
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                            Some(idxs)
                                        } else {
                                            None
                                        };

                                        let mut dummy_kernel = wasm_gl_emu::GpuKernel::new();
                                        rasterizer.draw(&mut dummy_kernel, wasm_gl_emu::rasterizer::DrawConfig {
                                                color_targets: vec![wasm_gl_emu::rasterizer::ColorTarget::Raw(&mut data)],
                                                depth_stencil_target: wasm_gl_emu::rasterizer::DepthStencilTarget::Raw {
                                                    depth: &mut dummy_depth,
                                                    stencil: &mut dummy_stencil,
                                                },
                                                width,
                                                height,
                                                internal_formats: vec![internal_format],
                                                pipeline: &raster_pipeline,
                                                state: &state,
                                                vertex_fetcher: &fetcher,
                                                vertex_count: *index_count as usize,
                                                instance_count: *instance_count as usize,
                                                first_vertex: 0, // Using 0 because base_vertex is already added to indices
                                                first_instance: *first_instance as usize,
                                                indices: indices.as_ref().map(|v| v as &dyn wasm_gl_emu::rasterizer::IndexBuffer),
                                                mode: match pipeline.primitive.topology {
                                                    wgt::PrimitiveTopology::PointList => GL_POINTS,
                                                    wgt::PrimitiveTopology::LineList => GL_LINES,
                                                    wgt::PrimitiveTopology::LineStrip => GL_LINE_STRIP,
                                                    wgt::PrimitiveTopology::TriangleList => GL_TRIANGLES,
                                                    wgt::PrimitiveTopology::TriangleStrip => GL_TRIANGLE_STRIP,
                                                },
                                            });
                                    }
                                }
                            }
                        }
                    }

                    // 3. Handle StoreOps (Resolve)
                    // Currently we write directly to the texture, so StoreOp::Store is implicit.
                    // StoreOp::Discard would mean we don't care, but we already wrote it.
                    // Resolve targets would need to be handled here.
                }
            }
        }

        // Pass 3: Execute all readback copies (TextureToBuffer)
        for cmd_buf in command_buffers {
            for cmd in &cmd_buf.commands {
                if let SoftCommand::CopyTextureToBuffer {
                    src,
                    dst,
                    regions,
                    texture_desc,
                } = cmd
                {
                    let src_data = src.lock().unwrap();
                    let mut dst_data = dst.lock().unwrap();
                    for region in regions {
                        let bytes_per_pixel = 4;
                        let width = region.size.width;
                        let height = region.size.height;
                        let depth = region.size.depth;
                        let row_pitch = width * bytes_per_pixel;
                        let slice_pitch = row_pitch * height;
                        let src_origin = region.texture_base.origin;
                        let src_offset =
                            (src_origin.z * texture_desc.size.height * texture_desc.size.width
                                + src_origin.y * texture_desc.size.width
                                + src_origin.x)
                                * bytes_per_pixel;
                        let dst_offset = region.buffer_layout.offset;

                        for z in 0..depth {
                            for y in 0..height {
                                let src_idx =
                                    (src_offset + (z * slice_pitch) + (y * row_pitch)) as usize;
                                let dst_idx = (dst_offset
                                    + (z as u64 * slice_pitch as u64)
                                    + (y as u64 * row_pitch as u64))
                                    as usize;
                                let len = row_pitch as usize;
                                if src_idx + len <= src_data.len()
                                    && dst_idx + len <= dst_data.len()
                                {
                                    dst_data[dst_idx..dst_idx + len]
                                        .copy_from_slice(&src_data[src_idx..src_idx + len]);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Update fence
        let (fence, value) = fence;
        *fence.value.lock().unwrap() = value;

        Ok(())
    }

    unsafe fn present(
        &self,
        _surface: &SoftSurface,
        _texture: SoftTexture,
    ) -> Result<(), hal::SurfaceError> {
        Ok(())
    }

    unsafe fn get_timestamp_period(&self) -> f32 {
        1.0
    }
}

#[derive(Debug)]
pub struct SoftCommandEncoder {
    commands: Vec<SoftCommand>,
    current_render_pass: Option<(SoftRenderPassDescriptor, Vec<SoftRenderCommand>)>,
}

impl hal::CommandEncoder for SoftCommandEncoder {
    type A = SoftApi;

    unsafe fn begin_encoding(&mut self, _label: hal::Label) -> Result<(), hal::DeviceError> {
        self.commands.clear();
        self.current_render_pass = None;
        Ok(())
    }

    unsafe fn discard_encoding(&mut self) {
        self.commands.clear();
        self.current_render_pass = None;
    }

    unsafe fn end_encoding(&mut self) -> Result<SoftCommandBuffer, hal::DeviceError> {
        let cmd_buf = SoftCommandBuffer {
            commands: std::mem::take(&mut self.commands),
        };
        Ok(cmd_buf)
    }

    unsafe fn reset_all<I>(&mut self, _command_buffers: I)
    where
        I: Iterator<Item = SoftCommandBuffer>,
    {
        // In a real implementation, we might recycle command buffers
    }

    // Missing methods implementation
    unsafe fn transition_buffers<'a, T>(&mut self, _barriers: T)
    where
        T: Iterator<Item = hal::BufferBarrier<'a, SoftBuffer>>,
    {
    }
    unsafe fn transition_textures<'a, T>(&mut self, _barriers: T)
    where
        T: Iterator<Item = hal::TextureBarrier<'a, SoftTexture>>,
    {
    }
    unsafe fn clear_buffer(
        &mut self,
        _buffer: &SoftBuffer,
        _range: std::ops::Range<wgt::BufferAddress>,
    ) {
    }

    unsafe fn copy_buffer_to_buffer<T>(&mut self, src: &SoftBuffer, dst: &SoftBuffer, regions: T)
    where
        T: Iterator<Item = hal::BufferCopy>,
    {
        let regions_vec: Vec<hal::BufferCopy> = regions.collect();
        self.commands.push(SoftCommand::CopyBufferToBuffer {
            src: src.data.clone(),
            dst: dst.data.clone(),
            regions: regions_vec,
        });
    }

    unsafe fn copy_texture_to_texture<T>(
        &mut self,
        _src: &SoftTexture,
        _src_usage: wgt::TextureUses,
        _dst: &SoftTexture,
        _regions: T,
    ) where
        T: Iterator<Item = hal::TextureCopy>,
    {
    }
    unsafe fn copy_buffer_to_texture<T>(&mut self, src: &SoftBuffer, dst: &SoftTexture, regions: T)
    where
        T: Iterator<Item = hal::BufferTextureCopy>,
    {
        let regions_vec: Vec<hal::BufferTextureCopy> = regions.collect();
        self.commands.push(SoftCommand::CopyBufferToTexture {
            src: src.data.clone(),
            dst: dst.data.clone(),
            regions: regions_vec,
            texture_desc: dst.desc.clone(),
        });
    }
    unsafe fn copy_texture_to_buffer<T>(
        &mut self,
        src: &SoftTexture,
        _src_usage: wgt::TextureUses,
        dst: &SoftBuffer,
        regions: T,
    ) where
        T: Iterator<Item = hal::BufferTextureCopy>,
    {
        let regions_vec: Vec<hal::BufferTextureCopy> = regions.collect();
        self.commands.push(SoftCommand::CopyTextureToBuffer {
            src: src.data.clone(),
            dst: dst.data.clone(),
            regions: regions_vec,
            texture_desc: src.desc.clone(),
        });
    }
    unsafe fn copy_acceleration_structure_to_acceleration_structure(
        &mut self,
        _src: &SoftAccelerationStructure,
        _dst: &SoftAccelerationStructure,
        _copy: wgt::AccelerationStructureCopy,
    ) {
    }
    unsafe fn set_bind_group(
        &mut self,
        _layout: &SoftPipelineLayout,
        index: u32,
        group: &SoftBindGroup,
        dynamic_offsets: &[u32],
    ) {
        if let Some((_, commands)) = &mut self.current_render_pass {
            commands.push(SoftRenderCommand::SetBindGroup {
                index,
                group: group.clone(),
                dynamic_offsets: dynamic_offsets.to_vec(),
            });
        }
    }
    unsafe fn set_immediates(&mut self, _layout: &SoftPipelineLayout, _index: u32, _data: &[u32]) {}
    unsafe fn insert_debug_marker(&mut self, _label: &str) {}
    unsafe fn begin_debug_marker(&mut self, _group_label: &str) {}
    unsafe fn end_debug_marker(&mut self) {}
    unsafe fn begin_query(&mut self, _set: &SoftQuerySet, _index: u32) {}
    unsafe fn end_query(&mut self, _set: &SoftQuerySet, _index: u32) {}
    unsafe fn write_timestamp(&mut self, _set: &SoftQuerySet, _index: u32) {}
    unsafe fn reset_queries(&mut self, _set: &SoftQuerySet, _range: std::ops::Range<u32>) {}
    unsafe fn copy_query_results(
        &mut self,
        _set: &SoftQuerySet,
        _range: std::ops::Range<u32>,
        _buffer: &SoftBuffer,
        _offset: wgt::BufferAddress,
        _stride: NonZero<wgt::BufferAddress>,
    ) {
    }

    unsafe fn begin_render_pass(
        &mut self,
        desc: &hal::RenderPassDescriptor<SoftQuerySet, SoftTextureView>,
    ) -> Result<(), hal::DeviceError> {
        let color_attachments = desc
            .color_attachments
            .iter()
            .map(|att| {
                att.as_ref().map(|a| {
                    let load_op = if a.ops.contains(hal::AttachmentOps::LOAD) {
                        wgt::LoadOp::Load
                    } else {
                        wgt::LoadOp::Clear(a.clear_value)
                    };

                    let store_op = if a.ops.contains(hal::AttachmentOps::STORE) {
                        wgt::StoreOp::Store
                    } else {
                        wgt::StoreOp::Discard
                    };

                    SoftRenderPassColorAttachment {
                        view: a.target.view.clone(),
                        resolve_target: a.resolve_target.as_ref().map(|r| r.view.clone()),
                        load_op,
                        store_op,
                        clear_value: a.clear_value,
                    }
                })
            })
            .collect();

        let depth_stencil_attachment = desc.depth_stencil_attachment.as_ref().map(|a| {
            let depth_load_op = if a.depth_ops.contains(hal::AttachmentOps::LOAD) {
                Some(wgt::LoadOp::Load)
            } else if a.depth_ops.contains(hal::AttachmentOps::LOAD_CLEAR) {
                Some(wgt::LoadOp::Clear(a.clear_value.0))
            } else {
                None
            };

            let depth_store_op = if a.depth_ops.contains(hal::AttachmentOps::STORE) {
                Some(wgt::StoreOp::Store)
            } else if a.depth_ops.contains(hal::AttachmentOps::STORE_DISCARD) {
                Some(wgt::StoreOp::Discard)
            } else {
                None
            };

            let stencil_load_op = if a.stencil_ops.contains(hal::AttachmentOps::LOAD) {
                Some(wgt::LoadOp::Load)
            } else if a.stencil_ops.contains(hal::AttachmentOps::LOAD_CLEAR) {
                Some(wgt::LoadOp::Clear(a.clear_value.1))
            } else {
                None
            };

            let stencil_store_op = if a.stencil_ops.contains(hal::AttachmentOps::STORE) {
                Some(wgt::StoreOp::Store)
            } else if a.stencil_ops.contains(hal::AttachmentOps::STORE_DISCARD) {
                Some(wgt::StoreOp::Discard)
            } else {
                None
            };

            SoftRenderPassDepthStencilAttachment {
                view: a.target.view.clone(),
                depth_load_op,
                depth_store_op,
                depth_clear_value: a.clear_value.0,
                stencil_load_op,
                stencil_store_op,
                stencil_clear_value: a.clear_value.1,
            }
        });

        let pass_desc = SoftRenderPassDescriptor {
            color_attachments,
            depth_stencil_attachment,
        };

        self.current_render_pass = Some((pass_desc, Vec::new()));
        Ok(())
    }

    unsafe fn end_render_pass(&mut self) {
        if let Some((desc, commands)) = self.current_render_pass.take() {
            self.commands
                .push(SoftCommand::RenderPass { desc, commands });
        }
    }

    unsafe fn set_render_pipeline(&mut self, pipeline: &SoftRenderPipeline) {
        if let Some((_, commands)) = &mut self.current_render_pass {
            commands.push(SoftRenderCommand::SetPipeline(Box::new(pipeline.clone())));
        }
    }

    unsafe fn set_index_buffer(
        &mut self,
        binding: hal::BufferBinding<SoftBuffer>,
        format: wgt::IndexFormat,
    ) {
        if let Some((_, commands)) = &mut self.current_render_pass {
            commands.push(SoftRenderCommand::SetIndexBuffer {
                buffer: binding.buffer.clone(),
                offset: binding.offset,
                size: binding.size,
                format,
            });
        }
    }

    unsafe fn set_vertex_buffer(&mut self, index: u32, binding: hal::BufferBinding<SoftBuffer>) {
        if let Some((_, commands)) = &mut self.current_render_pass {
            commands.push(SoftRenderCommand::SetVertexBuffer {
                index,
                buffer: binding.buffer.clone(),
                offset: binding.offset,
                size: binding.size,
            });
        }
    }

    unsafe fn set_viewport(&mut self, rect: &hal::Rect<f32>, depth_range: std::ops::Range<f32>) {
        if let Some((_, commands)) = &mut self.current_render_pass {
            commands.push(SoftRenderCommand::SetViewport {
                x: rect.x,
                y: rect.y,
                width: rect.w,
                height: rect.h,
                min_depth: depth_range.start,
                max_depth: depth_range.end,
            });
        }
    }
    unsafe fn set_scissor_rect(&mut self, rect: &hal::Rect<u32>) {
        if let Some((_, commands)) = &mut self.current_render_pass {
            commands.push(SoftRenderCommand::SetScissorRect {
                x: rect.x,
                y: rect.y,
                width: rect.w,
                height: rect.h,
            });
        }
    }
    unsafe fn set_stencil_reference(&mut self, _reference: u32) {}
    unsafe fn set_blend_constants(&mut self, _color: &[f32; 4]) {}

    unsafe fn draw(
        &mut self,
        start_vertex: u32,
        vertex_count: u32,
        start_instance: u32,
        instance_count: u32,
    ) {
        if let Some((_, commands)) = &mut self.current_render_pass {
            commands.push(SoftRenderCommand::Draw {
                vertex_count,
                instance_count,
                first_vertex: start_vertex,
                first_instance: start_instance,
            });
        }
    }

    unsafe fn draw_indexed(
        &mut self,
        start_index: u32,
        index_count: u32,
        base_vertex: i32,
        start_instance: u32,
        instance_count: u32,
    ) {
        if let Some((_, commands)) = &mut self.current_render_pass {
            commands.push(SoftRenderCommand::DrawIndexed {
                index_count,
                instance_count,
                first_index: start_index,
                base_vertex,
                first_instance: start_instance,
            });
        }
    }

    unsafe fn draw_indirect(
        &mut self,
        _buffer: &SoftBuffer,
        _offset: wgt::BufferAddress,
        _draw_count: u32,
    ) {
    }
    unsafe fn draw_indexed_indirect(
        &mut self,
        _buffer: &SoftBuffer,
        _offset: wgt::BufferAddress,
        _draw_count: u32,
    ) {
    }
    unsafe fn draw_indirect_count(
        &mut self,
        _buffer: &SoftBuffer,
        _offset: wgt::BufferAddress,
        _count_buffer: &SoftBuffer,
        _count_offset: wgt::BufferAddress,
        _max_draw_count: u32,
    ) {
    }
    unsafe fn draw_indexed_indirect_count(
        &mut self,
        _buffer: &SoftBuffer,
        _offset: wgt::BufferAddress,
        _count_buffer: &SoftBuffer,
        _count_offset: wgt::BufferAddress,
        _max_draw_count: u32,
    ) {
    }
    unsafe fn draw_mesh_tasks(
        &mut self,
        _group_count_x: u32,
        _group_count_y: u32,
        _group_count_z: u32,
    ) {
    }
    unsafe fn draw_mesh_tasks_indirect(
        &mut self,
        _buffer: &SoftBuffer,
        _offset: wgt::BufferAddress,
        _draw_count: u32,
    ) {
    }
    unsafe fn draw_mesh_tasks_indirect_count(
        &mut self,
        _buffer: &SoftBuffer,
        _offset: wgt::BufferAddress,
        _count_buffer: &SoftBuffer,
        _count_offset: wgt::BufferAddress,
        _max_draw_count: u32,
    ) {
    }
    unsafe fn begin_compute_pass(&mut self, _desc: &hal::ComputePassDescriptor<SoftQuerySet>) {}
    unsafe fn end_compute_pass(&mut self) {}
    unsafe fn set_compute_pipeline(&mut self, _pipeline: &SoftComputePipeline) {}
    unsafe fn dispatch(&mut self, _count: [u32; 3]) {}
    unsafe fn dispatch_indirect(&mut self, _buffer: &SoftBuffer, _offset: wgt::BufferAddress) {}
    unsafe fn build_acceleration_structures<'a, T>(
        &mut self,
        _descriptor_count: u32,
        _descriptors: T,
    ) where
        T: IntoIterator<
            Item = hal::BuildAccelerationStructureDescriptor<
                'a,
                SoftBuffer,
                SoftAccelerationStructure,
            >,
        >,
    {
    }
    unsafe fn place_acceleration_structure_barrier(
        &mut self,
        _barrier: hal::AccelerationStructureBarrier,
    ) {
    }
    unsafe fn read_acceleration_structure_compact_size(
        &mut self,
        _as: &SoftAccelerationStructure,
        _buffer: &SoftBuffer,
    ) {
    }
}

struct SoftVertexFetcher<'a> {
    vertex_buffers: &'a [VertexBufferEntry],
    vertex_layouts: &'a [SoftVertexBufferLayout],
}

impl<'a> wasm_gl_emu::VertexFetcher for SoftVertexFetcher<'a> {
    fn fetch(
        &self,
        _kernel: &wasm_gl_emu::GpuKernel,
        vertex_index: u32,
        instance_index: u32,
        dest: &mut [u8],
    ) {
        for (i, layout) in self.vertex_layouts.iter().enumerate() {
            if i >= self.vertex_buffers.len() {
                continue;
            }

            if let Some((buffer_data, buffer_offset)) = &self.vertex_buffers[i] {
                let data = buffer_data.lock().unwrap();
                let stride = layout.array_stride as usize;

                let index = match layout.step_mode {
                    wgt::VertexStepMode::Vertex => vertex_index,
                    wgt::VertexStepMode::Instance => instance_index,
                } as usize;

                let start = *buffer_offset as usize + index * stride;

                for attribute in &layout.attributes {
                    let location = attribute.shader_location;
                    let (dest_offset, _) =
                        crate::naga_wasm_backend::output_layout::compute_input_offset(
                            location,
                            naga::ShaderStage::Vertex,
                        );
                    let dest_offset = dest_offset as usize;

                    if dest_offset + 16 > dest.len() {
                        continue;
                    }

                    let attr_offset = attribute.offset as usize;
                    let attr_format = attribute.format;

                    // Read from data[start + attr_offset]
                    let src_start = start + attr_offset;

                    // Simple size mapping
                    let size = match attr_format {
                        wgt::VertexFormat::Float32x4 => 16,
                        wgt::VertexFormat::Float32x3 => 12,
                        wgt::VertexFormat::Float32x2 => 8,
                        wgt::VertexFormat::Float32 => 4,
                        // TODO: Handle other formats
                        _ => 16,
                    };

                    if src_start + size <= data.len() {
                        let src_slice = &data[src_start..src_start + size];
                        let dst_slice = &mut dest[dest_offset..dest_offset + size];
                        dst_slice.copy_from_slice(src_slice);
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct SoftCommandBuffer {
    pub commands: Vec<SoftCommand>,
}

#[derive(Debug, Clone)]
pub struct SoftBuffer {
    pub data: Arc<Mutex<Vec<u8>>>,
    pub size: wgt::BufferAddress,
    pub usage: wgt::BufferUses,
}

#[derive(Debug, Clone)]
pub struct SoftTextureDescriptor {
    pub size: wgt::Extent3d,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub dimension: wgt::TextureDimension,
    pub format: wgt::TextureFormat,
    pub usage: wgt::TextureUses,
}

impl From<&hal::TextureDescriptor<'_>> for SoftTextureDescriptor {
    fn from(desc: &hal::TextureDescriptor) -> Self {
        Self {
            size: desc.size,
            mip_level_count: desc.mip_level_count,
            sample_count: desc.sample_count,
            dimension: desc.dimension,
            format: desc.format,
            usage: desc.usage,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SoftTexture {
    pub data: Arc<Mutex<Vec<u8>>>,
    pub desc: SoftTextureDescriptor,
}

#[derive(Debug, Clone)]
pub struct SoftTextureView {
    pub texture: Arc<Mutex<Vec<u8>>>,
    pub desc: hal::TextureViewDescriptor<'static>,
    pub texture_desc: SoftTextureDescriptor,
}

#[derive(Debug, Clone)]
pub struct SoftSampler {
    pub desc: hal::SamplerDescriptor<'static>,
}

#[derive(Debug, Clone)]
pub struct SoftQuerySet;

#[derive(Debug, Clone)]
pub struct SoftFence {
    pub value: Arc<Mutex<hal::FenceValue>>,
}

#[derive(Debug, Clone)]
pub struct SoftPipelineLayout;

#[derive(Debug, Clone)]
pub struct SoftVertexBufferLayout {
    pub array_stride: wgt::BufferAddress,
    pub step_mode: wgt::VertexStepMode,
    pub attributes: Vec<wgt::VertexAttribute>,
}

#[derive(Debug, Clone)]
pub struct SoftShaderStage {
    pub module: Arc<naga::Module>,
    pub entry_point: String,
    pub wasm_module: WasmModule,
    pub uniform_map:
        HashMap<(u32, u32), (u32, crate::naga_wasm_backend::output_layout::UniformLayout)>,
}

#[derive(Debug, Clone)]
pub struct SoftRenderPipeline {
    pub vertex_stage: SoftShaderStage,
    pub fragment_stage: SoftShaderStage,
    pub vertex_layouts: Vec<SoftVertexBufferLayout>,
    pub color_targets: Vec<Option<wgt::ColorTargetState>>,
    pub depth_stencil: Option<wgt::DepthStencilState>,
    pub primitive: wgt::PrimitiveState,
    pub multisample: wgt::MultisampleState,
}

#[derive(Debug, Clone)]
pub struct SoftComputePipeline;

#[derive(Clone)]
pub struct SoftShaderModule {
    pub module: Arc<naga::Module>,
}

impl std::fmt::Debug for SoftShaderModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SoftShaderModule").finish()
    }
}

#[derive(Debug, Clone)]
pub struct SoftBindGroupLayout {
    pub entries: Vec<wgt::BindGroupLayoutEntry>,
}

#[derive(Debug, Clone)]
pub enum SoftResource {
    Buffer {
        buffer: SoftBuffer,
        offset: wgt::BufferAddress,
        size: Option<wgt::BufferSize>,
    },
    Sampler(SoftSampler),
    TextureView(SoftTextureView),
}

#[derive(Debug, Clone)]
pub struct SoftBindGroup {
    pub resources: HashMap<u32, SoftResource>,
}

#[derive(Debug, Clone)]
pub struct SoftAccelerationStructure;

impl_dyn_resource!(
    SoftInstance,
    SoftSurface,
    SoftAdapter,
    SoftDevice,
    SoftQueue,
    SoftCommandEncoder,
    SoftCommandBuffer,
    SoftBuffer,
    SoftTexture,
    SoftTextureView,
    SoftSampler,
    SoftQuerySet,
    SoftFence,
    SoftPipelineLayout,
    SoftRenderPipeline,
    SoftComputePipeline,
    SoftShaderModule,
    SoftBindGroupLayout,
    SoftBindGroup,
    SoftAccelerationStructure
);

impl hal::DynCommandBuffer for SoftCommandBuffer {}
impl hal::DynBuffer for SoftBuffer {}
impl hal::DynTexture for SoftTexture {}
impl hal::DynTextureView for SoftTextureView {}
impl hal::DynSampler for SoftSampler {}
impl hal::DynQuerySet for SoftQuerySet {}
impl hal::DynFence for SoftFence {}
impl hal::DynPipelineLayout for SoftPipelineLayout {}
impl hal::DynRenderPipeline for SoftRenderPipeline {}
impl hal::DynComputePipeline for SoftComputePipeline {}
impl hal::DynShaderModule for SoftShaderModule {}
impl hal::DynBindGroupLayout for SoftBindGroupLayout {}
impl hal::DynBindGroup for SoftBindGroup {}
impl hal::DynAccelerationStructure for SoftAccelerationStructure {}
impl hal::DynPipelineCache for SoftPipelineLayout {}

impl std::borrow::Borrow<dyn hal::DynTexture> for SoftTexture {
    fn borrow(&self) -> &dyn hal::DynTexture {
        self
    }
}

impl hal::DynSurfaceTexture for SoftTexture {}
