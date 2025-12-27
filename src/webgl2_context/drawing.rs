use super::registry::{clear_last_error, get_registry, set_last_error};
use super::types::*;
use crate::wasm_gl_emu::rasterizer::{
    RasterPipeline, RenderState, ShaderMemoryLayout, VertexFetcher,
};

use std::collections::HashMap;

struct WebGLVertexFetcher<'a> {
    vertex_arrays: &'a HashMap<u32, VertexArray>,
    bound_vertex_array: u32,
    buffers: &'a HashMap<u32, Buffer>,
}

impl<'a> VertexFetcher for WebGLVertexFetcher<'a> {
    fn fetch(&self, vertex_index: u32, instance_index: u32, dest: &mut [u8]) {
        let mut attr_data = vec![0.0f32; 16 * 4];
        Context::fetch_vertex_attributes_static(
            self.vertex_arrays,
            self.bound_vertex_array,
            self.buffers,
            vertex_index,
            instance_index,
            &mut attr_data,
        );

        // Clear dest
        dest.fill(0);

        // Alignment logic: 16 locations, each 64 bytes (16 floats)
        for (loc, chunk) in attr_data.chunks(4).enumerate() {
            let dest_offset = loc * 64;
            if dest_offset + 16 <= dest.len() {
                let bytes =
                    unsafe { std::slice::from_raw_parts(chunk.as_ptr() as *const u8, 16) };
                dest[dest_offset..dest_offset + 16].copy_from_slice(bytes);
            }
        }
    }
}

/// Draw arrays.
pub fn ctx_draw_arrays(ctx: u32, mode: u32, first: i32, count: i32) -> u32 {
    ctx_draw_arrays_instanced(ctx, mode, first, count, 1)
}

/// Draw arrays instanced.
pub fn ctx_draw_arrays_instanced(
    ctx: u32,
    mode: u32,
    first: i32,
    count: i32,
    instance_count: i32,
) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let reg_ptr = &mut *reg;

    let ctx_obj = match reg_ptr.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    let _program_id = match ctx_obj.current_program {
        Some(p) => p,
        None => {
            set_last_error("no program bound");
            return ERR_INVALID_ARGS;
        }
    };

    let (vx, vy, vw, vh) = ctx_obj.viewport;

    // Create pipeline configuration
    let pipeline = RasterPipeline::default();
    
    // Prepare textures once
    unsafe {
        ctx_obj.prepare_texture_metadata(pipeline.memory.texture_ptr);
    }

    let state = RenderState {
        ctx_handle: ctx,
        memory: ShaderMemoryLayout::default(),
        viewport: (vx, vy, vw, vh),
        uniform_data: &ctx_obj.uniform_data,
        prepare_textures: None,
    };

    let fetcher = WebGLVertexFetcher {
        vertex_arrays: &ctx_obj.vertex_arrays,
        bound_vertex_array: ctx_obj.bound_vertex_array,
        buffers: &ctx_obj.buffers,
    };

    // Calculate indices for draw_arrays (just 0..count)
    // But draw takes indices: Option<&[u32]>.
    // If we pass None, it iterates 0..vertex_count.
    // But we have 'first' parameter.
    // We need to handle 'first'.
    // Rasterizer::draw iterates 0..vertex_count if indices is None.
    // It passes i as vertex_id.
    // If we want start from 'first', we should probably pass indices.
    
    let indices: Vec<u32> = (0..count).map(|i| (first + i) as u32).collect();
    
    ctx_obj.rasterizer.draw(
        &mut ctx_obj.default_framebuffer,
        &pipeline,
        &state,
        &fetcher,
        count as usize,
        instance_count as usize,
        Some(&indices),
        mode,
    );

    ERR_OK
}

/// Draw elements.
pub fn ctx_draw_elements(ctx: u32, mode: u32, count: i32, type_: u32, offset: u32) -> u32 {
    ctx_draw_elements_instanced(ctx, mode, count, type_, offset, 1)
}

