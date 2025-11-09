//! WASM shader runtime using Wasmtime

#[cfg(feature = "cli")]
use wasmtime::*;
#[cfg(feature = "cli")]
use crate::naga_wasm_backend::WasmModule;

#[cfg(feature = "cli")]
/// Runtime for executing WASM shaders
pub struct ShaderRuntime {
    engine: Engine,
    module: Module,
    store: Store<RuntimeState>,
}

#[cfg(feature = "cli")]
/// Runtime state accessible by WASM shaders
pub struct RuntimeState {
    pub memory: Vec<u8>,
    pub textures: Vec<super::Texture>,
}

#[cfg(feature = "cli")]
impl RuntimeState {
    fn new() -> Self {
        // Allocate 1MB of memory (16 pages)
        Self {
            memory: vec![0; 1024 * 1024],
            textures: Vec::new(),
        }
    }
}

/// Runtime errors (always available for consistency)
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("WASM initialization error: {0}")]
    InitError(String),

    #[error("WASM execution error: {0}")]
    ExecutionError(String),

    #[error("Memory access error: {0}")]
    MemoryError(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),
}

impl From<anyhow::Error> for RuntimeError {
    fn from(err: anyhow::Error) -> Self {
        RuntimeError::InitError(err.to_string())
    }
}

#[cfg(feature = "cli")]
impl ShaderRuntime {
    /// Create a new shader runtime from compiled WASM
    pub fn new(wasm_module: &WasmModule) -> Result<Self, RuntimeError> {
        tracing::debug!("Creating shader runtime from {} bytes of WASM", wasm_module.wasm_bytes.len());

        let engine = Engine::default();
        let module = Module::new(&engine, &wasm_module.wasm_bytes)?;
        let store = Store::new(&engine, RuntimeState::new());

        Ok(Self {
            engine,
            module,
            store,
        })
    }

    /// Execute a vertex shader for a single vertex
    ///
    /// # Arguments
    ///
    /// * `entry_point` - Name of the entry point function (e.g., "main")
    /// * `attributes` - Vertex attribute data
    ///
    /// # Returns
    ///
    /// Position and varying data (x, y, z, w)
    pub fn run_vertex_shader(
        &mut self,
        entry_point: &str,
        _attributes: &[f32],
    ) -> Result<(f32, f32, f32, f32), RuntimeError> {
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut self.store, &self.module)?;

        // Get the entry point function
        let func = instance
            .get_typed_func::<(i32, i32, i32), (f32, f32, f32, f32)>(&mut self.store, entry_point)
            .map_err(|_| RuntimeError::FunctionNotFound(entry_point.to_string()))?;

        // Phase 0: Just call with dummy pointers
        let result = func.call(&mut self.store, (0x2000, 0x1000, 0x3000))
            .map_err(|e| RuntimeError::ExecutionError(e.to_string()))?;

        tracing::debug!("Vertex shader result: {:?}", result);
        Ok(result)
    }

    /// Execute a fragment shader for a single pixel
    ///
    /// # Arguments
    ///
    /// * `entry_point` - Name of the entry point function
    /// * `varyings` - Interpolated varying data
    ///
    /// # Returns
    ///
    /// RGBA color output
    pub fn run_fragment_shader(
        &mut self,
        entry_point: &str,
        _varyings: &[f32],
    ) -> Result<(f32, f32, f32, f32), RuntimeError> {
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut self.store, &self.module)?;

        let func = instance
            .get_typed_func::<(i32, i32), (f32, f32, f32, f32)>(&mut self.store, entry_point)
            .map_err(|_| RuntimeError::FunctionNotFound(entry_point.to_string()))?;

        let result = func.call(&mut self.store, (0x3000, 0x1000))
            .map_err(|e| RuntimeError::ExecutionError(e.to_string()))?;

        tracing::debug!("Fragment shader result: {:?}", result);
        Ok(result)
    }
}
