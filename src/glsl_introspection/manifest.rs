//! Resource manifest generation

use naga::Module;
use serde::{Deserialize, Serialize};

/// Complete resource manifest for a shader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManifest {
    pub uniforms: Vec<UniformInfo>,
    pub attributes: Vec<AttributeInfo>,
    pub varyings: Vec<VaryingInfo>,
    pub textures: Vec<TextureInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniformInfo {
    pub name: String,
    pub glsl_type: String,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeInfo {
    pub name: String,
    pub glsl_type: String,
    pub location: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaryingInfo {
    pub name: String,
    pub glsl_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureInfo {
    pub name: String,
    pub binding: u32,
}

/// Generate a resource manifest from parsed Naga module
pub fn generate_manifest(_module: &Module) -> ResourceManifest {
    // Phase 0: Return empty manifest
    ResourceManifest {
        uniforms: Vec::new(),
        attributes: Vec::new(),
        varyings: Vec::new(),
        textures: Vec::new(),
    }
}