pub fn ctx_draw_elements_instanced(
    ctx: u32,
    mode: u32,
    count: i32,
    type_: u32,
    offset: u32,
    instance_count: i32,
) -> u32 {
    clear_last_error();
    let mut reg = get_registry().borrow_mut();
    let reg_ptr = &mut *reg;

    let ctx_obj = match reg_ptr.contexts.get_mut(&ctx) {
        Some(c) => c,
        None => return ERR_INVALID_HANDLE,
    };

    let _program_id = match ctx_obj.current_program {
        Some(p) => p,
        None => {
            set_last_error("no program bound");
            return ERR_INVALID_ARGS;
        }
    };

    // Get EBO
    let ebo_handle = if let Some(vao) = ctx_obj.vertex_arrays.get(&ctx_obj.bound_vertex_array) {
        vao.element_array_buffer
    } else {
        None
    };

    let indices: Vec<u32> = if let Some(h) = ebo_handle {
        if let Some(buf) = ctx_obj.buffers.get(&h) {
            let data = &buf.data;
            let mut idxs = Vec::with_capacity(count as usize);
            for i in 0..count {
                let idx = match type_ {
                    0x1401 => {
                        // GL_UNSIGNED_BYTE
                        let off = (offset as usize) + i as usize;
                        if off < data.len() {
                            data[off] as u32
                        } else {
                            0
                        }
                    }
                    0x1403 => {
                        // GL_UNSIGNED_SHORT
                        let off = (offset as usize) + (i as usize) * 2;
                        if off + 2 <= data.len() {
                            u16::from_ne_bytes([data[off], data[off + 1]]) as u32
                        } else {
                            0
                        }
                    }
                    0x1405 => {
                        // GL_UNSIGNED_INT
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
                    _ => return ERR_INVALID_ENUM,
                };
                idxs.push(idx);
            }
            idxs
        } else {
            return ERR_INVALID_OPERATION;
        }
    } else {
        return ERR_INVALID_OPERATION;
    };

    let (vx, vy, vw, vh) = ctx_obj.viewport;

    // Create pipeline configuration
    let pipeline = RasterPipeline::default();
    
    // Prepare textures once
    unsafe {
        ctx_obj.prepare_texture_metadata(pipeline.memory.texture_ptr);
    }

    let state = RenderState {
        ctx_handle: ctx,
        memory: ShaderMemoryLayout::default(),
        viewport: (vx, vy, vw, vh),
        uniform_data: &ctx_obj.uniform_data,
        prepare_textures: None,
    };

    let fetcher = WebGLVertexFetcher {
        vertex_arrays: &ctx_obj.vertex_arrays,
        bound_vertex_array: ctx_obj.bound_vertex_array,
        buffers: &ctx_obj.buffers,
    };

    ctx_obj.rasterizer.draw(
        &mut ctx_obj.default_framebuffer,
        &pipeline,
        &state,
        &fetcher,
        count as usize,
        instance_count as usize,
        Some(&indices),
        mode,
    );

    ERR_OK
}

// ============================================================================
// Pixel Read
// ============================================================================

/// Read pixels from the currently bound framebuffer's color attachment.
/// Writes RGBA u8 data to dest_ptr in WASM linear memory.
/// Returns errno.
#[allow(clippy::too_many_arguments)]
pub fn ctx_read_pixels(
    ctx: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    _format: u32,
    _type_: u32,
    dest_ptr: u32,
    dest_len: u32,
) -> u32 {
    clear_last_error();

    let reg = get_registry().borrow();
    let ctx_obj = match reg.contexts.get(&ctx) {
        Some(c) => c,
        None => {
            set_last_error("invalid context handle");
            return ERR_INVALID_HANDLE;
        }
    };

    // Get the source data and dimensions
    let (src_data, src_width, src_height, src_format) =
        if let Some(fb_handle) = ctx_obj.bound_framebuffer {
            let fb = match ctx_obj.framebuffers.get(&fb_handle) {
                Some(f) => f,
                None => {
                    set_last_error("framebuffer not found");
                    return ERR_INVALID_HANDLE;
                }
            };

            match fb.color_attachment {
                Some(Attachment::Texture(tex_handle)) => {
                    let tex = match ctx_obj.textures.get(&tex_handle) {
                        Some(t) => t,
                        None => {
                            set_last_error("attached texture not found");
                            return ERR_INVALID_HANDLE;
                        }
                    };
                    (&tex.data, tex.width, tex.height, GL_RGBA8) // Textures are RGBA8 in this impl
                }
                Some(Attachment::Renderbuffer(rb_handle)) => {
                    let rb = match ctx_obj.renderbuffers.get(&rb_handle) {
                        Some(r) => r,
                        None => {
                            set_last_error("attached renderbuffer not found");
                            return ERR_INVALID_HANDLE;
                        }
                    };
                    (&rb.data, rb.width, rb.height, rb.internal_format)
                }
                None => {
                    set_last_error("framebuffer has no color attachment");
                    return ERR_INVALID_ARGS;
                }
            }
        } else {
            (
                &ctx_obj.default_framebuffer.color,
                ctx_obj.default_framebuffer.width,
                ctx_obj.default_framebuffer.height,
                GL_RGBA8,
            )
        };

    // Verify output buffer size
    let expected_size = (width as u64)
        .saturating_mul(height as u64)
        .saturating_mul(4);
    if dest_len as u64 != expected_size {
        set_last_error("output buffer size mismatch");
        return ERR_INVALID_ARGS;
    }

    // Read pixels and write to destination
    let dest_slice =
        unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut u8, dest_len as usize) };

    let mut dst_off = 0;
    for row in 0..height {
        for col in 0..width {
            let sx = x + col as i32;
            let sy = y + row as i32;

            if sx >= 0 && sx < src_width as i32 && sy >= 0 && sy < src_height as i32 {
                match src_format {
                    GL_RGBA8 => {
                        let src_idx = ((sy as u32 * src_width + sx as u32) * 4) as usize;
                        if src_idx + 3 < src_data.len() {
                            dest_slice[dst_off] = src_data[src_idx];
                            dest_slice[dst_off + 1] = src_data[src_idx + 1];
                            dest_slice[dst_off + 2] = src_data[src_idx + 2];
                            dest_slice[dst_off + 3] = src_data[src_idx + 3];
                        } else {
                            dest_slice[dst_off] = 0;
                            dest_slice[dst_off + 1] = 0;
                            dest_slice[dst_off + 2] = 0;
                            dest_slice[dst_off + 3] = 0;
                        }
                    }
                    GL_RGBA4 => {
                        let src_idx = ((sy as u32 * src_width + sx as u32) * 2) as usize;
                        if src_idx + 1 < src_data.len() {
                            let val =
                                u16::from_le_bytes([src_data[src_idx], src_data[src_idx + 1]]);
                            // R4 G4 B4 A4
                            let r = ((val >> 12) & 0xF) as u8;
                            let g = ((val >> 8) & 0xF) as u8;
                            let b = ((val >> 4) & 0xF) as u8;
                            let a = (val & 0xF) as u8;
                            // Expand to 8-bit: (c * 255) / 15  => c * 17
                            dest_slice[dst_off] = r * 17;
                            dest_slice[dst_off + 1] = g * 17;
                            dest_slice[dst_off + 2] = b * 17;
                            dest_slice[dst_off + 3] = a * 17;
                        } else {
                            dest_slice[dst_off] = 0;
                            dest_slice[dst_off + 1] = 0;
                            dest_slice[dst_off + 2] = 0;
                            dest_slice[dst_off + 3] = 0;
                        }
                    }
                    GL_RGB565 => {
                        let src_idx = ((sy as u32 * src_width + sx as u32) * 2) as usize;
                        if src_idx + 1 < src_data.len() {
                            let val =
                                u16::from_le_bytes([src_data[src_idx], src_data[src_idx + 1]]);
                            // R5 G6 B5
                            let r = ((val >> 11) & 0x1F) as u8;
                            let g = ((val >> 5) & 0x3F) as u8;
                            let b = (val & 0x1F) as u8;
                            // Expand
                            // 5-bit: c * 255 / 31 => c * 8.22... approx (c << 3) | (c >> 2)
                            // 6-bit: c * 255 / 63 => c * 4.04... approx (c << 2) | (c >> 4)
                            dest_slice[dst_off] = (r << 3) | (r >> 2);
                            dest_slice[dst_off + 1] = (g << 2) | (g >> 4);
                            dest_slice[dst_off + 2] = (b << 3) | (b >> 2);
                            dest_slice[dst_off + 3] = 255;
                        } else {
                            dest_slice[dst_off] = 0;
                            dest_slice[dst_off + 1] = 0;
                            dest_slice[dst_off + 2] = 0;
                            dest_slice[dst_off + 3] = 255;
                        }
                    }
                    GL_RGB5_A1 => {
                        let src_idx = ((sy as u32 * src_width + sx as u32) * 2) as usize;
                        if src_idx + 1 < src_data.len() {
                            let val =
                                u16::from_le_bytes([src_data[src_idx], src_data[src_idx + 1]]);
                            // R5 G5 B5 A1
                            let r = ((val >> 11) & 0x1F) as u8;
                            let g = ((val >> 6) & 0x1F) as u8;
                            let b = ((val >> 1) & 0x1F) as u8;
                            let a = (val & 0x1) as u8;
                            dest_slice[dst_off] = (r << 3) | (r >> 2);
                            dest_slice[dst_off + 1] = (g << 3) | (g >> 2);
                            dest_slice[dst_off + 2] = (b << 3) | (b >> 2);
                            dest_slice[dst_off + 3] = if a == 1 { 255 } else { 0 };
                        } else {
                            dest_slice[dst_off] = 0;
                            dest_slice[dst_off + 1] = 0;
                            dest_slice[dst_off + 2] = 0;
                            dest_slice[dst_off + 3] = 0;
                        }
                    }
                    _ => {
                        // Unsupported format or depth/stencil, return 0
                        dest_slice[dst_off] = 0;
                        dest_slice[dst_off + 1] = 0;
                        dest_slice[dst_off + 2] = 0;
                        dest_slice[dst_off + 3] = 0;
                    }
                }
            } else {
                // Out of bounds: write transparent black
                dest_slice[dst_off] = 0;
                dest_slice[dst_off + 1] = 0;
                dest_slice[dst_off + 2] = 0;
                dest_slice[dst_off + 3] = 0;
            }
            dst_off += 4;
        }
    }

    ERR_OK
}
