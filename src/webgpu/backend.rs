use std::sync::{Arc, Mutex};
use std::num::NonZero;
use std::time::Duration;
use std::any::Any;
use wgpu_hal as hal;
use wgpu_types as wgt;

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

pub struct SoftInstance;

impl hal::Instance for SoftInstance {
    type A = SoftApi;

    unsafe fn init(_desc: &hal::InstanceDescriptor) -> Result<Self, hal::InstanceError> {
        Ok(SoftInstance)
    }

    unsafe fn create_surface(
        &self,
        _display_handle: raw_window_handle::RawDisplayHandle,
        _window_handle: raw_window_handle::RawWindowHandle,
    ) -> Result<SoftSurface, hal::InstanceError> {
        Ok(SoftSurface)
    }

    unsafe fn enumerate_adapters(&self, _surface_hint: Option<&SoftSurface>) -> Vec<hal::ExposedAdapter<SoftApi>> {
        let adapter = SoftAdapter {
            handle: 0, // Dummy handle
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

    unsafe fn unconfigure(&self, _device: &SoftDevice) {
    }

    unsafe fn acquire_texture(
        &self,
        _timeout: Option<std::time::Duration>,
        _fence: &SoftFence,
    ) -> Result<Option<hal::AcquiredSurfaceTexture<SoftApi>>, hal::SurfaceError> {
        // For now, return nothing or a dummy texture
        Ok(None)
    }

    unsafe fn discard_texture(&self, _texture: SoftTexture) {
    }
}

#[derive(Debug)]
pub struct SoftAdapter {
    handle: u32,
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
            mem_allocator: Arc::new(Mutex::new(0)), // Simple allocator
        };
        let queue = SoftQueue;
        Ok(hal::OpenDevice { device, queue })
    }

    unsafe fn texture_format_capabilities(
        &self,
        _format: wgt::TextureFormat,
    ) -> hal::TextureFormatCapabilities {
        hal::TextureFormatCapabilities::all()
    }

    unsafe fn surface_capabilities(&self, _surface: &SoftSurface) -> Option<hal::SurfaceCapabilities> {
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
    mem_allocator: Arc<Mutex<u32>>,
}

impl hal::Device for SoftDevice {
    type A = SoftApi;

    unsafe fn create_buffer(
        &self,
        desc: &hal::BufferDescriptor,
    ) -> Result<SoftBuffer, hal::DeviceError> {
        // Allocate memory for the buffer
        let size = desc.size as usize;
        let mut data = Vec::with_capacity(size);
        data.resize(size, 0);

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

    unsafe fn unmap_buffer(&self, _buffer: &SoftBuffer) {
    }

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
        Ok(SoftCommandEncoder)
    }

    unsafe fn create_bind_group_layout(
        &self,
        _desc: &hal::BindGroupLayoutDescriptor,
    ) -> Result<SoftBindGroupLayout, hal::DeviceError> {
        Ok(SoftBindGroupLayout)
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
        _shader: hal::ShaderInput,
    ) -> Result<SoftShaderModule, hal::ShaderError> {
        Ok(SoftShaderModule)
    }

    unsafe fn create_render_pipeline(
        &self,
        _desc: &hal::RenderPipelineDescriptor<SoftPipelineLayout, SoftShaderModule, SoftPipelineLayout>,
    ) -> Result<SoftRenderPipeline, hal::PipelineError> {
        Ok(SoftRenderPipeline)
    }

    unsafe fn create_compute_pipeline(
        &self,
        _desc: &hal::ComputePipelineDescriptor<SoftPipelineLayout, SoftShaderModule, SoftPipelineLayout>,
    ) -> Result<SoftComputePipeline, hal::PipelineError> {
        Ok(SoftComputePipeline)
    }

    unsafe fn create_bind_group(
        &self,
        _desc: &hal::BindGroupDescriptor<SoftBindGroupLayout, SoftBuffer, SoftSampler, SoftTextureView, SoftAccelerationStructure>,
    ) -> Result<SoftBindGroup, hal::DeviceError> {
        Ok(SoftBindGroup)
    }

    unsafe fn create_texture(
        &self,
        _desc: &hal::TextureDescriptor,
    ) -> Result<SoftTexture, hal::DeviceError> {
        Ok(SoftTexture)
    }

