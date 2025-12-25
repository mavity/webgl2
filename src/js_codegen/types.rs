//! TypeScript type system

/// TypeScript type representations
#[derive(Debug, Clone)]
pub enum TypeScriptType {
    Number,
    Boolean,
    String,
    Array(Box<TypeScriptType>),
    Object,
    Void,
}

impl TypeScriptType {
    /// Convert a GLSL type string to TypeScript
    pub fn from_glsl(glsl_type: &str) -> Self {
        match glsl_type {
            "float" | "int" | "uint" => Self::Number,
            "bool" => Self::Boolean,
            "vec2" | "vec3" | "vec4" => Self::Array(Box::new(Self::Number)),
            "mat2" | "mat3" | "mat4" => Self::Array(Box::new(Self::Number)),
            _ => Self::Object,
        }
    }
}

impl std::fmt::Display for TypeScriptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number => write!(f, "number"),
            Self::Boolean => write!(f, "boolean"),
            Self::String => write!(f, "string"),
            Self::Array(inner) => write!(f, "{}[]", inner),
            Self::Object => write!(f, "object"),
            Self::Void => write!(f, "void"),
        }
    }
}
