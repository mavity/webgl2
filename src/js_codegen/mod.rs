//! TypeScript/JavaScript Harness Code Generator
//!
//! This module generates TypeScript wrapper code that makes it easy to use
//! compiled WASM shaders from JavaScript applications.

mod generator;
mod types;

pub use generator::{generate_harness, CodegenError};
pub use types::TypeScriptType;

use crate::glsl_introspection::ResourceManifest;

/// Generate TypeScript harness code from a resource manifest
pub fn generate_typescript(manifest: &ResourceManifest) -> Result<String, CodegenError> {
    generator::generate_harness(manifest)
}
