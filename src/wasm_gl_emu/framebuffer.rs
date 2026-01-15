//! Framebuffer management for render targets

/// Framebuffer that owns its data
pub struct OwnedFramebuffer {
    pub width: u32,
    pub height: u32,
    pub internal_format: u32,
    pub color: Vec<u8>,
    pub depth: Vec<f32>,
    pub stencil: Vec<u8>,
}

impl OwnedFramebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self::new_with_format(width, height, 0x8058) // GL_RGBA8
    }

    pub fn new_with_format(width: u32, height: u32, internal_format: u32) -> Self {
        let bytes_per_pixel = match internal_format {
            0x822E => 4,  // GL_R32F
            0x8230 => 8,  // GL_RG32F
            0x8814 => 16, // GL_RGBA32F
            _ => 4,       // GL_RGBA8 default
        };
        Self {
            width,
            height,
            internal_format,
            color: vec![0; (width * height * bytes_per_pixel) as usize],
            depth: vec![1.0; (width * height) as usize],
            stencil: vec![0; (width * height) as usize],
        }
    }

    pub fn as_framebuffer(&mut self) -> Framebuffer<'_> {
        Framebuffer {
            width: self.width,
            height: self.height,
            internal_format: self.internal_format,
            color: &mut self.color,
            depth: &mut self.depth,
            stencil: &mut self.stencil,
        }
    }
}

/// Framebuffer that borrows its data
pub struct Framebuffer<'a> {
    pub width: u32,
    pub height: u32,
    pub internal_format: u32,
    pub color: &'a mut [u8],
    pub depth: &'a mut [f32],
    pub stencil: &'a mut [u8],
}

impl<'a> Framebuffer<'a> {
    pub fn new(
        width: u32,
        height: u32,
        internal_format: u32,
        color: &'a mut [u8],
        depth: &'a mut [f32],
        stencil: &'a mut [u8],
    ) -> Self {
        Self {
            width,
            height,
            internal_format,
            color,
            depth,
            stencil,
        }
    }
}
