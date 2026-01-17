//! Framebuffer management for render targets
use crate::wasm_gl_emu::device::{GpuHandle, GpuKernel, StorageLayout};
use wgpu_types as wgt;

/// Framebuffer that owns its data
pub struct OwnedFramebuffer {
    pub width: u32,
    pub height: u32,
    pub internal_format: u32,
    pub gpu_handle: GpuHandle,
    pub depth: Vec<f32>,
    pub stencil: Vec<u8>,
}

impl OwnedFramebuffer {
    pub fn new(kernel: &mut GpuKernel, width: u32, height: u32) -> Self {
        Self::new_with_format(kernel, width, height, 0x8058) // GL_RGBA8
    }

    pub fn new_with_format(kernel: &mut GpuKernel, width: u32, height: u32, internal_format: u32) -> Self {
        let format = match internal_format {
            0x822E => wgt::TextureFormat::R32Float,  // GL_R32F
            0x8230 => wgt::TextureFormat::Rg32Float, // GL_RG32F
            0x8814 => wgt::TextureFormat::Rgba32Float, // GL_RGBA32F
            _ => wgt::TextureFormat::Rgba8Unorm,     // GL_RGBA8
        };
        
        let gpu_handle = kernel.create_buffer(width, height, 1, format, StorageLayout::Linear);
        
        Self {
            width,
            height,
            internal_format,
            gpu_handle,
            depth: vec![1.0; (width * height) as usize],
            stencil: vec![0; (width * height) as usize],
        }
    }

    pub fn as_framebuffer<'a>(&'a mut self, kernel: &'a mut GpuKernel) -> Framebuffer<'a> {
        let buffer = kernel.get_buffer_mut(self.gpu_handle).expect("buffer lost");
        Framebuffer {
            width: self.width,
            height: self.height,
            internal_format: self.internal_format,
            color: &mut buffer.data,
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
