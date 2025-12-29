//! WebGPU Shader Module management

use super::adapter::with_context;
use std::borrow::Cow;
use wgpu_core::pipeline;
use wgpu_types as wgt;

/// Create a new shader module
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers for shader code.
pub unsafe fn create_shader_module(
    ctx_handle: u32,
    device_handle: u32,
    code_ptr: *const u8,
    code_len: usize,
) -> u32 {
    let code = {
        let slice = std::slice::from_raw_parts(code_ptr, code_len);
        std::str::from_utf8_unchecked(slice)
    };

    with_context(ctx_handle, |ctx| {
        let device_id = match ctx.devices.get(&device_handle) {
            Some(id) => *id,
            None => return super::NULL_HANDLE,
        };

        let source = pipeline::ShaderModuleSource::Wgsl(Cow::Borrowed(code));
        let desc = pipeline::ShaderModuleDescriptor {
            label: None,
            runtime_checks: wgt::ShaderRuntimeChecks::default(),
        };

        let (shader_id, error) = ctx
            .global
            .device_create_shader_module(device_id, &desc, source, None);
        if let Some(e) = error {
            // TODO: handle properly, propagate error
            return super::NULL_HANDLE;
        }

        let handle = ctx.next_shader_module_id;
        ctx.next_shader_module_id += 1;
        ctx.shader_modules.insert(handle, shader_id);

        handle
    })
}
