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

pub struct TransferEngine;

impl TransferEngine {
    fn safe_read_u32(data: &[u8], offset: usize) -> u32 {
        if offset + 4 <= data.len() {
            u32::from_ne_bytes(data[offset..offset+4].try_into().unwrap())
        } else if offset < data.len() {
            let mut buf = [0u8; 4];
            let len = data.len() - offset;
            buf[..len].copy_from_slice(&data[offset..offset+len]);
            u32::from_ne_bytes(buf)
        } else {
            0
        }
    }

    fn safe_read_u16(data: &[u8], offset: usize) -> u16 {
        if offset + 2 <= data.len() {
            u16::from_ne_bytes([data[offset], data[offset+1]])
        } else if offset < data.len() {
            data[offset] as u16
        } else {
            0
        }
    }
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
                        u32::from_ne_bytes([
                            data[off],
                            data[off + 1],
                            data[off + 2],
                            data[off + 3],
                        ])
                    } else {
                        0
                    }
                }
            };
            result.push(idx);
        }
        result
    }

    /// Fetch a single vertex attribute component set from a buffer
    pub fn fetch_vertex_attribute(
        src: &GpuBuffer,
        format: wgt::VertexFormat,
        offset: u32,
        effective_index: u32,
        stride: u32,
        dest: &mut [u32; 4],
    ) {
        let data = &src.data;
        let base_offset = offset as usize + (effective_index as usize * stride as usize);
        
        // Zero out dest initially
        dest.fill(0);

        if base_offset >= data.len() {
            return;
        }

        // We use bits representation for floating point values
        match format {
            wgt::VertexFormat::Float32 => {
                dest[0] = Self::safe_read_u32(data, base_offset);
            }
            wgt::VertexFormat::Float32x2 => {
                for i in 0..2 {
                    dest[i] = Self::safe_read_u32(data, base_offset + i * 4);
                }
            }
            wgt::VertexFormat::Float32x3 => {
                for i in 0..3 {
                    dest[i] = Self::safe_read_u32(data, base_offset + i * 4);
                }
            }
            wgt::VertexFormat::Float32x4 => {
                for i in 0..4 {
                    dest[i] = Self::safe_read_u32(data, base_offset + i * 4);
                }
            }
            wgt::VertexFormat::Uint8 => {
                dest[0] = data[base_offset] as u32;
            }
            wgt::VertexFormat::Sint8 => {
                dest[0] = (data[base_offset] as i8) as i32 as u32;
            }
            wgt::VertexFormat::Unorm8 => {
                let val = data[base_offset] as f32 / 255.0;
                dest[0] = val.to_bits();
            }
            wgt::VertexFormat::Snorm8 => {
                let val = (data[base_offset] as i8) as f32 / 127.0;
                dest[0] = val.clamp(-1.0, 1.0).to_bits();
            }
            wgt::VertexFormat::Uint8x2 => {
                for i in 0..2 {
                    if base_offset + i < data.len() {
                        dest[i] = data[base_offset + i] as u32;
                    }
                }
            }
            wgt::VertexFormat::Unorm8x2 => {
                for i in 0..2 {
                    if base_offset + i < data.len() {
                        let val = data[base_offset + i] as f32 / 255.0;
                        dest[i] = val.to_bits();
                    }
                }
            }
            wgt::VertexFormat::Sint8x2 => {
                for i in 0..2 {
                    if base_offset + i < data.len() {
                        dest[i] = (data[base_offset + i] as i8) as i32 as u32;
                    }
                }
            }
            wgt::VertexFormat::Snorm8x2 => {
                for i in 0..2 {
                    if base_offset + i < data.len() {
                        let val = (data[base_offset + i] as i8) as f32 / 127.0;
                        dest[i] = val.clamp(-1.0, 1.0).to_bits();
                    }
                }
            }
            wgt::VertexFormat::Uint8x4 => {
                for i in 0..4 {
                    if base_offset + i < data.len() {
                        dest[i] = data[base_offset + i] as u32;
                    }
                }
            }
            wgt::VertexFormat::Unorm8x4 => {
                for i in 0..4 {
                    if base_offset + i < data.len() {
                        let val = data[base_offset + i] as f32 / 255.0;
                        dest[i] = val.to_bits();
                    }
                }
            }
            wgt::VertexFormat::Sint8x4 => {
                for i in 0..4 {
                    if base_offset + i < data.len() {
                        dest[i] = (data[base_offset + i] as i8) as i32 as u32;
                    }
                }
            }
            wgt::VertexFormat::Snorm8x4 => {
                for i in 0..4 {
                    if base_offset + i < data.len() {
                        let val = (data[base_offset + i] as i8) as f32 / 127.0;
                        dest[i] = val.clamp(-1.0, 1.0).to_bits();
                    }
                }
            }
            wgt::VertexFormat::Uint16 => {
                dest[0] = Self::safe_read_u16(data, base_offset) as u32;
            }
            wgt::VertexFormat::Sint16 => {
                dest[0] = (Self::safe_read_u16(data, base_offset) as i16 as i32) as u32;
            }
            wgt::VertexFormat::Unorm16 => {
                let val = Self::safe_read_u16(data, base_offset) as f32 / 65535.0;
                dest[0] = val.to_bits();
            }
            wgt::VertexFormat::Snorm16 => {
                let val = (Self::safe_read_u16(data, base_offset) as i16 as f32) / 32767.0;
                dest[0] = val.clamp(-1.0, 1.0).to_bits();
            }
            wgt::VertexFormat::Uint16x2 => {
                for i in 0..2 {
                    dest[i] = Self::safe_read_u16(data, base_offset + i * 2) as u32;
                }
            }
            wgt::VertexFormat::Unorm16x2 => {
                for i in 0..2 {
                    let val = Self::safe_read_u16(data, base_offset + i * 2) as f32 / 65535.0;
                    dest[i] = val.to_bits();
                }
            }
            wgt::VertexFormat::Sint16x2 => {
                for i in 0..2 {
                    dest[i] = (Self::safe_read_u16(data, base_offset + i * 2) as i16 as i32) as u32;
                }
            }
            wgt::VertexFormat::Snorm16x2 => {
                for i in 0..2 {
                    let val = (Self::safe_read_u16(data, base_offset + i * 2) as i16 as f32) / 32767.0;
                    dest[i] = val.clamp(-1.0, 1.0).to_bits();
                }
            }
            wgt::VertexFormat::Uint16x4 => {
                for i in 0..4 {
                    dest[i] = Self::safe_read_u16(data, base_offset + i * 2) as u32;
                }
            }
            wgt::VertexFormat::Unorm16x4 => {
                for i in 0..4 {
                    let val = Self::safe_read_u16(data, base_offset + i * 2) as f32 / 65535.0;
                    dest[i] = val.to_bits();
                }
            }
            wgt::VertexFormat::Sint16x4 => {
                for i in 0..4 {
                    dest[i] = (Self::safe_read_u16(data, base_offset + i * 2) as i16 as i32) as u32;
                }
            }
            wgt::VertexFormat::Snorm16x4 => {
                for i in 0..4 {
                    let val = (Self::safe_read_u16(data, base_offset + i * 2) as i16 as f32) / 32767.0;
                    dest[i] = val.clamp(-1.0, 1.0).to_bits();
                }
            }
            wgt::VertexFormat::Uint32 => {
                dest[0] = Self::safe_read_u32(data, base_offset);
            }
            wgt::VertexFormat::Sint32 => {
                dest[0] = Self::safe_read_u32(data, base_offset);
            }
            wgt::VertexFormat::Uint32x2 => {
                for i in 0..2 {
                    dest[i] = Self::safe_read_u32(data, base_offset + i * 4);
                }
            }
            wgt::VertexFormat::Sint32x2 => {
                for i in 0..2 {
                    dest[i] = Self::safe_read_u32(data, base_offset + i * 4);
                }
            }
            wgt::VertexFormat::Uint32x3 => {
                for i in 0..3 {
                    dest[i] = Self::safe_read_u32(data, base_offset + i * 4);
                }
            }
            wgt::VertexFormat::Sint32x3 => {
                for i in 0..3 {
                    dest[i] = Self::safe_read_u32(data, base_offset + i * 4);
                }
            }
            wgt::VertexFormat::Uint32x4 => {
                for i in 0..4 {
                    dest[i] = Self::safe_read_u32(data, base_offset + i * 4);
                }
            }
            wgt::VertexFormat::Sint32x4 => {
                for i in 0..4 {
                    dest[i] = Self::safe_read_u32(data, base_offset + i * 4);
                }
            }
            _ => {
                // Fallback for unimplemented formats
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
                            }
                        }
                        wgt::TextureFormat::R16Uint => {
                            // RGB565
                            let val = u16::from_ne_bytes([src.data[src_off], src.data[src_off + 1]]);
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
                            let val = u16::from_ne_bytes([src.data[src_off], src.data[src_off + 1]]);
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
                            let val = u16::from_ne_bytes([src.data[src_off], src.data[src_off + 1]]);
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
                            let bytes_to_copy = dst_bpp.min(16);
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
    pub fn write_pixels(
        kernel: &mut super::device::GpuKernel,
        handle: super::device::GpuHandle,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        src: &[u8],
    ) {
        if let Some(buf) = kernel.get_buffer_mut(handle) {
            let bpp = buf.format.block_copy_size(None).unwrap_or(4) as usize;
            let src_stride = width as usize * bpp;

            for row in 0..height {
                for col in 0..width {
                    let dx = (x + col as i32) as u32;
                    let dy = (y + row as i32) as u32;

                    if dx < buf.width && dy < buf.height {
                        let dst_off = buf.get_pixel_offset(dx, dy, 0);
                        let src_off = (row as usize * src_stride) + (col as usize * bpp);

                        if dst_off + bpp <= buf.data.len() && src_off + bpp <= src.len() {
                            buf.data[dst_off..dst_off + bpp].copy_from_slice(&src[src_off..src_off + bpp]);
                        }
                    }
                }
            }
        }
    }
}
