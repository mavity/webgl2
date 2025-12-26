//! WebGPU Adapter, Instance, and Device initialization

use std::cell::RefCell;
use std::collections::HashMap;
use wgpu_core::global::Global;
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
}

impl WebGpuContext {
    pub fn new(id: u32) -> Self {
        // Create wgpu-core Global with noop backend
        // Note: Backends::empty() causes wgpu-core to use the noop/empty backend
        // by default, which provides full API validation without rendering
        let instance_desc = wgt::InstanceDescriptor {
            backends: wgt::Backends::empty(), // Empty backends = noop backend
            flags: wgt::InstanceFlags::empty(),
            memory_budget_thresholds: wgt::MemoryBudgetThresholds::default(),
            backend_options: wgt::BackendOptions::default(),
            display: None,
        };

        let global = Global::new("webgpu-wasm", instance_desc, None);

        WebGpuContext { id, global }
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
