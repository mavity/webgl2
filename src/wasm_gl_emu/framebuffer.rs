//! Framebuffer management for render targets
use crate::wasm_gl_emu::device::{GpuBuffer, GpuHandle, GpuKernel, StorageLayout};
use wgpu_types as wgt;

/// Framebuffer that owns its data
pub struct OwnedFramebuffer {
    pub width: u32,
    pub height: u32,
    pub internal_format: u32,
    pub gpu_handle: GpuHandle,
    pub depth: Vec<f32>,
    pub stencil: Vec<u8>,
    pub layout: StorageLayout,
}

impl OwnedFramebuffer {
    pub fn new(kernel: &mut GpuKernel, width: u32, height: u32) -> Self {
        Self::new_with_format(kernel, width, height, 0x8058) // GL_RGBA8
    }

    pub fn new_with_format(
        kernel: &mut GpuKernel,
        width: u32,
        height: u32,
        internal_format: u32,
    ) -> Self {
        let format = match internal_format {
            0x822E => wgt::TextureFormat::R32Float,    // GL_R32F
            0x8230 => wgt::TextureFormat::Rg32Float,   // GL_RG32F
            0x8814 => wgt::TextureFormat::Rgba32Float, // GL_RGBA32F
            _ => wgt::TextureFormat::Rgba8Unorm,       // GL_RGBA8
        };

        let layout = StorageLayout::Linear;
        let gpu_handle = kernel.create_buffer(width, height, 1, format, layout);

        let pixel_count = match layout {
            StorageLayout::Linear => (width * height) as usize,
            StorageLayout::Tiled8x8 => {
                let tiles_w = width.div_ceil(8);
                let tiles_h = height.div_ceil(8);
                (tiles_w * tiles_h * 64) as usize
            }
            StorageLayout::Morton => {
                let dim = width.max(height).next_power_of_two();
                (dim * dim) as usize
            }
        };

        Self {
            width,
            height,
            internal_format,
            gpu_handle,
            depth: vec![1.0; pixel_count],
            stencil: vec![0; pixel_count],
            layout,
        }
    }

    pub fn as_framebuffer<'a>(&'a mut self, kernel: &'a mut GpuKernel) -> Framebuffer<'a> {
        let buffer = kernel.get_buffer_mut(self.gpu_handle).expect("buffer lost");
        Framebuffer {
            width: buffer.width,
            height: buffer.height,
            color_attachments: vec![Some(ColorAttachment {
                data: &mut buffer.data,
                internal_format: self.internal_format,
            })],
            depth: &mut self.depth,
            stencil: &mut self.stencil,
            layout: self.layout,
        }
    }

    pub fn clear_depth(&mut self, depth: f32, mask: bool) {
        if mask {
            self.depth.fill(depth);
        }
    }

    pub fn clear_stencil(&mut self, value: u8, write_mask: u8) {
        if write_mask == 0xFF {
            self.stencil.fill(value);
        } else {
            for s in self.stencil.iter_mut() {
                *s = (*s & !write_mask) | (value & write_mask);
            }
        }
    }
}

/// Framebuffer that borrows its data
pub struct ColorAttachment<'a> {
    pub data: &'a mut [u8],
    pub internal_format: u32,
}

pub struct Framebuffer<'a> {
    pub width: u32,
    pub height: u32,
    pub color_attachments: Vec<Option<ColorAttachment<'a>>>,
    pub depth: &'a mut [f32],
    pub stencil: &'a mut [u8],
    pub layout: StorageLayout,
}

impl<'a> Framebuffer<'a> {
    pub fn new(
        width: u32,
        height: u32,
        color_attachments: Vec<Option<ColorAttachment<'a>>>,
        depth: &'a mut [f32],
        stencil: &'a mut [u8],
        layout: StorageLayout,
    ) -> Self {
        Self {
            width,
            height,
            color_attachments,
            depth,
            stencil,
            layout,
        }
    }

    pub fn get_pixel_offset_params(
        x: u32,
        y: u32,
        z: u32,
        internal_format: u32,
        width: u32,
        height: u32,
        layout: StorageLayout,
    ) -> usize {
        let format = match internal_format {
            0x822E => wgt::TextureFormat::R32Float,
            0x8230 => wgt::TextureFormat::Rg32Float,
            0x8814 => wgt::TextureFormat::Rgba32Float,
            _ => wgt::TextureFormat::Rgba8Unorm,
        };
        GpuBuffer::offset_for_layout(x, y, z, width, height, 1, format, layout)
    }

    pub fn get_pixel_offset(&self, x: u32, y: u32, z: u32, internal_format: u32) -> usize {
        Self::get_pixel_offset_params(
            x,
            y,
            z,
            internal_format,
            self.width,
            self.height,
            self.layout,
        )
    }

    pub fn get_pixel_index(&self, x: u32, y: u32, z: u32) -> usize {
        // Use R8Unorm to get a 1-byte bpp offset (effectively pixel index)
        GpuBuffer::offset_for_layout(
            x,
            y,
            z,
            self.width,
            self.height,
            1,
            wgt::TextureFormat::R8Unorm,
            self.layout,
        )
    }
}
