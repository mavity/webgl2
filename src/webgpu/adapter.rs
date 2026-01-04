//! WebGPU Adapter, Instance, and Device initialization

use std::cell::RefCell;
use std::collections::HashMap;
use wgpu_core::global::Global;
use wgpu_core::id::{
    AdapterId, BindGroupId, BindGroupLayoutId, BufferId, CommandBufferId, CommandEncoderId,
    ComputePipelineId, DeviceId, PipelineLayoutId, QueueId, RenderPipelineId, SamplerId,
    ShaderModuleId, TextureId, TextureViewId,
};
use wgpu_types as wgt;

thread_local! {
    // Thread-local storage for WebGPU contexts
    // This is safe because WASM is single-threaded
    static WEBGPU_CONTEXTS: RefCell<HashMap<u32, WebGpuContext>> = RefCell::new(HashMap::new());
    static NEXT_CONTEXT_ID: RefCell<u32> = const { RefCell::new(1) };
}

/// WebGPU context state
pub struct WebGpuContext {
    pub id: u32,
    pub global: Global,
    pub adapters: HashMap<u32, AdapterId>,
    pub devices: HashMap<u32, DeviceId>,
    pub queues: HashMap<u32, QueueId>,
    pub buffers: HashMap<u32, BufferId>,
    pub shader_modules: HashMap<u32, ShaderModuleId>,
    pub pipeline_layouts: HashMap<u32, PipelineLayoutId>,
    pub bind_group_layouts: HashMap<u32, BindGroupLayoutId>,
    pub bind_groups: HashMap<u32, BindGroupId>,
    pub render_pipelines: HashMap<u32, RenderPipelineId>,
    pub compute_pipelines: HashMap<u32, ComputePipelineId>,
    pub command_encoders: HashMap<u32, CommandEncoderId>,
    pub command_buffers: HashMap<u32, CommandBufferId>,
    pub textures: HashMap<u32, TextureId>,
    pub texture_views: HashMap<u32, TextureViewId>,
    pub samplers: HashMap<u32, SamplerId>,

    pub next_adapter_id: u32,
    pub next_device_id: u32,
    pub next_buffer_id: u32,
    pub next_shader_module_id: u32,
    pub next_pipeline_layout_id: u32,
    pub next_bind_group_layout_id: u32,
    pub next_bind_group_id: u32,
    pub next_render_pipeline_id: u32,
    pub next_compute_pipeline_id: u32,
    pub next_command_encoder_id: u32,
    pub next_command_buffer_id: u32,
    pub next_texture_id: u32,
    pub next_texture_view_id: u32,
    pub next_sampler_id: u32,
}

impl WebGpuContext {
    pub fn new(id: u32) -> Self {
        // Initialize our custom backend instance
        let soft_instance = crate::webgpu::backend::SoftInstance;

        // Create Global using our custom backend
        // unsafe: We are responsible for the lifetime of the instance, which Global takes ownership of
        let global = unsafe {
            Global::from_hal_instance::<crate::webgpu::backend::SoftApi>(
                "webgpu-wasm",
                soft_instance,
            )
        };

        WebGpuContext {
            id,
            global,
            adapters: HashMap::new(),
            devices: HashMap::new(),
            queues: HashMap::new(),
            buffers: HashMap::new(),
            shader_modules: HashMap::new(),
            pipeline_layouts: HashMap::new(),
            bind_group_layouts: HashMap::new(),
            bind_groups: HashMap::new(),
            render_pipelines: HashMap::new(),
            compute_pipelines: HashMap::new(),
            command_encoders: HashMap::new(),
            command_buffers: HashMap::new(),
            textures: HashMap::new(),
            texture_views: HashMap::new(),
            samplers: HashMap::new(),

            next_adapter_id: 1,
            next_device_id: 1,
            next_buffer_id: 1,
            next_shader_module_id: 1,
            next_pipeline_layout_id: 1,
            next_bind_group_layout_id: 1,
            next_bind_group_id: 1,
            next_render_pipeline_id: 1,
            next_compute_pipeline_id: 1,
            next_command_encoder_id: 1,
            next_command_buffer_id: 1,
            next_texture_id: 1,
            next_texture_view_id: 1,
            next_sampler_id: 1,
        }
    }
}

/// Create a new WebGPU context
pub fn create_context() -> u32 {
    NEXT_CONTEXT_ID.with(|next_id| {
        let id = *next_id.borrow();
        *next_id.borrow_mut() = id + 1;

        let ctx = WebGpuContext::new(id);

        WEBGPU_CONTEXTS.with(|contexts| {
            contexts.borrow_mut().insert(id, ctx);
        });

        id
    })
}

