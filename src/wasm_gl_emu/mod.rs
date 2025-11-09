//! WebGL2 Software Rasterizer and WASM Shader Runtime
//!
//! This module provides a software implementation of WebGL2 that executes
//! compiled WASM shaders in a CPU-based rasterizer, enabling full debugging
//! capabilities.

// Runtime selection: native (wasmtime) or web (wasm-bindgen)
#[cfg(feature = "cli")]
mod runtime_native;

#[cfg(feature = "web")]
mod runtime_web;

// Default to native runtime when no feature is specified
#[cfg(not(any(feature = "cli", feature = "web")))]
mod runtime_native;

mod framebuffer;
mod pipeline;
mod rasterizer;
mod state;
mod texture;

#[cfg(feature = "cli")]
pub use runtime_native::ShaderRuntime;
pub use runtime_native::RuntimeError;

#[cfg(feature = "web")]
pub use runtime_web::instantiate_shader;

pub use framebuffer::Framebuffer;
pub use pipeline::{Pipeline, VertexOutput};
pub use rasterizer::Rasterizer;
pub use state::WebGLState;
pub use texture::Texture;

/// Initialize the emulator with default configuration
pub fn init() -> Result<WebGLState, RuntimeError> {
    tracing::info!("Initializing WebGL2 emulator");
    WebGLState::new()
}
