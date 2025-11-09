//! Web runtime for WASM targets (future implementation)

use super::RuntimeError;

/// Instantiate shader in web context (future)
pub fn instantiate_shader() -> Result<(), RuntimeError> {
    Err(RuntimeError::InitError(
        "Web runtime not yet implemented".to_string(),
    ))
}
