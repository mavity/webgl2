//! Expression translation from Naga IR to WASM
//!
//! Phase 0: Placeholder for future expression handling

use super::BackendError;

/// Translate a Naga expression to WASM instructions (future implementation)
pub fn translate_expression(_expr: &naga::Expression) -> Result<Vec<u8>, BackendError> {
    // Phase 0: Not implemented yet
    Err(BackendError::UnsupportedFeature(
        "Expression translation not yet implemented".to_string(),
    ))
}
