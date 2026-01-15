//! Texture storage and sampling
//!
//! Phase 0: Basic texture structure

/// 2D texture with format-aware storage
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub internal_format: u32,
    pub data: Vec<u8>,
}

impl Texture {
    /// Create a new texture with the given dimensions and format
    pub fn new(width: u32, height: u32) -> Self {
        Self::new_with_format(width, height, 0x8058) // GL_RGBA8 default
    }

    pub fn new_with_format(width: u32, height: u32, internal_format: u32) -> Self {
        let bytes_per_pixel = match internal_format {
            0x822E => 4,  // GL_R32F: 1 channel × 4 bytes
            0x8230 => 8,  // GL_RG32F: 2 channels × 4 bytes
            0x8814 => 16, // GL_RGBA32F: 4 channels × 4 bytes
            _ => 4,       // GL_RGBA8: 4 channels × 1 byte
        };
        let size = (width * height * bytes_per_pixel) as usize;
        Self {
            width,
            height,
            internal_format,
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
        let bytes_per_pixel = match self.internal_format {
            0x822E => 4,  // GL_R32F
            0x8230 => 8,  // GL_RG32F
            0x8814 => 16, // GL_RGBA32F
            _ => 4,       // GL_RGBA8
        };
        let offset = ((y * self.width + x) * bytes_per_pixel) as usize;
        [
            self.data[offset],
            self.data.get(offset + 1).copied().unwrap_or(0),
            self.data.get(offset + 2).copied().unwrap_or(0),
            self.data.get(offset + 3).copied().unwrap_or(255),
        ]
    }

    /// Set pixel at coordinates
    pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x < self.width && y < self.height {
            let bytes_per_pixel = match self.internal_format {
                0x822E => 4,  // GL_R32F
                0x8230 => 8,  // GL_RG32F
                0x8814 => 16, // GL_RGBA32F
                _ => 4,       // GL_RGBA8
            };
            let offset = ((y * self.width + x) * bytes_per_pixel) as usize;
            for (i, &byte) in color.iter().enumerate() {
                if offset + i < self.data.len() {
                    self.data[offset + i] = byte;
                }
            }
        }
    }
}
