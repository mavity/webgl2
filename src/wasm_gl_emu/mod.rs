//! WebGL2 Software Rasterizer and WASM Shader Runtime
//!
//! This module provides a software implementation of WebGL2 that executes
//! compiled WASM shaders in a CPU-based rasterizer, enabling full debugging
//! capabilities.

// Runtime selection: wasmi (for wasm32 and native)
mod runtime_wasmi;

pub mod device;
pub mod imaging;
mod framebuffer;
mod pipeline;
pub mod rasterizer;
mod state;
mod texture;

pub use runtime_wasmi::RuntimeError;
pub use runtime_wasmi::ShaderRuntime;

pub use device::{GpuBuffer, GpuHandle, GpuKernel, StorageLayout};
pub use imaging::TransferEngine;
pub use framebuffer::{Framebuffer, OwnedFramebuffer};
pub use pipeline::{Pipeline, VertexOutput};
pub use rasterizer::{
    ProcessedVertex, RasterPipeline, Rasterizer, RenderState, ShaderMemoryLayout, VertexFetcher,
};
pub use state::WebGLState;
pub use texture::Texture;

/// Initialize the emulator with default configuration
pub fn init() -> Result<WebGLState, RuntimeError> {
    tracing::info!("Initializing WebGL2 emulator");
    WebGLState::new()
}
