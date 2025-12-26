//! WebGPU Adapter, Instance, and Device initialization

use std::cell::RefCell;
use std::collections::HashMap;
use wgpu_core::global::Global;
use wgpu_core::id::{AdapterId, DeviceId};
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
    pub next_adapter_id: u32,
    pub next_device_id: u32,
}

impl WebGpuContext {
    pub fn new(id: u32) -> Self {
        let mut backend_options = wgt::BackendOptions::default();
        backend_options.noop.enable = true;

        // Create wgpu-core Global with noop backend
        // Note: Backends::NOOP causes wgpu-core to use the noop/empty backend
        // which provides full API validation without rendering
        let instance_desc = wgt::InstanceDescriptor {
            backends: wgt::Backends::NOOP,
            flags: wgt::InstanceFlags::empty(),
            memory_budget_thresholds: wgt::MemoryBudgetThresholds::default(),
            backend_options,
            display: None,
        };

        let global = Global::new("webgpu-wasm", instance_desc, None);

        WebGpuContext {
            id,
            global,
            adapters: HashMap::new(),
            devices: HashMap::new(),
            next_adapter_id: 1,
            next_device_id: 1,
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
                crate::js_log(0, &format!("Failed to request adapter: {:?}", e));
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

        let (device_id, _) =
            match ctx
                .global
                .adapter_request_device(adapter_id, &device_desc, None, None)
            {
                Ok(res) => res,
                Err(e) => {
                    crate::js_log(0, &format!("Failed to request device: {:?}", e));
                    return super::NULL_HANDLE;
                }
            };

        let handle = ctx.next_device_id;
        ctx.next_device_id += 1;
        ctx.devices.insert(handle, device_id);

        handle
    })
}
