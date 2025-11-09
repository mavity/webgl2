//! Code generation logic

use crate::glsl_introspection::ResourceManifest;

/// Generate TypeScript harness code
pub fn generate_harness(_manifest: &ResourceManifest) -> Result<String, CodegenError> {
    // Phase 0: Return minimal template
    Ok(r#"
// Generated TypeScript harness for WebGL2 shader
export class ShaderProgram {
    constructor() {
        // TODO: Initialize shader
    }

    setUniforms(uniforms: any) {
        // TODO: Set uniform values
    }

    draw(attributes: any) {
        // TODO: Execute shader
    }
}
"#
    .to_string())
}

/// Code generation errors
#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Type mapping error: {0}")]
    TypeMappingError(String),
}
