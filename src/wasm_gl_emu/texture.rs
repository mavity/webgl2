//! Texture storage and sampling
//!
//! Phase 0: Basic texture structure

/// 2D texture with RGBA data
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGBA8 format
}

impl Texture {
    /// Create a new texture with the given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            width,
            height,
            data: vec![0; size],
        }
    }

    /// Sample texture with bilinear filtering (future implementation)
    pub fn sample_bilinear(&self, _u: f32, _v: f32) -> [f32; 4] {
        // Phase 0: Return black
        [0.0, 0.0, 0.0, 1.0]
    }

    /// Get pixel at coordinates (clamped)
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);
        let offset = ((y * self.width + x) * 4) as usize;
        [
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
        ]
    }

    /// Set pixel at coordinates
    pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x < self.width && y < self.height {
            let offset = ((y * self.width + x) * 4) as usize;
            self.data[offset..offset + 4].copy_from_slice(&color);
        }
    }
}
