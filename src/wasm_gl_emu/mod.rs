//! WebGL2 Software Rasterizer and WASM Shader Runtime
//!
//! This module provides a software implementation of WebGL2 that executes
//! compiled WASM shaders in a CPU-based rasterizer, enabling full debugging
//! capabilities.

pub mod device;
pub mod transfer;
mod framebuffer;
mod pipeline;
pub mod rasterizer;
mod state;
mod texture;

pub use device::{GpuBuffer, GpuHandle, GpuKernel, StorageLayout};
pub use transfer::{IndexType, TransferEngine, TransferRequest};
pub use framebuffer::{Framebuffer, OwnedFramebuffer};
pub use pipeline::{Pipeline, VertexOutput};
pub use rasterizer::{
    ProcessedVertex, RasterPipeline, Rasterizer, RenderState, ShaderMemoryLayout, VertexFetcher,
};
pub use state::WebGLState;
pub use texture::Texture;

/// Initialize the emulator with default configuration
pub fn init() -> WebGLState {
    tracing::info!("Initializing WebGL2 emulator");
    WebGLState::new()
}
