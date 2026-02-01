//! Data Transfer and Format Conversion Engine
//!
//! Handles movement and interpretation of raw bytes between
//! GPU buffers and shader-accessible formats (Vertex, Index, etc.)

use crate::wasm_gl_emu::device::{GpuBuffer, StorageLayout};
use wgpu_types as wgt;

pub struct TransferRequest<'a> {
    pub src_buffer: &'a GpuBuffer,
    pub dst_format: wgt::TextureFormat,
    pub dst_layout: StorageLayout,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    U8,
    U16,
    U32,
}

/// A lazy index buffer fetcher that avoids intermediate allocations
pub struct LazyIndexBuffer {
    pub src_ptr: *const u8,
    pub src_len: usize,
    pub index_type: IndexType,
    pub offset: u32,
    pub count: u32,
}

impl crate::wasm_gl_emu::rasterizer::IndexBuffer for LazyIndexBuffer {
    fn len(&self) -> usize {
        self.count as usize
    }

    fn get(&self, i: usize) -> u32 {
        let data = unsafe { std::slice::from_raw_parts(self.src_ptr, self.src_len) };
        match self.index_type {
            IndexType::U8 => {
                let off = (self.offset as usize) + i;
                data.get(off).copied().map(|v| v as u32).unwrap_or(0)
            }
            IndexType::U16 => {
                let off = (self.offset as usize) + i * 2;
                if off + 2 <= data.len() {
                    u16::from_ne_bytes([data[off], data[off + 1]]) as u32
                } else {
                    0
                }
            }
            IndexType::U32 => {
                let off = (self.offset as usize) + i * 4;
                if off + 4 <= data.len() {
                    u32::from_ne_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
                } else {
                    0
                }
            }
        }
    }
}

/// Description of a vertex attribute binding for backend fetching
#[derive(Debug, Clone, Copy)]
pub struct AttributeBinding {
    /// Pointer to the start of the buffer data
    pub buffer_ptr: *const u8,
    /// Offset in bytes within the buffer
    pub offset: usize,
    /// Stride in bytes between consecutive vertices
    pub stride: usize,
    /// Number of components (1, 2, 3, or 4)
    pub size: i32,
    /// GL type (GL_FLOAT, GL_UNSIGNED_BYTE, etc)
    pub type_: u32,
    /// Whether the data should be normalized
    pub normalized: bool,
    /// Whether the attribute is an integer type (vertexAttribIPointer)
    pub is_integer: bool,
    /// Size of a single component in bytes
    pub type_size: usize,
    /// Divisor for instanced rendering (0 = per-vertex, >0 = per-instance)
    pub divisor: u32,
    /// Default value for the attribute if the buffer is missing or disabled
    pub default_value: [u32; 4],
}

pub struct TransferEngine;

impl TransferEngine {
    /// Fetch indices from a GPU buffer and convert to u32
    pub fn fetch_indices(
        src: &GpuBuffer,
        index_type: IndexType,
        offset: u32,
        count: u32,
    ) -> Vec<u32> {
        let mut result = Vec::with_capacity(count as usize);
        let data = &src.data;

        for i in 0..count {
            let idx = match index_type {
                IndexType::U8 => {
                    let off = (offset as usize) + i as usize;
                    data.get(off).copied().map(|v| v as u32).unwrap_or(0)
                }
                IndexType::U16 => {
                    let off = (offset as usize) + (i as usize) * 2;
                    if off + 2 <= data.len() {
                        u16::from_ne_bytes([data[off], data[off + 1]]) as u32
                    } else {
                        0
                    }
                }
                IndexType::U32 => {
                    let off = (offset as usize) + (i as usize) * 4;
                    if off + 4 <= data.len() {
                        u32::from_ne_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
                    } else {
                        0
                    }
                }
            };
            result.push(idx);
        }
        result
    }

    /// Fetch a batch of vertex attributes for a specific vertex and instance
    pub fn fetch_vertex_batch(
        bindings: &[AttributeBinding],
        vertex_index: u32,
        instance_index: u32,
        dest: &mut [u32], // Destination array (usually 16 slots * 4 components)
    ) {
        for (loc, attr) in bindings.iter().enumerate() {
            let base_idx = loc * 4;
            if base_idx + 4 > dest.len() {
                break;
            }

            let effective_index = if attr.divisor == 0 {
                vertex_index as usize
            } else {
                (instance_index / attr.divisor) as usize
            };

            let mut comp_dest = [0u32; 4];
            Self::fetch_vertex_attribute(attr, effective_index, &mut comp_dest);

            dest[base_idx..base_idx + 4].copy_from_slice(&comp_dest);
        }
    }