    unsafe fn destroy_texture(&self, _texture: SoftTexture) {}

    unsafe fn create_texture_view(
        &self,
        _texture: &SoftTexture,
        _desc: &hal::TextureViewDescriptor,
    ) -> Result<SoftTextureView, hal::DeviceError> {
        Ok(SoftTextureView)
    }

    unsafe fn create_sampler(
        &self,
        _desc: &hal::SamplerDescriptor,
    ) -> Result<SoftSampler, hal::DeviceError> {
        Ok(SoftSampler)
    }

    unsafe fn create_query_set(
        &self,
        _desc: &wgt::QuerySetDescriptor<hal::Label>,
    ) -> Result<SoftQuerySet, hal::DeviceError> {
        Ok(SoftQuerySet)
    }

    unsafe fn create_fence(&self) -> Result<SoftFence, hal::DeviceError> {
        Ok(SoftFence)
    }

    unsafe fn get_fence_value(&self, _fence: &SoftFence) -> Result<hal::FenceValue, hal::DeviceError> {
        Ok(0)
    }

    unsafe fn wait(
        &self,
        _fence: &SoftFence,
        _value: hal::FenceValue,
        _timeout_ms: Option<Duration>,
    ) -> Result<bool, hal::DeviceError> {
        Ok(true)
    }


    // Missing methods implementation
    unsafe fn add_raw_buffer(&self, _buffer: &SoftBuffer) { todo!() }
    unsafe fn add_raw_texture(&self, _texture: &SoftTexture) { todo!() }
    unsafe fn destroy_texture_view(&self, _view: SoftTextureView) {}
    unsafe fn destroy_sampler(&self, _sampler: SoftSampler) {}
    unsafe fn destroy_bind_group_layout(&self, _bg_layout: SoftBindGroupLayout) {}
    unsafe fn destroy_pipeline_layout(&self, _pipeline_layout: SoftPipelineLayout) {}
    unsafe fn destroy_bind_group(&self, _group: SoftBindGroup) {}
    unsafe fn destroy_shader_module(&self, _module: SoftShaderModule) {}
    unsafe fn destroy_render_pipeline(&self, _pipeline: SoftRenderPipeline) {}
    unsafe fn destroy_compute_pipeline(&self, _pipeline: SoftComputePipeline) {}
    unsafe fn create_pipeline_cache(&self, _desc: &hal::PipelineCacheDescriptor) -> Result<SoftPipelineLayout, hal::PipelineCacheError> { Ok(SoftPipelineLayout) }
    unsafe fn destroy_pipeline_cache(&self, _cache: SoftPipelineLayout) {}
    unsafe fn destroy_query_set(&self, _set: SoftQuerySet) {}
    unsafe fn destroy_fence(&self, _fence: SoftFence) {}
    unsafe fn start_graphics_debugger_capture(&self) -> bool { false }
    unsafe fn stop_graphics_debugger_capture(&self) {}
    unsafe fn create_acceleration_structure(&self, _desc: &hal::AccelerationStructureDescriptor) -> Result<SoftAccelerationStructure, hal::DeviceError> { Ok(SoftAccelerationStructure) }
    unsafe fn get_acceleration_structure_build_sizes(&self, _desc: &hal::GetAccelerationStructureBuildSizesDescriptor<SoftBuffer>) -> hal::AccelerationStructureBuildSizes { hal::AccelerationStructureBuildSizes::default() }
    unsafe fn get_acceleration_structure_device_address(&self, _as: &SoftAccelerationStructure) -> wgt::BufferAddress { 0 }
    unsafe fn destroy_acceleration_structure(&self, _as: SoftAccelerationStructure) {}
    fn tlas_instance_to_bytes(&self, _instance: hal::TlasInstance) -> Vec<u8> { vec![] }
    fn get_internal_counters(&self) -> wgt::HalCounters { wgt::HalCounters::default() }
    fn check_if_oom(&self) -> Result<(), hal::DeviceError> { Ok(()) }
}

#[derive(Debug)]
pub struct SoftQueue;

impl hal::Queue for SoftQueue {
    type A = SoftApi;

