//! Pixel Transfer Engine
//!
//! Handles format conversion, de-tiling, and layout translation
//! between GPU buffers and host memory.

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

pub struct TransferEngine;

impl TransferEngine {
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
