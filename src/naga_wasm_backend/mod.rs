//! Naga IR to WebAssembly Backend with DWARF Debug Information
//!
//! This module provides a backend for the Naga shader IR that compiles to WebAssembly
//! bytecode with embedded DWARF debug information for browser DevTools integration.

mod backend;
mod builtins;
mod call_lowering;
mod control_flow;
pub mod debug;
mod expressions;
pub mod function_abi;
pub mod functions;
pub mod output_layout;
pub mod types;

use naga::{valid::ModuleInfo, Module};
use std::collections::HashMap;

/// Configuration for WASM generation
#[derive(Debug, Clone)]
pub struct WasmBackendConfig {
    /// Enable shader stepping via JS stub
    pub debug_shaders: bool,
    /// Optimize generated WASM (future: dead code elimination, constant folding)
    pub optimize: bool,
    /// Target WASM features (SIMD, threads, etc.)
    pub features: WasmFeatures,
}

impl Default for WasmBackendConfig {
    fn default() -> Self {
        Self {
            debug_shaders: true,
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
    /// JS debug stub (optional, for shader stepping)
    pub debug_stub: Option<String>,
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

/// Configuration for compiling a module
pub struct CompileConfig<'a> {
    pub module: &'a Module,
    pub info: &'a ModuleInfo,
    pub source: &'a str,
    pub stage: naga::ShaderStage,
    pub attribute_locations: &'a HashMap<String, u32>,
    pub uniform_locations: &'a HashMap<String, u32>,
    pub varying_locations: &'a HashMap<String, u32>,
    /// Program-level varying type map (name -> (type_code, components)).
    pub varying_types: &'a HashMap<String, (u8, u32)>,
    /// Program-level uniform type map: name -> (type_code, components)
    pub uniform_types: &'a HashMap<String, (u8, u32)>,
    /// Program-level attribute type map: name -> (type_code, components)
    pub attribute_types: &'a HashMap<String, (u8, u32)>,
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
    pub fn compile(
        &self,
        config: CompileConfig,
        name: Option<&str>,
    ) -> Result<WasmModule, BackendError> {
        backend::compile_module(self, config, name)
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

/// Context for translating a single Naga IR function into a WebAssembly function.
///
/// This struct packages together all state and lookup tables required during
/// instruction selection and emission, so that helper routines do not need a
/// large number of separate parameters.
///
pub struct TranslationContext<'a> {
    /// The Naga IR function currently being translated.
    pub func: &'a naga::Function,
    /// The full Naga module that contains the function and its dependencies.
    pub module: &'a naga::Module,
    /// The original source code (for line number mapping)
    pub source: &'a str,
    /// The target WebAssembly function under construction.
    pub wasm_func: &'a mut wasm_encoder::Function,
    /// Mapping from Naga global variables to their (index, size) in the WASM
    /// linear memory or global space, used when emitting loads and stores.
    pub global_offsets: &'a HashMap<naga::Handle<naga::GlobalVariable>, (u32, u32)>,
    /// Mapping from Naga local variables to their corresponding WASM local
    /// indices, used when reading and writing locals.
    pub local_offsets: &'a HashMap<naga::Handle<naga::LocalVariable>, u32>,
    /// Optional mapping from Naga local variable handles to originating global
    /// variable handles (when the local was initialized from a global pointer).
    pub local_origins:
        &'a HashMap<naga::Handle<naga::LocalVariable>, naga::Handle<naga::GlobalVariable>>,
    /// Mapping from Naga expressions that produce values to the WASM local
    /// index where the result is stored, allowing reuse of computed values.
    pub call_result_locals: &'a HashMap<naga::Handle<naga::Expression>, u32>,
    /// Shader stage of the current entry point or function being translated.
    pub stage: naga::ShaderStage,
    /// Debug mode configuration
    pub debug_shaders: bool,
    /// Index of the debug_step host function (if imported)
    pub debug_step_idx: Option<u32>,
    /// Typifier used to query the inferred types of Naga expressions.
    pub typifier: &'a naga::front::Typifier,
    /// Mapping from Naga function handles to their corresponding WASM function
    /// indices, used for emitting call instructions.
    pub naga_function_map: &'a HashMap<naga::Handle<naga::Function>, u32>,
    /// Mapping from Naga function handles to their computed FunctionABI,
    /// Function registry containing pre-calculated ABI and frame manifests
    pub function_registry: &'a functions::FunctionRegistry,
    /// Mapping from argument indices to the WASM local indices that hold the
    /// translated argument values for the current function.
    pub argument_local_offsets: &'a HashMap<u32, u32>,
    /// Mapping from attribute names to their locations.
    pub attribute_locations: &'a HashMap<String, u32>,
    /// Mapping from uniform names to their locations.
    pub uniform_locations: &'a HashMap<String, u32>,
    /// Mapping from varying names to their locations.
    pub varying_locations: &'a HashMap<String, u32>,
    /// Program-level varying type map (name -> (type_code, components)).
    pub varying_types: &'a HashMap<String, (u8, u32)>,
    /// Program-level uniform type map (name -> (type_code, components)).
    pub uniform_types: &'a HashMap<String, (u8, u32)>,
    /// Program-level attribute type map (name -> (type_code, components)).
    pub attribute_types: &'a HashMap<String, (u8, u32)>,
    /// Indicates whether the current function is a shader entry point, which
    /// can affect how inputs, outputs, and builtins are lowered.
    pub is_entry_point: bool,
    /// Explicit swap local for i32 store operations (runtime index after grouping)
    pub swap_i32_local: u32,
    /// Explicit swap local for f32 store operations (runtime index after grouping)
    pub swap_f32_local: u32,
    /// Flattened list of local types (corresponding to WASM local indices starting
    /// at `param_count`). Use this to determine a local's declared type.
    pub local_types: &'a [wasm_encoder::ValType],
    /// Number of function parameters (WASM locals reserved for params start at 0)
    pub param_count: u32,
    /// Index of the emitted module-local helper `__webgl_texture_sample`
    pub webgl_texture_sample_idx: Option<u32>,
    /// Base index for the 4 explicit f32 locals used for texture sampling results
    pub sample_f32_locals: Option<u32>,
    /// Index of a temp I32 local for frame pointer calculations (if allocated)
    pub frame_temp_idx: Option<u32>,
}
