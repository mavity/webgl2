//! Naga IR to WebAssembly Backend with DWARF Debug Information
//!
//! This module provides a backend for the Naga shader IR that compiles to WebAssembly
//! bytecode with embedded DWARF debug information for browser DevTools integration.

mod backend;
mod builtins;
mod control_flow;
pub mod debug;
mod expressions;
pub mod types;

use naga::{valid::ModuleInfo, Module};
use std::collections::HashMap;

/// Configuration for WASM generation
#[derive(Debug, Clone)]
pub struct WasmBackendConfig {
    /// Include DWARF debug information
    pub debug_info: bool,
    /// Optimize generated WASM (future: dead code elimination, constant folding)
    pub optimize: bool,
    /// Target WASM features (SIMD, threads, etc.)
    pub features: WasmFeatures,
}

impl Default for WasmBackendConfig {
    fn default() -> Self {
        Self {
            debug_info: true,
            optimize: false,
            features: WasmFeatures::default(),
        }
    }
}

/// WebAssembly target features
#[derive(Debug, Clone, Default)]
pub struct WasmFeatures {
    /// Enable SIMD instructions (v128)
    pub simd: bool,
    /// Enable bulk memory operations
    pub bulk_memory: bool,
    /// Enable reference types
    pub reference_types: bool,
}

/// Output of WASM compilation
#[derive(Debug)]
pub struct WasmModule {
    /// WASM bytecode
    pub wasm_bytes: Vec<u8>,
    /// Separate DWARF debug data (optional, embedded in custom sections)
    pub dwarf_bytes: Option<Vec<u8>>,
    /// Entry point function names mapped to function indices
    pub entry_points: HashMap<String, u32>,
    /// Memory layout information
    pub memory_layout: MemoryLayout,
}

/// Memory layout for shader execution
#[derive(Debug, Clone)]
pub struct MemoryLayout {
    /// Stack size (for local variables)
    pub stack_size: u32,
    /// Uniform buffer offset
    pub uniform_offset: u32,
    /// Uniform buffer size
    pub uniform_size: u32,
    /// Attribute buffer offset
    pub attribute_offset: u32,
    /// Attribute buffer size
    pub attribute_size: u32,
    /// Varying buffer offset
    pub varying_offset: u32,
    /// Varying buffer size
    pub varying_size: u32,
}

impl Default for MemoryLayout {
    fn default() -> Self {
        Self {
            stack_size: 0x1000,       // 4KB stack
            uniform_offset: 0x1000,   // Uniforms start at 4KB
            uniform_size: 0x1000,     // 4KB for uniforms
            attribute_offset: 0x2000, // Attributes at 8KB
            attribute_size: 0x1000,   // 4KB for attributes
            varying_offset: 0x3000,   // Varyings at 12KB
            varying_size: 0x1000,     // 4KB for varyings
        }
    }
}

/// Main backend interface
pub struct WasmBackend {
    config: WasmBackendConfig,
}

impl WasmBackend {
    /// Create a new WASM backend with the given configuration
    pub fn new(config: WasmBackendConfig) -> Self {
        Self { config }
    }

    /// Compile a Naga module to WASM
    ///
    /// # Arguments
    ///
    /// * `module` - The validated Naga IR module
    /// * `info` - Validation information from Naga
    /// * `source` - Original GLSL source code (for DWARF line mappings)
    ///
    /// # Returns
    ///
    /// A compiled WASM module with optional debug information
    pub fn compile(
        &self,
        module: &Module,
        info: &ModuleInfo,
        source: &str,
        stage: naga::ShaderStage,
        uniform_locations: &HashMap<String, u32>,
        varying_locations: &HashMap<String, u32>,
    ) -> Result<WasmModule, BackendError> {
        backend::compile_module(
            self,
            module,
            info,
            source,
            stage,
            uniform_locations,
            varying_locations,
        )
    }
}

/// Error types for WASM backend compilation
#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("Unsupported Naga IR feature: {0}")]
    UnsupportedFeature(String),

    #[error("WASM encoding error: {0}")]
    WasmEncoding(String),

    #[error("DWARF generation error: {0}")]
    DwarfGeneration(String),

    #[error("Type conversion error: {0}")]
    TypeConversion(String),

    #[error("Invalid function signature: {0}")]
    InvalidSignature(String),

    #[error("Internal compiler error: {0}")]
    InternalError(String),
}
