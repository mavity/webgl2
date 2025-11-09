//! Framebuffer management for render targets

/// Framebuffer with color and depth buffers
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub color: Vec<u8>,
    pub depth: Vec<f32>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            color: vec![0; (width * height * 4) as usize],
            depth: vec![1.0; (width * height) as usize],
        }
    }
}
