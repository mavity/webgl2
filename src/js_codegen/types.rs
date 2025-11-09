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

    /// Convert to TypeScript type string
    pub fn to_string(&self) -> String {
        match self {
            Self::Number => "number".to_string(),
            Self::Boolean => "boolean".to_string(),
            Self::String => "string".to_string(),
            Self::Array(inner) => format!("{}[]", inner.to_string()),
            Self::Object => "object".to_string(),
            Self::Void => "void".to_string(),
        }
    }
}
