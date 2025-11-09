//! Custom annotation types for GLSL

use serde::{Deserialize, Serialize};

/// Custom annotations in GLSL comments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Annotation {
    /// @uniform_group(N)
    UniformGroup(UniformGroup),
    /// @buffer_layout(std140)
    BufferLayout(BufferLayout),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniformGroup {
    pub group: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferLayout {
    pub layout: String, // "std140", "std430", etc.
}
