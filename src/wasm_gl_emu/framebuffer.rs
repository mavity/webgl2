//! Framebuffer management for render targets

/// Framebuffer that owns its data
pub struct OwnedFramebuffer {
    pub width: u32,
    pub height: u32,
    pub color: Vec<u8>,
    pub depth: Vec<f32>,
}

impl OwnedFramebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            color: vec![0; (width * height * 4) as usize],
            depth: vec![1.0; (width * height) as usize],
        }
    }

    pub fn as_framebuffer(&mut self) -> Framebuffer {
        Framebuffer {
            width: self.width,
            height: self.height,
            color: &mut self.color,
            depth: &mut self.depth,
        }
    }
}

/// Framebuffer that borrows its data
pub struct Framebuffer<'a> {
    pub width: u32,
    pub height: u32,
    pub color: &'a mut [u8],
    pub depth: &'a mut [f32],
}

impl<'a> Framebuffer<'a> {
    pub fn new(width: u32, height: u32, color: &'a mut [u8], depth: &'a mut [f32]) -> Self {
        Self {
            width,
            height,
            color,
            depth,
        }
    }
}
