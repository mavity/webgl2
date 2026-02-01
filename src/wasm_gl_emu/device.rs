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
    pub fn new(
        width: u32,
        height: u32,
        depth: u32,
        format: wgt::TextureFormat,
        layout: StorageLayout,
    ) -> Self {
        let bpp = format.block_copy_size(None).unwrap_or(4) as u64;
        let size = match layout {
            StorageLayout::Linear => (width as u64) * (height as u64) * (depth as u64) * bpp,
            StorageLayout::Tiled8x8 => {
                let tiles_w = width.div_ceil(8);
                let tiles_h = height.div_ceil(8);
                (tiles_w as u64) * (tiles_h as u64) * 64 * (depth as u64) * bpp
            }
            StorageLayout::Morton => {
                // For Morton we ideally want power-of-two dimensions, but for now we just use the next 2^n
                let dim = width.max(height).next_power_of_two();
                (dim as u64) * (dim as u64) * (depth as u64) * bpp
            }
        };
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
        Self::offset_for_layout(
            x,
            y,
            z,
            self.width,
            self.height,
            self.depth,
            self.format,
            self.layout,
        )
    }

    /// Calculate byte offset given layout parameters (useful when buffer is borrowed)
    #[allow(clippy::too_many_arguments)]
    pub fn offset_for_layout(
        x: u32,
        y: u32,
        z: u32,
        width: u32,
        height: u32,
        _depth: u32,
        format: wgt::TextureFormat,
        layout: StorageLayout,
    ) -> usize {
        let bpp = format.block_copy_size(None).unwrap_or(4) as usize;
        match layout {
            StorageLayout::Linear => ((z * height * width + y * width + x) as usize) * bpp,
            StorageLayout::Tiled8x8 => {
                let tiles_w = width.div_ceil(8);
                let tiles_h = height.div_ceil(8);
                let tile_x = x / 8;
                let tile_y = y / 8;
                let inner_x = x % 8;
                let inner_y = y % 8;

                let tile_idx = (tile_y * tiles_w + tile_x) as usize;
                let inner_idx = (inner_y * 8 + inner_x) as usize;
                let layer_size = (tiles_w * tiles_h * 64) as usize * bpp;
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
                let dim = width.max(height).next_power_of_two();
                let layer_size = (dim * dim) as usize * bpp;
                (z as usize * layer_size) + z_order(x, y) * bpp
            }
        }
    }
}

/// The centralized GPU Kernel that owns all raw resources
#[derive(Default)]
pub struct GpuKernel {
    resources: HashMap<GpuHandle, GpuBuffer>,
}

/// Description of a texture binding for shader metadata
#[derive(Debug, Clone)]
pub struct TextureBinding {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub format: u32,
    pub bytes_per_pixel: u32,
    pub wrap_s: u32,
    pub wrap_t: u32,
    pub wrap_r: u32,
    pub min_filter: u32,
    pub mag_filter: u32,
    pub gpu_handle: GpuHandle,
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

    /// Copy a sub-region from one buffer to another
    #[allow(clippy::too_many_arguments)]
    pub fn copy_buffer(
        &mut self,
        src_handle: GpuHandle,
        dst_handle: GpuHandle,
        src_x: u32,
        src_y: u32,
        dst_x: u32,
        dst_y: u32,
        width: u32,
        height: u32,
    ) {
        self.blit(
            src_handle,
            dst_handle,
            src_x as i32,
            src_y as i32,
            (src_x + width) as i32,
            (src_y + height) as i32,
            dst_x as i32,
            dst_y as i32,
            (dst_x + width) as i32,
            (dst_y + height) as i32,
            0x2600, /* GL_NEAREST */
        );
    }

