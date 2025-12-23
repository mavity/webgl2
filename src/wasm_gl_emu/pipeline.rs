//! Pipeline coordinate for vertex and fragment shading

/// Vertex output data
#[derive(Debug, Clone)]
pub struct VertexOutput {
    pub position: [f32; 4],
    pub varyings: Vec<f32>,
}

/// Rendering pipeline
#[derive(Default)]
pub struct Pipeline {
    // Phase 0: Placeholder
}
