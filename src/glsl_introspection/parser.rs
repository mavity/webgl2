//! GLSL parsing with Naga

use naga::{valid::Validator, Module};

/// Parse GLSL source code into Naga IR
pub fn parse_glsl(source: &str) -> Result<Module, ParseError> {
    let options = naga::front::glsl::Options::from(naga::ShaderStage::Vertex);

    let module = naga::front::glsl::Frontend::default()
        .parse(&options, source)
        .map_err(|errors| ParseError::GlslParseError(format!("{:?}", errors)))?;

    // Validate the module
    let mut validator = Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );

    validator
        .validate(&module)
        .map_err(|e| ParseError::ValidationError(e.to_string()))?;

    Ok(module)
}

/// Parsing errors
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("GLSL parsing error: {0}")]
    GlslParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Annotation parsing error: {0}")]
    AnnotationError(String),
}
