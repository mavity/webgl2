//! GLSL Annotation Parser and Resource Manifest Generator
//!
//! This module parses GLSL source code with custom annotations (like @uniform_group)
//! and generates resource manifests for the JavaScript/TypeScript harness.

mod annotations;
mod manifest;
mod parser;

pub use annotations::{Annotation, BufferLayout, UniformGroup};
pub use manifest::{AttributeInfo, ResourceManifest, UniformInfo};
pub use parser::{parse_glsl, ParseError};

/// Parse GLSL with annotations and generate a resource manifest
pub fn introspect_shader(source: &str) -> Result<ResourceManifest, ParseError> {
    let parsed = parser::parse_glsl(source)?;
    Ok(manifest::generate_manifest(&parsed))
}