/// Execute a closure with a mutable reference to a WebGPU context
pub fn with_context<F, R>(handle: u32, f: F) -> R
where
    F: FnOnce(&mut WebGpuContext) -> R,
    R: From<u32>,
{
    WEBGPU_CONTEXTS.with(|contexts| {
        let mut contexts = contexts.borrow_mut();
        if let Some(ctx) = contexts.get_mut(&handle) {
            f(ctx)
        } else {
            super::NULL_HANDLE.into()
        }
    })
}

/// Execute a closure with a mutable reference to a WebGPU context, returning a default value on failure
pub fn with_context_val<F, R>(handle: u32, default: R, f: F) -> R
where
    F: FnOnce(&mut WebGpuContext) -> R,
{
    WEBGPU_CONTEXTS.with(|contexts| {
        let mut contexts = contexts.borrow_mut();
        if let Some(ctx) = contexts.get_mut(&handle) {
            f(ctx)
        } else {
            default
        }
    })
}

/// Destroy a WebGPU context
pub fn destroy_context(handle: u32) -> u32 {
    WEBGPU_CONTEXTS.with(|contexts| {
        if contexts.borrow_mut().remove(&handle).is_some() {
            super::WEBGPU_SUCCESS
        } else {
            super::WEBGPU_ERROR_INVALID_HANDLE
        }
    })
}

/// Request an adapter
pub fn request_adapter(ctx_handle: u32, power_preference: wgt::PowerPreference) -> u32 {
    WEBGPU_CONTEXTS.with(|contexts| {
        let mut contexts = contexts.borrow_mut();
        let ctx = match contexts.get_mut(&ctx_handle) {
            Some(ctx) => ctx,
            None => return super::NULL_HANDLE,
        };

        let adapter_id = match ctx.global.request_adapter(
            &wgt::RequestAdapterOptions {
                power_preference,
                force_fallback_adapter: false,
                compatible_surface: None,
            },
            wgt::Backends::NOOP,
            None,
        ) {
            Ok(id) => id,
            Err(e) => {
                crate::error::set_error(
                    crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                    super::WEBGPU_ERROR_VALIDATION,
                    format!("Failed to request adapter: {}", e),
                );
                return super::NULL_HANDLE;
            }
        };

        let handle = ctx.next_adapter_id;
        ctx.next_adapter_id += 1;
        ctx.adapters.insert(handle, adapter_id);

        handle
    })
}

/// Request a device
pub fn request_device(ctx_handle: u32, adapter_handle: u32) -> u32 {
    WEBGPU_CONTEXTS.with(|contexts| {
        let mut contexts = contexts.borrow_mut();
        let ctx = match contexts.get_mut(&ctx_handle) {
            Some(ctx) => ctx,
            None => return super::NULL_HANDLE,
        };

        let adapter_id = match ctx.adapters.get(&adapter_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let device_desc = wgt::DeviceDescriptor {
            label: None,
            required_features: wgt::Features::empty(),
            required_limits: wgt::Limits::default(),
            memory_hints: wgt::MemoryHints::default(),
            experimental_features: wgt::ExperimentalFeatures::default(),
            trace: wgt::Trace::Off,
        };

        let (device_id, queue_id) =
            match ctx
                .global
                .adapter_request_device(adapter_id, &device_desc, None, None)
            {
                Ok(res) => res,
                Err(e) => {
                    crate::error::set_error(
                        crate::error::ErrorSource::WebGPU(crate::error::WebGPUErrorFilter::Validation),
                        super::WEBGPU_ERROR_VALIDATION,
                        format!("Failed to request device: {}", e),
                    );
                    return super::NULL_HANDLE;
                }
            };

        let handle = ctx.next_device_id;
        ctx.next_device_id += 1;
        ctx.devices.insert(handle, device_id);
        ctx.queues.insert(handle, queue_id); // Use same handle for default queue for now

        handle
    })
}

/// Destroy a device
pub fn destroy_device(ctx_handle: u32, device_handle: u32) -> u32 {
    with_context(ctx_handle, |ctx| {
        if ctx.devices.remove(&device_handle).is_some() {
            super::WEBGPU_SUCCESS
        } else {
            super::WEBGPU_ERROR_INVALID_HANDLE
        }
    })
}
