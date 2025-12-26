use super::registry::{clear_last_error, get_registry, set_last_error};
use super::types::*;
use crate::wasm_gl_emu::rasterizer::{
    ProcessedVertex, RasterPipeline, RenderState, ShaderMemoryLayout,
};

struct Vertex {
    pos: [f32; 4],
    varyings: Vec<u8>,
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

    for instance_id in 0..instance_count {
        let mut vertices = Vec::with_capacity(count as usize);

        // 1. Run VS for all vertices
        let mut attr_data = vec![0.0f32; 16 * 4];
        for i in 0..count {
            ctx_obj.fetch_vertex_attributes((first + i) as u32, instance_id as u32, &mut attr_data);

            let attr_ptr: u32 = 0x2000;
            let uniform_ptr: u32 = 0x1000;
            let varying_ptr: u32 = 0x3000;
            let private_ptr: u32 = 0x4000;
            let texture_ptr: u32 = 0x5000;

            // Ensure attr_data is large enough for 64-byte alignment per location
            let mut aligned_attr_data = vec![0.0f32; 1024]; // Enough for 16 locations * 16 floats
            for (loc, chunk) in attr_data.chunks(4).enumerate() {
                let offset = loc * 16;
                if offset + 4 <= aligned_attr_data.len() {
                    aligned_attr_data[offset..offset + 4].copy_from_slice(chunk);
                }
            }

            unsafe {
                std::ptr::copy_nonoverlapping(
                    aligned_attr_data.as_ptr() as *const u8,
                    attr_ptr as *mut u8,
                    aligned_attr_data.len() * 4,
                );
                std::ptr::copy_nonoverlapping(
                    ctx_obj.uniform_data.as_ptr(),
                    uniform_ptr as *mut u8,
                    ctx_obj.uniform_data.len(),
                );
                ctx_obj.prepare_texture_metadata(texture_ptr);
            }

            crate::js_execute_shader(
                0x8B31, /* VERTEX_SHADER */
                attr_ptr,
                uniform_ptr,
                varying_ptr,
                private_ptr,
                texture_ptr,
            );

            let mut pos_bytes = [0u8; 16];
            let mut varying_bytes = vec![0u8; 256]; // Capture first 256 bytes of varyings
            unsafe {
                std::ptr::copy_nonoverlapping(varying_ptr as *const u8, pos_bytes.as_mut_ptr(), 16);
                std::ptr::copy_nonoverlapping(
                    varying_ptr as *const u8,
                    varying_bytes.as_mut_ptr(),
                    256,
                );
            }
            let pos: [f32; 4] = unsafe { std::mem::transmute(pos_bytes) };

            vertices.push(Vertex {
                pos,
                varyings: varying_bytes,
            });
        }

        // 2. Rasterize
        if mode == 0x0000 {
            // GL_POINTS
            for v in &vertices {
                let screen_x = vx as f32 + (v.pos[0] / v.pos[3] + 1.0) * 0.5 * vw as f32;
                let screen_y = vy as f32 + (v.pos[1] / v.pos[3] + 1.0) * 0.5 * vh as f32;

                // Run FS
                let uniform_ptr: u32 = 0x1000;
                let varying_ptr: u32 = 0x3000;
                let private_ptr: u32 = 0x4000;
                let texture_ptr: u32 = 0x5000;

                unsafe {
                    std::ptr::copy_nonoverlapping(
                        v.varyings.as_ptr(),
                        varying_ptr as *mut u8,
                        v.varyings.len(),
                    );
                }

                crate::js_execute_shader(
                    0x8B30, /* FRAGMENT_SHADER */
                    0,
                    uniform_ptr,
                    varying_ptr,
                    private_ptr,
                    texture_ptr,
                );

                let mut color_bytes = [0u8; 16];
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        private_ptr as *const u8,
                        color_bytes.as_mut_ptr(),
                        16,
                    );
                }
                let c: [f32; 4] = unsafe { std::mem::transmute(color_bytes) };
                let color_u8 = [
                    (c[0].clamp(0.0, 1.0) * 255.0) as u8,
                    (c[1].clamp(0.0, 1.0) * 255.0) as u8,
                    (c[2].clamp(0.0, 1.0) * 255.0) as u8,
                    (c[3].clamp(0.0, 1.0) * 255.0) as u8,
                ];
                ctx_obj.rasterizer.draw_point(
                    &mut ctx_obj.default_framebuffer,
                    screen_x,
                    screen_y,
                    color_u8,
                );
            }
        } else if mode == 0x0004 {
            // GL_TRIANGLES - use shared rasterizer
            // Create pipeline configuration
            let pipeline = RasterPipeline::default();
            let state = RenderState {
                memory: ShaderMemoryLayout::default(),
                viewport: (vx, vy, vw, vh),
                uniform_data: &ctx_obj.uniform_data,
                prepare_textures: None,
            };

            for i in (0..vertices.len()).step_by(3) {
                if i + 2 >= vertices.len() {
                    break;
                }
                let v0 = &vertices[i];
                let v1 = &vertices[i + 1];
                let v2 = &vertices[i + 2];

                // Convert to ProcessedVertex format
                let v0_f32: Vec<f32> =
                    unsafe { std::slice::from_raw_parts(v0.varyings.as_ptr() as *const f32, 64) }
                        .to_vec();
                let v1_f32: Vec<f32> =
                    unsafe { std::slice::from_raw_parts(v1.varyings.as_ptr() as *const f32, 64) }
                        .to_vec();
                let v2_f32: Vec<f32> =
                    unsafe { std::slice::from_raw_parts(v2.varyings.as_ptr() as *const f32, 64) }
                        .to_vec();

                let pv0 = ProcessedVertex {
                    position: v0.pos,
                    varyings: v0_f32,
                };
                let pv1 = ProcessedVertex {
                    position: v1.pos,
                    varyings: v1_f32,
                };
                let pv2 = ProcessedVertex {
                    position: v2.pos,
                    varyings: v2_f32,
                };

                // Use shared rasterizer
                ctx_obj.rasterizer.rasterize_triangle(
                    &mut ctx_obj.default_framebuffer,
                    &pv0,
                    &pv1,
                    &pv2,
                    &pipeline,
                    &state,
                );
            }
        }
    }
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

    for instance_id in 0..instance_count {
        let mut vertices = Vec::with_capacity(count as usize);

        // 1. Run VS for all vertices
        let mut attr_data = vec![0.0f32; 16 * 4];
        for &index in &indices {
            ctx_obj.fetch_vertex_attributes(index, instance_id as u32, &mut attr_data);

            let attr_ptr: u32 = 0x2000;
            let uniform_ptr: u32 = 0x1000;
            let varying_ptr: u32 = 0x3000;
            let private_ptr: u32 = 0x4000;
            let texture_ptr: u32 = 0x5000;

            // Ensure attr_data is large enough for 64-byte alignment per location
            let mut aligned_attr_data = vec![0.0f32; 1024]; // Enough for 16 locations * 16 floats
            for (loc, chunk) in attr_data.chunks(4).enumerate() {
                let offset = loc * 16;
                if offset + 4 <= aligned_attr_data.len() {
                    aligned_attr_data[offset..offset + 4].copy_from_slice(chunk);
                }
            }

            unsafe {
                std::ptr::copy_nonoverlapping(
                    aligned_attr_data.as_ptr() as *const u8,
                    attr_ptr as *mut u8,
                    aligned_attr_data.len() * 4,
                );
                std::ptr::copy_nonoverlapping(
                    ctx_obj.uniform_data.as_ptr(),
                    uniform_ptr as *mut u8,
                    ctx_obj.uniform_data.len(),
                );
                ctx_obj.prepare_texture_metadata(texture_ptr);
            }

            crate::js_execute_shader(
                0x8B31, /* VERTEX_SHADER */
                attr_ptr,
                uniform_ptr,
                varying_ptr,
                private_ptr,
                texture_ptr,
            );

            let mut pos_bytes = [0u8; 16];
            let mut varying_bytes = vec![0u8; 256]; // Capture first 256 bytes of varyings
            unsafe {
                std::ptr::copy_nonoverlapping(varying_ptr as *const u8, pos_bytes.as_mut_ptr(), 16);
                std::ptr::copy_nonoverlapping(
                    varying_ptr as *const u8,
                    varying_bytes.as_mut_ptr(),
                    256,
                );
            }
            let pos: [f32; 4] = unsafe { std::mem::transmute(pos_bytes) };

            vertices.push(Vertex {
                pos,
                varyings: varying_bytes,
            });
        }

        // 2. Rasterize
        if mode == 0x0000 {
            // GL_POINTS
            for v in &vertices {
                let screen_x = vx as f32 + (v.pos[0] / v.pos[3] + 1.0) * 0.5 * vw as f32;
                let screen_y = vy as f32 + (v.pos[1] / v.pos[3] + 1.0) * 0.5 * vh as f32;

                // Run FS
                let uniform_ptr: u32 = 0x1000;
                let varying_ptr: u32 = 0x3000;
                let private_ptr: u32 = 0x4000;
                let texture_ptr: u32 = 0x5000;

                unsafe {
                    std::ptr::copy_nonoverlapping(
                        v.varyings.as_ptr(),
                        varying_ptr as *mut u8,
                        v.varyings.len(),
                    );
                }

                crate::js_execute_shader(
                    0x8B30, /* FRAGMENT_SHADER */
                    0,
                    uniform_ptr,
                    varying_ptr,
                    private_ptr,
                    texture_ptr,
                );

                let mut color_bytes = [0u8; 16];
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        private_ptr as *const u8,
                        color_bytes.as_mut_ptr(),
                        16,
                    );
                }
                let c: [f32; 4] = unsafe { std::mem::transmute(color_bytes) };
                let color_u8 = [
                    (c[0].clamp(0.0, 1.0) * 255.0) as u8,
                    (c[1].clamp(0.0, 1.0) * 255.0) as u8,
                    (c[2].clamp(0.0, 1.0) * 255.0) as u8,
                    (c[3].clamp(0.0, 1.0) * 255.0) as u8,
                ];
                ctx_obj.rasterizer.draw_point(
                    &mut ctx_obj.default_framebuffer,
                    screen_x,
                    screen_y,
                    color_u8,
                );
            }
        } else if mode == 0x0004 {
            // GL_TRIANGLES - use shared rasterizer
            let pipeline = RasterPipeline::default();
            let state = RenderState {
                memory: ShaderMemoryLayout::default(),
                viewport: (vx, vy, vw, vh),
                uniform_data: &ctx_obj.uniform_data,
                prepare_textures: None,
            };

            for i in (0..vertices.len()).step_by(3) {
                if i + 2 >= vertices.len() {
                    break;
                }
                let v0 = &vertices[i];
                let v1 = &vertices[i + 1];
                let v2 = &vertices[i + 2];

                // Convert to ProcessedVertex format
                let v0_f32: Vec<f32> =
                    unsafe { std::slice::from_raw_parts(v0.varyings.as_ptr() as *const f32, 64) }
                        .to_vec();
                let v1_f32: Vec<f32> =
                    unsafe { std::slice::from_raw_parts(v1.varyings.as_ptr() as *const f32, 64) }
                        .to_vec();
                let v2_f32: Vec<f32> =
                    unsafe { std::slice::from_raw_parts(v2.varyings.as_ptr() as *const f32, 64) }
                        .to_vec();

                let pv0 = ProcessedVertex {
                    position: v0.pos,
                    varyings: v0_f32,
                };
                let pv1 = ProcessedVertex {
                    position: v1.pos,
                    varyings: v1_f32,
                };
                let pv2 = ProcessedVertex {
                    position: v2.pos,
                    varyings: v2_f32,
                };

                // Use shared rasterizer
                ctx_obj.rasterizer.rasterize_triangle(
                    &mut ctx_obj.default_framebuffer,
                    &pv0,
                    &pv1,
                    &pv2,
                    &pipeline,
                    &state,
                );
            }
        }
    }
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
    let (src_data, src_width, src_height) = if let Some(fb_handle) = ctx_obj.bound_framebuffer {
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
                (&tex.data, tex.width, tex.height)
            }
            Some(Attachment::Renderbuffer(rb_handle)) => {
                let rb = match ctx_obj.renderbuffers.get(&rb_handle) {
                    Some(r) => r,
                    None => {
                        set_last_error("attached renderbuffer not found");
                        return ERR_INVALID_HANDLE;
                    }
                };
                (&rb.data, rb.width, rb.height)
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