    /// Blit a region from one buffer to another with scaling
    #[allow(clippy::too_many_arguments)]
    pub fn blit(
        &mut self,
        src_handle: GpuHandle,
        dst_handle: GpuHandle,
        src_x0: i32,
        src_y0: i32,
        src_x1: i32,
        src_y1: i32,
        dst_x0: i32,
        dst_y0: i32,
        dst_x1: i32,
        dst_y1: i32,
        _filter: u32, // TODO: support linear?
    ) {
        let (src_data, src_w, src_h, src_d, src_format, src_layout) =
            if let Some(buf) = self.get_buffer(src_handle) {
                (
                    buf.data.clone(),
                    buf.width,
                    buf.height,
                    buf.depth,
                    buf.format,
                    buf.layout,
                )
            } else {
                return;
            };

        if let Some(dst_buf) = self.get_buffer_mut(dst_handle) {
            let src_bpp = src_format.block_copy_size(None).unwrap_or(4) as usize;
            let dst_bpp = dst_buf.format.block_copy_size(None).unwrap_or(4) as usize;
            let bpp = src_bpp.min(dst_bpp);

            let dst_w_region = (dst_x1 - dst_x0).abs();
            let dst_h_region = (dst_y1 - dst_y0).abs();
            let src_w_region = (src_x1 - src_x0).abs();
            let src_h_region = (src_y1 - src_y0).abs();

            if dst_w_region == 0 || dst_h_region == 0 {
                return;
            }

            let x_step = src_w_region as f32 / dst_w_region as f32;
            let y_step = src_h_region as f32 / dst_h_region as f32;

            let x_dir = if dst_x1 > dst_x0 { 1 } else { -1 };
            let y_dir = if dst_y1 > dst_y0 { 1 } else { -1 };
            let sx_dir = if src_x1 > src_x0 { 1 } else { -1 };
            let sy_dir = if src_y1 > src_y0 { 1 } else { -1 };

            for dy_rel in 0..dst_h_region {
                for dx_rel in 0..dst_w_region {
                    let dx = dst_x0 + dx_rel * x_dir;
                    let dy = dst_y0 + dy_rel * y_dir;

                    if dx < 0 || dx >= dst_buf.width as i32 || dy < 0 || dy >= dst_buf.height as i32
                    {
                        continue;
                    }

                    let sx = src_x0 + ((dx_rel as f32 * x_step) as i32) * sx_dir;
                    let sy = src_y0 + ((dy_rel as f32 * y_step) as i32) * sy_dir;

                    if sx < 0 || sx >= src_w as i32 || sy < 0 || sy >= src_h as i32 {
                        continue;
                    }

                    let src_off = GpuBuffer::offset_for_layout(
                        sx as u32, sy as u32, 0, src_w, src_h, src_d, src_format, src_layout,
                    );
                    let dst_off = dst_buf.get_pixel_offset(dx as u32, dy as u32, 0);

                    if src_off + bpp <= src_data.len() && dst_off + bpp <= dst_buf.data.len() {
                        dst_buf.data[dst_off..dst_off + bpp]
                            .copy_from_slice(&src_data[src_off..src_off + bpp]);
                    }
                }
            }
        }
    }

    /// Clear a buffer with a specific color
    pub fn clear(&mut self, handle: GpuHandle, color: [f32; 4]) {
        if let Some(buf) = self.get_buffer(handle) {
            let width = buf.width;
            let height = buf.height;
            self.clear_rect(handle, color, 0, 0, width, height);
        }
    }

    /// Clear a sub-region of a buffer with raw bytes
    pub fn clear_rect_raw(
        &mut self,
        handle: GpuHandle,
        pixel_bytes: &[u8],
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) {
        if let Some(buf) = self.get_buffer_mut(handle) {
            let bpp = buf.format.block_copy_size(None).unwrap_or(4) as usize;
            let bytes_to_copy = &pixel_bytes[..bpp.min(pixel_bytes.len())];

            for row in 0..height {
                for col in 0..width {
                    let dx = x + col as i32;
                    let dy = y + row as i32;
                    if dx >= 0 && dx < buf.width as i32 && dy >= 0 && dy < buf.height as i32 {
                        let off = buf.get_pixel_offset(dx as u32, dy as u32, 0);
                        if off + bpp <= buf.data.len() {
                            buf.data[off..off + bpp].copy_from_slice(bytes_to_copy);
                        }
                    }
                }
            }
        }
    }