    /// Fetch a single vertex attribute component set from a buffer using raw GL parameters
    pub fn fetch_vertex_attribute(
        binding: &AttributeBinding,
        vertex_index: usize,
        dest: &mut [u32; 4],
    ) {
        // Zero out dest initially
        dest.fill(0);
        // Default W component is 1.0 (float) or 1 (int)
        if binding.is_integer {
            dest[3] = 1;
        } else {
            dest[3] = 1.0f32.to_bits();
        }

        if binding.buffer_ptr.is_null() {
            dest.copy_from_slice(&binding.default_value);
            return;
        }

        let base_offset = binding.offset + vertex_index * binding.stride;

        #[allow(clippy::needless_range_loop)]
        for i in 0..(binding.size as usize).min(4) {
            let component_offset = base_offset + i * binding.type_size;

            unsafe {
                let ptr = binding.buffer_ptr.add(component_offset);

                match binding.type_ {
                    0x1406 /* FLOAT */ => {
                        let mut bytes = [0u8; 4];
                        std::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr(), 4);
                        dest[i] = u32::from_le_bytes(bytes);
                    }
                    0x1401 /* UNSIGNED_BYTE */ => {
                        let val = *ptr;
                        if binding.is_integer {
                            dest[i] = val as u32;
                        } else if binding.normalized {
                            dest[i] = (val as f32 / 255.0).to_bits();
                        } else {
                            dest[i] = (val as f32).to_bits();
                        }
                    }
                    0x1400 /* BYTE */ => {
                        let val = *ptr as i8;
                        if binding.is_integer {
                            dest[i] = val as i32 as u32;
                        } else if binding.normalized {
                            dest[i] = (val as f32 / 127.0).clamp(-1.0, 1.0).to_bits();
                        } else {
                            dest[i] = (val as f32).to_bits();
                        }
                    }
                    0x1403 /* UNSIGNED_SHORT */ => {
                        let mut bytes = [0u8; 2];
                        std::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr(), 2);
                        let val = u16::from_le_bytes(bytes);
                        if binding.is_integer {
                            dest[i] = val as u32;
                        } else if binding.normalized {
                            dest[i] = (val as f32 / 65535.0).to_bits();
                        } else {
                            dest[i] = (val as f32).to_bits();
                        }
                    }
                    0x1402 /* SHORT */ => {
                        let mut bytes = [0u8; 2];
                        std::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr(), 2);
                        let val = i16::from_le_bytes(bytes);
                        if binding.is_integer {
                            dest[i] = val as i32 as u32;
                        } else if binding.normalized {
                            dest[i] = (val as f32 / 32767.0).clamp(-1.0, 1.0).to_bits();
                        } else {
                            dest[i] = (val as f32).to_bits();
                        }
                    }
                    0x1405 /* UNSIGNED_INT */ => {
                        let mut bytes = [0u8; 4];
                        std::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr(), 4);
                        let val = u32::from_le_bytes(bytes);
                        dest[i] = val;
                    }
                    0x1404 /* INT */ => {
                        let mut bytes = [0u8; 4];
                        std::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr(), 4);
                        let val = i32::from_le_bytes(bytes);
                        dest[i] = val as u32;
                    }
                    _ => {}
                }
            }
        }
    }

    /// Writes texture metadata to the specified linear memory pointer for shader access
    pub fn write_texture_metadata(
        kernel: &crate::wasm_gl_emu::device::GpuKernel,
        bindings: &[Option<super::device::TextureBinding>],
        dest_ptr: u32,
    ) {
        kernel.write_texture_metadata(bindings, dest_ptr);
    }

    /// Downscale an RGBA8 buffer by 2x in each dimension (box filter)
    pub fn downscale_rgba8(
        src: &[u8],
        src_w: u32,
        src_h: u32,
        dst: &mut [u8],
        dst_w: u32,
        dst_h: u32,
    ) {
        let bpp = 4;
        for y in 0..dst_h {
            for x in 0..dst_w {
                let src_x = x * 2;
                let src_y = y * 2;
                let mut r_sum = 0u32;
                let mut g_sum = 0u32;
                let mut b_sum = 0u32;
                let mut a_sum = 0u32;
                let mut count = 0u32;

                for dy in 0..2 {
                    for dx in 0..2 {
                        let sx = src_x + dx;
                        let sy = src_y + dy;
                        if sx < src_w && sy < src_h {
                            let idx = ((sy * src_w + sx) * bpp) as usize;
                            if idx + 4 <= src.len() {
                                r_sum += src[idx] as u32;
                                g_sum += src[idx + 1] as u32;
                                b_sum += src[idx + 2] as u32;
                                a_sum += src[idx + 3] as u32;
                                count += 1;
                            }
                        }
                    }
                }

                if count > 0 {
                    let out_idx = ((y * dst_w + x) * bpp) as usize;
                    if out_idx + 4 <= dst.len() {
                        dst[out_idx] = (r_sum / count) as u8;
                        dst[out_idx + 1] = (g_sum / count) as u8;
                        dst[out_idx + 2] = (b_sum / count) as u8;
                        dst[out_idx + 3] = (a_sum / count) as u8;
                    }
                }
            }
        }
    }

    /// Copy pixels from a GPU buffer to a host buffer (linear)
    pub fn read_pixels(request: &TransferRequest, dest: &mut [u8]) {
        let src = request.src_buffer;
        let dst_bpp = request.dst_format.block_copy_size(None).unwrap_or(4) as usize;

        for row in 0..request.height {
            for col in 0..request.width {
                let sx = (request.x + col as i32) as u32;
                let sy = (request.y + row as i32) as u32;

                if sx < src.width && sy < src.height {
                    let src_off = src.get_pixel_offset(sx, sy, 0);
                    let dst_off = (row * request.width + col) as usize * dst_bpp;

                    if dst_off + dst_bpp > dest.len() {
                        continue;
                    }

                    match src.format {
                        wgt::TextureFormat::Rgba8Unorm => {
                            if dst_bpp == 4 {
                                dest[dst_off..dst_off + 4]
                                    .copy_from_slice(&src.data[src_off..src_off + 4]);
                            } else if dst_bpp == 16 {
                                // RGBA8 -> RGBA32F
                                for i in 0..4 {
                                    let val = src.data[src_off + i] as f32 / 255.0;
                                    dest[dst_off + i * 4..dst_off + (i + 1) * 4]
                                        .copy_from_slice(&val.to_ne_bytes());
                                }
                            }
                        }
                        wgt::TextureFormat::R16Uint => {
                            // RGB565 (stored as BGR565 bits in our impl)
                            let val =
                                u16::from_ne_bytes([src.data[src_off], src.data[src_off + 1]]);
                            let b5 = ((val >> 11) & 0x1F) as u8;
                            let g6 = ((val >> 5) & 0x3F) as u8;
                            let r5 = (val & 0x1F) as u8;

                            if dst_bpp == 4 {
                                dest[dst_off] = (r5 << 3) | (r5 >> 2);
                                dest[dst_off + 1] = (g6 << 2) | (g6 >> 4);
                                dest[dst_off + 2] = (b5 << 3) | (b5 >> 2);
                                dest[dst_off + 3] = 255;
                            }
                        }
                        wgt::TextureFormat::Rg8Uint => {
                            // RGBA4
                            let val =
                                u16::from_ne_bytes([src.data[src_off], src.data[src_off + 1]]);
                            let r4 = ((val >> 12) & 0xF) as u8;
                            let g4 = ((val >> 8) & 0xF) as u8;
                            let b4 = ((val >> 4) & 0xF) as u8;
                            let a4 = (val & 0xF) as u8;

                            if dst_bpp == 4 {
                                dest[dst_off] = (r4 << 4) | r4;
                                dest[dst_off + 1] = (g4 << 4) | g4;
                                dest[dst_off + 2] = (b4 << 4) | b4;
                                dest[dst_off + 3] = (a4 << 4) | a4;
                            }
                        }
                        wgt::TextureFormat::R16Sint => {
                            // RGB5_A1
                            let val =
                                u16::from_ne_bytes([src.data[src_off], src.data[src_off + 1]]);
                            let r5 = ((val >> 11) & 0x1F) as u8;
                            let g5 = ((val >> 6) & 0x1F) as u8;
                            let b5 = ((val >> 1) & 0x1F) as u8;
                            let a1 = (val & 1) as u8;

                            if dst_bpp == 4 {
                                dest[dst_off] = (r5 << 3) | (r5 >> 2);
                                dest[dst_off + 1] = (g5 << 3) | (g5 >> 2);
                                dest[dst_off + 2] = (b5 << 3) | (b5 >> 2);
                                dest[dst_off + 3] = if a1 != 0 { 255 } else { 0 };
                            }
                        }
                        wgt::TextureFormat::Rgba32Float => {
                            if dst_bpp == 16 {
                                dest[dst_off..dst_off + 16]
                                    .copy_from_slice(&src.data[src_off..src_off + 16]);
                            } else if dst_bpp == 4 {
                                // RGBA32F -> RGBA8
                                for i in 0..4 {
                                    let mut bits = [0u8; 4];
                                    bits.copy_from_slice(
                                        &src.data[src_off + i * 4..src_off + (i + 1) * 4],
                                    );
                                    let val = f32::from_ne_bytes(bits);
                                    dest[dst_off + i] = (val.clamp(0.0, 1.0) * 255.0).round() as u8;
                                }
                            }
                        }
                        wgt::TextureFormat::R32Float => {
                            if dst_bpp == 4 {
                                dest[dst_off..dst_off + 4]
                                    .copy_from_slice(&src.data[src_off..src_off + 4]);
                            } else if dst_bpp == 1 {
                                // R32F -> R8
                                let mut bits = [0u8; 4];
                                bits.copy_from_slice(&src.data[src_off..src_off + 4]);
                                let val = f32::from_ne_bytes(bits);
                                dest[dst_off] = (val.clamp(0.0, 1.0) * 255.0).round() as u8;
                            }
                        }
                        wgt::TextureFormat::Rgba32Uint | wgt::TextureFormat::Rgba32Sint => {
                            let bytes_to_copy = dst_bpp.min(16);
                            dest[dst_off..dst_off + bytes_to_copy]
                                .copy_from_slice(&src.data[src_off..src_off + bytes_to_copy]);
                        }
                        wgt::TextureFormat::R32Uint | wgt::TextureFormat::R32Sint => {
                            let bytes_to_copy = dst_bpp.min(4);
                            dest[dst_off..dst_off + bytes_to_copy]
                                .copy_from_slice(&src.data[src_off..src_off + bytes_to_copy]);
                        }
                        _ => {
                            // Fallback raw copy
                            let src_bpp = src.format.block_copy_size(None).unwrap_or(4) as usize;
                            let bytes_to_copy = src_bpp.min(dst_bpp);
                            dest[dst_off..dst_off + bytes_to_copy]
                                .copy_from_slice(&src.data[src_off..src_off + bytes_to_copy]);
                        }
                    }
                }
            }
        }
    }

    /// Copy pixels from a host buffer to a GPU buffer (handles layout)
    #[allow(clippy::too_many_arguments)]
    pub fn write_pixels(
        kernel: &mut super::device::GpuKernel,
        handle: super::device::GpuHandle,
        x: i32,
        y: i32,
        z: i32,
        width: u32,
        height: u32,
        depth: u32,
        src: &[u8],
    ) {
        if let Some(buf) = kernel.get_buffer_mut(handle) {
            let bpp = buf.format.block_copy_size(None).unwrap_or(4) as usize;
            let src_stride = width as usize * bpp;
            let layer_size = height as usize * src_stride;

            for d in 0..depth {
                for row in 0..height {
                    for col in 0..width {
                        let dx = (x + col as i32) as u32;
                        let dy = (y + row as i32) as u32;
                        let dz = (z + d as i32) as u32;

                        if dx < buf.width && dy < buf.height && dz < buf.depth {
                            let dst_off = buf.get_pixel_offset(dx, dy, dz);
                            let src_off = (d as usize * layer_size)
                                + (row as usize * src_stride)
                                + (col as usize * bpp);

                            if dst_off + bpp <= buf.data.len() && src_off + bpp <= src.len() {
                                buf.data[dst_off..dst_off + bpp]
                                    .copy_from_slice(&src[src_off..src_off + bpp]);
                            }
                        }
                    }
                }
            }
        }
    }
}
