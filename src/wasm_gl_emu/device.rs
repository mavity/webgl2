//! GPU Kernel and Device Management
//!
//! This module provides a centralized "Hardware Abstraction Layer" (HAL)
//! that owns raw pixel storage and handles memory layout logic (Linear, Tiled, etc.)
//! across both WebGL2 and WebGPU frontends.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use wgpu_types as wgt;

/// Unique identifier for a GPU resource (Texture, Renderbuffer, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuHandle(u32);

static NEXT_HANDLE: AtomicU32 = AtomicU32::new(1);

impl GpuHandle {
    pub fn next() -> Self {
        Self(NEXT_HANDLE.fetch_add(1, Ordering::SeqCst))
    }

    pub fn invalid() -> Self {
        Self(0)
    }

    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

/// Memory layout of the GPU buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageLayout {
    /// Standard scanline order: (y * width + x)
    Linear,
    /// Tiled storage (e.g., 8x8 blocks) for cache locality
    Tiled8x8,
    /// Z-order/Morton curve for optimal traversal
    Morton,
}

/// A centralized GPU buffer owned by the GpuKernel
pub struct GpuBuffer {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub format: wgt::TextureFormat,
    pub layout: StorageLayout,
}

impl GpuBuffer {
    pub fn new(width: u32, height: u32, depth: u32, format: wgt::TextureFormat, layout: StorageLayout) -> Self {
        let bpp = format.block_copy_size(None).unwrap_or(4);
        let size = (width as u64) * (height as u64) * (depth as u64) * (bpp as u64);
        Self {
            data: vec![0; size as usize],
            width,
            height,
            depth,
            format,
            layout,
        }
    }

    /// Calculate byte offset for a pixel at (x, y, z)
    pub fn get_pixel_offset(&self, x: u32, y: u32, z: u32) -> usize {
        let bpp = self.format.block_copy_size(None).unwrap_or(4) as usize;
        match self.layout {
            StorageLayout::Linear => {
                ((z * self.height * self.width + y * self.width + x) as usize) * bpp
            }
            StorageLayout::Tiled8x8 => {
                let tile_x = x / 8;
                let tile_y = y / 8;
                let inner_x = x % 8;
                let inner_y = y % 8;
                let tiles_per_row = (self.width + 7) / 8;
                let tile_idx = (tile_y * tiles_per_row + tile_x) as usize;
                let inner_idx = (inner_y * 8 + inner_x) as usize;
                let layer_size = (self.width * self.height * bpp as u32) as usize;
                (z as usize * layer_size) + (tile_idx * 64 + inner_idx) * bpp
            }
            StorageLayout::Morton => {
                // Simplified Morton for 2D per layer
                fn z_order(x: u32, y: u32) -> usize {
                    let mut z = 0;
                    for i in 0..16 {
                        z |= (x & (1 << i)) << i | (y & (1 << i)) << (i + 1);
                    }
                    z as usize
                }
                let layer_size = (self.width * self.height * bpp as u32) as usize;
                (z as usize * layer_size) + z_order(x, y) * bpp
            }
        }
    }
}

/// The centralized GPU Kernel that owns all raw resources
pub struct GpuKernel {
    resources: HashMap<GpuHandle, GpuBuffer>,
}

impl Default for GpuKernel {
    fn default() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}

impl GpuKernel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_buffer(
        &mut self,
        width: u32,
        height: u32,
        depth: u32,
        format: wgt::TextureFormat,
        layout: StorageLayout,
    ) -> GpuHandle {
        let handle = GpuHandle::next();
        let buffer = GpuBuffer::new(width, height, depth, format, layout);
        self.resources.insert(handle, buffer);
        handle
    }

    /// Shortcut for creating a 1D blob buffer (e.g., EBO, VBO)
    pub fn create_buffer_blob(&mut self, size: usize) -> GpuHandle {
        let handle = GpuHandle::next();
        let buffer = GpuBuffer {
            data: vec![0; size],
            width: size as u32,
            height: 1,
            depth: 1,
            format: wgt::TextureFormat::R8Uint,
            layout: StorageLayout::Linear,
        };
        self.resources.insert(handle, buffer);
        handle
    }

    pub fn get_buffer(&self, handle: GpuHandle) -> Option<&GpuBuffer> {
        self.resources.get(&handle)
    }

    pub fn get_buffer_mut(&mut self, handle: GpuHandle) -> Option<&mut GpuBuffer> {
        self.resources.get_mut(&handle)
    }

    pub fn destroy_buffer(&mut self, handle: GpuHandle) {
        self.resources.remove(&handle);
    }

    /// Clear a buffer with a specific color
    pub fn clear(&mut self, handle: GpuHandle, color: [f32; 4]) {
        if let Some(buf) = self.get_buffer_mut(handle) {
            let bpp = buf.format.block_copy_size(None).unwrap_or(4) as usize;
            let mut pixel_bytes = vec![0u8; bpp];

            match buf.format {
                wgt::TextureFormat::Rgba8Unorm => {
                    pixel_bytes[0] = (color[0] * 255.0).round() as u8;
                    pixel_bytes[1] = (color[1] * 255.0).round() as u8;
                    pixel_bytes[2] = (color[2] * 255.0).round() as u8;
                    pixel_bytes[3] = (color[3] * 255.0).round() as u8;
                }
                wgt::TextureFormat::Rgba32Float => {
                    pixel_bytes.copy_from_slice(unsafe {
                        std::slice::from_raw_parts(color.as_ptr() as *const u8, 16)
                    });
                }
                wgt::TextureFormat::R16Uint => {
                    // RGB565 (packed as BGR in wgpu usually, but we define our own here for R16Uint)
                    // Let's use B: 11-15, G: 5-10, R: 0-4
                    let r = (color[0].clamp(0.0, 1.0) * 31.0).round() as u16;
                    let g = (color[1].clamp(0.0, 1.0) * 63.0).round() as u16;
                    let b = (color[2].clamp(0.0, 1.0) * 31.0).round() as u16;
                    let val = (b << 11) | (g << 5) | r;
                    pixel_bytes.copy_from_slice(&val.to_ne_bytes());
                }
                wgt::TextureFormat::Rg8Uint => {
                    // RGBA4
                    let r = (color[0].clamp(0.0, 1.0) * 15.0).round() as u16;
                    let g = (color[1].clamp(0.0, 1.0) * 15.0).round() as u16;
                    let b = (color[2].clamp(0.0, 1.0) * 15.0).round() as u16;
                    let a = (color[3].clamp(0.0, 1.0) * 15.0).round() as u16;
                    let val = (r << 12) | (g << 8) | (b << 4) | a;
                    pixel_bytes.copy_from_slice(&val.to_ne_bytes());
                }
                wgt::TextureFormat::R16Sint => {
                    // RGB5_A1
                    let r = (color[0].clamp(0.0, 1.0) * 31.0).round() as u16;
                    let g = (color[1].clamp(0.0, 1.0) * 31.0).round() as u16;
                    let b = (color[2].clamp(0.0, 1.0) * 31.0).round() as u16;
                    let a = if color[3] > 0.5 { 1 } else { 0 };
                    let val = (r << 11) | (g << 6) | (b << 1) | a;
                    pixel_bytes.copy_from_slice(&val.to_ne_bytes());
                }
                _ => {
                    buf.data.fill(0);
                    return;
                }
            }

            for chunk in buf.data.chunks_exact_mut(bpp) {
                chunk.copy_from_slice(&pixel_bytes);
            }
        }
    }
}