    /// Clear a sub-region of a buffer
    pub fn clear_rect(
        &mut self,
        handle: GpuHandle,
        color: [f32; 4],
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) {
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
                wgt::TextureFormat::R32Float => {
                    pixel_bytes.copy_from_slice(&color[0].to_ne_bytes());
                }
                wgt::TextureFormat::Rg32Float => {
                    pixel_bytes[0..4].copy_from_slice(&color[0].to_ne_bytes());
                    pixel_bytes[4..8].copy_from_slice(&color[1].to_ne_bytes());
                }
                wgt::TextureFormat::Rgba32Uint | wgt::TextureFormat::Rgba32Sint => {
                    // Lossy clear for integer formats from float color
                    for i in 0..4 {
                        let c = (color[i] as u32).to_ne_bytes();
                        pixel_bytes[i * 4..(i + 1) * 4].copy_from_slice(&c);
                    }
                }
                wgt::TextureFormat::R32Uint | wgt::TextureFormat::R32Sint => {
                    pixel_bytes.copy_from_slice(&(color[0] as u32).to_ne_bytes());
                }
                wgt::TextureFormat::Rg32Uint | wgt::TextureFormat::Rg32Sint => {
                    pixel_bytes[0..4].copy_from_slice(&(color[0] as u32).to_ne_bytes());
                    pixel_bytes[4..8].copy_from_slice(&(color[1] as u32).to_ne_bytes());
                }
                wgt::TextureFormat::R8Uint | wgt::TextureFormat::R8Sint => {
                    pixel_bytes[0] = color[0] as u8;
                }
                wgt::TextureFormat::Rg8Sint => {
                    pixel_bytes[0] = color[0] as u8;
                    pixel_bytes[1] = color[1] as u8;
                }
                wgt::TextureFormat::Rgba8Uint | wgt::TextureFormat::Rgba8Sint => {
                    pixel_bytes[0] = color[0] as u8;
                    pixel_bytes[1] = color[1] as u8;
                    pixel_bytes[2] = color[2] as u8;
                    pixel_bytes[3] = color[3] as u8;
                }
                wgt::TextureFormat::R16Uint => {
                    // Placeholder for RGB565 logic
                    let r = (color[0].clamp(0.0, 1.0) * 31.0).round() as u16;
                    let g = (color[1].clamp(0.0, 1.0) * 63.0).round() as u16;
                    let b = (color[2].clamp(0.0, 1.0) * 31.0).round() as u16;
                    let val = (b << 11) | (g << 5) | r;
                    pixel_bytes.copy_from_slice(&val.to_ne_bytes());
                }
                wgt::TextureFormat::Rg8Uint => {
                    // Placeholder for RGBA4 logic
                    let r = (color[0].clamp(0.0, 1.0) * 15.0).round() as u16;
                    let g = (color[1].clamp(0.0, 1.0) * 15.0).round() as u16;
                    let b = (color[2].clamp(0.0, 1.0) * 15.0).round() as u16;
                    let a = (color[3].clamp(0.0, 1.0) * 15.0).round() as u16;
                    let val = (r << 12) | (g << 8) | (b << 4) | a;
                    pixel_bytes.copy_from_slice(&val.to_ne_bytes());
                }
                wgt::TextureFormat::R16Sint => {
                    // Placeholder for RGB5A1 logic
                    let r = (color[0].clamp(0.0, 1.0) * 31.0).round() as u16;
                    let g = (color[1].clamp(0.0, 1.0) * 31.0).round() as u16;
                    let b = (color[2].clamp(0.0, 1.0) * 31.0).round() as u16;
                    let a = if color[3] > 0.5 { 1 } else { 0 };
                    let val = (r << 11) | (g << 6) | (b << 1) | a;
                    pixel_bytes.copy_from_slice(&val.to_ne_bytes());
                }
                _ => {
                    // Fallback to zeroing if format not explicitly handled for clear color
                    pixel_bytes.fill(0);
                }
            }

            self.clear_rect_raw(handle, &pixel_bytes, x, y, width, height);
        }
    }

    /// Writes texture metadata to the specified linear memory pointer for shader access
    pub fn write_texture_metadata(&self, bindings: &[Option<TextureBinding>], dest_ptr: u32) {
        for (i, binding) in bindings.iter().enumerate() {
            let offset = i * 64; // Match Naga stride (aligned to 64 bytes)
            if let Some(b) = binding {
                if let Some(buf) = self.get_buffer(b.gpu_handle) {
                    unsafe {
                        let base = (dest_ptr + offset as u32) as *mut i32;
                        *base.offset(0) = b.width as i32;
                        *base.offset(1) = b.height as i32;
                        *base.offset(2) = buf.data.as_ptr() as i32;
                        *base.offset(3) = b.depth as i32;
                        *base.offset(4) = b.format as i32;
                        *base.offset(5) = b.bytes_per_pixel as i32;
                        *base.offset(6) = b.wrap_s as i32;
                        *base.offset(7) = b.wrap_t as i32;
                        *base.offset(8) = b.wrap_r as i32;
                        *base.offset(9) = buf.layout as i32;
                        *base.offset(10) = b.min_filter as i32;
                        *base.offset(11) = b.mag_filter as i32;
                    }
                }
            }
        }
    }

    /// Copy a 1D range between two buffers (blobs)
    pub fn copy_blob(
        &mut self,
        src_handle: GpuHandle,
        dst_handle: GpuHandle,
        src_off: usize,
        dst_off: usize,
        size: usize,
    ) {
        if src_handle == dst_handle {
            return;
        }

        let data_to_copy = if let Some(src_buf) = self.resources.get(&src_handle) {
            if src_off + size <= src_buf.data.len() {
                src_buf.data[src_off..src_off + size].to_vec()
            } else {
                return;
            }
        } else {
            return;
        };

        if let Some(dst_buf) = self.resources.get_mut(&dst_handle) {
            if dst_off + size <= dst_buf.data.len() {
                dst_buf.data[dst_off..dst_off + size].copy_from_slice(&data_to_copy);
            }
        }
    }
}
