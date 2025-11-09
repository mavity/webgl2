//! WebGL2 Shader Compiler and Emulator
//!
//! A complete toolchain for compiling GLSL shaders to WebAssembly with DWARF debugging,
//! including a software rasterizer for shader emulation and TypeScript harness generation.
//!
//! # Modules
//!
//! - [`naga_wasm_backend`] - Compile Naga IR to WASM with DWARF debug information
//! - [`wasm_gl_emu`] - Software rasterizer and WASM shader runtime
//! - [`glsl_introspection`] - GLSL parser with annotation extraction
//! - [`js_codegen`] - TypeScript harness code generator

pub mod naga_wasm_backend;
pub mod wasm_gl_emu;
pub mod glsl_introspection;
pub mod js_codegen;

// Re-export commonly used types
pub use naga_wasm_backend::{WasmBackend, WasmBackendConfig, WasmModule, BackendError};
#[cfg(feature = "cli")]
pub use wasm_gl_emu::ShaderRuntime;
pub use wasm_gl_emu::RuntimeError;
pub use glsl_introspection::ResourceManifest;
pub use js_codegen::generate_harness;