    unsafe fn submit(
        &self,
        _command_buffers: &[&SoftCommandBuffer],
        _surface_textures: &[&SoftTexture],
        _fence: (&mut SoftFence, hal::FenceValue),
    ) -> Result<(), hal::DeviceError> {
        // Execute commands here
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
pub struct SoftCommandEncoder;

impl hal::CommandEncoder for SoftCommandEncoder {
    type A = SoftApi;

    unsafe fn begin_encoding(&mut self, _label: hal::Label) -> Result<(), hal::DeviceError> {
        Ok(())
    }

    unsafe fn discard_encoding(&mut self) {}

    unsafe fn end_encoding(&mut self) -> Result<SoftCommandBuffer, hal::DeviceError> {
        Ok(SoftCommandBuffer)
    }

    unsafe fn reset_all<I>(&mut self, _command_buffers: I)
    where
        I: Iterator<Item = SoftCommandBuffer>,
    {
    }

    // Missing methods implementation
    unsafe fn transition_buffers<'a, T>(&mut self, _barriers: T) where T: Iterator<Item = hal::BufferBarrier<'a, SoftBuffer>> {}
    unsafe fn transition_textures<'a, T>(&mut self, _barriers: T) where T: Iterator<Item = hal::TextureBarrier<'a, SoftTexture>> {}
    unsafe fn clear_buffer(&mut self, _buffer: &SoftBuffer, _range: std::ops::Range<wgt::BufferAddress>) {}
    unsafe fn copy_buffer_to_buffer<T>(&mut self, _src: &SoftBuffer, _dst: &SoftBuffer, _regions: T) where T: Iterator<Item = hal::BufferCopy> {}
    unsafe fn copy_texture_to_texture<T>(&mut self, _src: &SoftTexture, _src_usage: wgt::TextureUses, _dst: &SoftTexture, _regions: T) where T: Iterator<Item = hal::TextureCopy> {}
    unsafe fn copy_buffer_to_texture<T>(&mut self, _src: &SoftBuffer, _dst: &SoftTexture, _regions: T) where T: Iterator<Item = hal::BufferTextureCopy> {}
    unsafe fn copy_texture_to_buffer<T>(&mut self, _src: &SoftTexture, _src_usage: wgt::TextureUses, _dst: &SoftBuffer, _regions: T) where T: Iterator<Item = hal::BufferTextureCopy> {}
    unsafe fn copy_acceleration_structure_to_acceleration_structure(&mut self, _src: &SoftAccelerationStructure, _dst: &SoftAccelerationStructure, _copy: wgt::AccelerationStructureCopy) {}
    unsafe fn set_bind_group(&mut self, _layout: &SoftPipelineLayout, _index: u32, _group: &SoftBindGroup, _dynamic_offsets: &[u32]) {}
    unsafe fn set_immediates(&mut self, _layout: &SoftPipelineLayout, _index: u32, _data: &[u32]) {}
    unsafe fn insert_debug_marker(&mut self, _label: &str) {}
    unsafe fn begin_debug_marker(&mut self, _group_label: &str) {}
    unsafe fn end_debug_marker(&mut self) {}
    unsafe fn begin_query(&mut self, _set: &SoftQuerySet, _index: u32) {}
    unsafe fn end_query(&mut self, _set: &SoftQuerySet, _index: u32) {}
    unsafe fn write_timestamp(&mut self, _set: &SoftQuerySet, _index: u32) {}
    unsafe fn reset_queries(&mut self, _set: &SoftQuerySet, _range: std::ops::Range<u32>) {}
    unsafe fn copy_query_results(&mut self, _set: &SoftQuerySet, _range: std::ops::Range<u32>, _buffer: &SoftBuffer, _offset: wgt::BufferAddress, _stride: NonZero<wgt::BufferAddress>) {}
    unsafe fn begin_render_pass(&mut self, _desc: &hal::RenderPassDescriptor<SoftQuerySet, SoftTextureView>) -> Result<(), hal::DeviceError> { Ok(()) }
    unsafe fn end_render_pass(&mut self) {}
    unsafe fn set_render_pipeline(&mut self, _pipeline: &SoftRenderPipeline) {}
    unsafe fn set_index_buffer(&mut self, _binding: hal::BufferBinding<SoftBuffer>, _format: wgt::IndexFormat) {}
    unsafe fn set_vertex_buffer(&mut self, _index: u32, _binding: hal::BufferBinding<SoftBuffer>) {}
    unsafe fn set_viewport(&mut self, _rect: &hal::Rect<f32>, _depth_range: std::ops::Range<f32>) {}
    unsafe fn set_scissor_rect(&mut self, _rect: &hal::Rect<u32>) {}
    unsafe fn set_stencil_reference(&mut self, _reference: u32) {}
    unsafe fn set_blend_constants(&mut self, _color: &[f32; 4]) {}
    unsafe fn draw(&mut self, _start_vertex: u32, _vertex_count: u32, _start_instance: u32, _instance_count: u32) {}
    unsafe fn draw_indexed(&mut self, _start_index: u32, _index_count: u32, _base_vertex: i32, _start_instance: u32, _instance_count: u32) {}
    unsafe fn draw_indirect(&mut self, _buffer: &SoftBuffer, _offset: wgt::BufferAddress, _draw_count: u32) {}
    unsafe fn draw_indexed_indirect(&mut self, _buffer: &SoftBuffer, _offset: wgt::BufferAddress, _draw_count: u32) {}
    unsafe fn draw_indirect_count(&mut self, _buffer: &SoftBuffer, _offset: wgt::BufferAddress, _count_buffer: &SoftBuffer, _count_offset: wgt::BufferAddress, _max_draw_count: u32) {}
    unsafe fn draw_indexed_indirect_count(&mut self, _buffer: &SoftBuffer, _offset: wgt::BufferAddress, _count_buffer: &SoftBuffer, _count_offset: wgt::BufferAddress, _max_draw_count: u32) {}
    unsafe fn draw_mesh_tasks(&mut self, _group_count_x: u32, _group_count_y: u32, _group_count_z: u32) {}
    unsafe fn draw_mesh_tasks_indirect(&mut self, _buffer: &SoftBuffer, _offset: wgt::BufferAddress, _draw_count: u32) {}
    unsafe fn draw_mesh_tasks_indirect_count(&mut self, _buffer: &SoftBuffer, _offset: wgt::BufferAddress, _count_buffer: &SoftBuffer, _count_offset: wgt::BufferAddress, _max_draw_count: u32) {}
    unsafe fn begin_compute_pass(&mut self, _desc: &hal::ComputePassDescriptor<SoftQuerySet>) {}
    unsafe fn end_compute_pass(&mut self) {}
    unsafe fn set_compute_pipeline(&mut self, _pipeline: &SoftComputePipeline) {}
    unsafe fn dispatch(&mut self, _count: [u32; 3]) {}
    unsafe fn dispatch_indirect(&mut self, _buffer: &SoftBuffer, _offset: wgt::BufferAddress) {}
    unsafe fn build_acceleration_structures<'a, T>(&mut self, _descriptor_count: u32, _descriptors: T) where T: IntoIterator<Item = hal::BuildAccelerationStructureDescriptor<'a, SoftBuffer, SoftAccelerationStructure>> {}
    unsafe fn place_acceleration_structure_barrier(&mut self, _barrier: hal::AccelerationStructureBarrier) {}
    unsafe fn read_acceleration_structure_compact_size(&mut self, _as: &SoftAccelerationStructure, _buffer: &SoftBuffer) {}
}

#[derive(Debug)]
pub struct SoftCommandBuffer;

#[derive(Debug)]
pub struct SoftBuffer {
    pub data: Arc<Mutex<Vec<u8>>>,
    pub size: wgt::BufferAddress,
    pub usage: wgt::BufferUses,
}

#[derive(Debug)]
pub struct SoftTexture;

#[derive(Debug)]
pub struct SoftTextureView;

#[derive(Debug)]
pub struct SoftSampler;

#[derive(Debug)]
pub struct SoftQuerySet;

#[derive(Debug)]
pub struct SoftFence;

#[derive(Debug)]
pub struct SoftPipelineLayout;

#[derive(Debug)]
pub struct SoftRenderPipeline;

#[derive(Debug)]
pub struct SoftComputePipeline;

#[derive(Debug)]
pub struct SoftShaderModule;

#[derive(Debug)]
pub struct SoftBindGroupLayout;

#[derive(Debug)]
pub struct SoftBindGroup;

#[derive(Debug)]
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

