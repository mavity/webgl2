//! WebGPU API implementation
//!
//! A complete WebGPU API surface that runs entirely in WebAssembly/Rust,
//! providing deterministic execution, advanced debugging, and software
//! rasterization of WebGPU workloads.

pub mod adapter;
pub mod backend;
pub mod bind_group;
pub mod buffer;
pub mod command;
pub mod pipeline;
pub mod shader;
pub mod texture;

#[cfg(test)]
mod tests;

// WebGPU context handle type
pub type ContextHandle = u32;

// WebGPU object handle types
pub type AdapterHandle = u32;
pub type DeviceHandle = u32;
pub type QueueHandle = u32;
pub type BufferHandle = u32;
pub type TextureHandle = u32;
pub type ShaderModuleHandle = u32;
pub type PipelineLayoutHandle = u32;
pub type BindGroupLayoutHandle = u32;
pub type BindGroupHandle = u32;
pub type RenderPipelineHandle = u32;
pub type ComputePipelineHandle = u32;
pub type CommandEncoderHandle = u32;
pub type CommandBufferHandle = u32;

// Reserved handle value for "null" or invalid handles
pub const NULL_HANDLE: u32 = 0;

// Error codes
pub const WEBGPU_SUCCESS: u32 = 0;
pub const WEBGPU_ERROR_INVALID_HANDLE: u32 = 1;
pub const WEBGPU_ERROR_OUT_OF_MEMORY: u32 = 2;
pub const WEBGPU_ERROR_VALIDATION: u32 = 3;
pub const WEBGPU_ERROR_OPERATION_FAILED: u32 = 4;
