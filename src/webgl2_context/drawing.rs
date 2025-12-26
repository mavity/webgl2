use super::registry::{get_registry, set_last_error, clear_last_error};
use super::types::*;

struct Vertex {
    pos: [f32; 4],
    varyings: Vec<u8>,
}

fn barycentric(p: (f32, f32), a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> (f32, f32, f32) {
    let area = (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0);
    if area.abs() < 1e-6 {
        return (-1.0, -1.0, -1.0);
    }
    let w0 = ((b.0 - p.0) * (c.1 - p.1) - (b.1 - p.1) * (c.0 - p.0)) / area;
    let w1 = ((c.0 - p.0) * (a.1 - p.1) - (c.1 - p.1) * (a.0 - p.0)) / area;
    let w2 = 1.0 - w0 - w1;
    (w0, w1, w2)
}

/// Draw arrays.
pub fn ctx_draw_arrays(ctx: u32, mode: u32, first: i32, count: i32) -> u32 {
    ctx_draw_arrays_instanced(ctx, mode, first, count, 1)
}

/// Draw arrays instanced.
pub fn ctx_draw_arrays_instanced(ctx: u32, mode: u32, first: i32, count: i32, instance_count: i32) -> u32 {
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
            // GL_TRIANGLES
            for i in (0..vertices.len()).step_by(3) {
                if i + 2 >= vertices.len() {
                    break;
                }
                let v0 = &vertices[i];
                let v1 = &vertices[i + 1];
                let v2 = &vertices[i + 2];

                // Screen coordinates (with perspective divide)
                let p0 = (
                    vx as f32 + (v0.pos[0] / v0.pos[3] + 1.0) * 0.5 * vw as f32,
                    vy as f32 + (v0.pos[1] / v0.pos[3] + 1.0) * 0.5 * vh as f32,
                );
                let p1 = (
                    vx as f32 + (v1.pos[0] / v1.pos[3] + 1.0) * 0.5 * vw as f32,
                    vy as f32 + (v1.pos[1] / v1.pos[3] + 1.0) * 0.5 * vh as f32,
                );
                let p2 = (
                    vx as f32 + (v2.pos[0] / v2.pos[3] + 1.0) * 0.5 * vw as f32,
                    vy as f32 + (v2.pos[1] / v2.pos[3] + 1.0) * 0.5 * vh as f32,
                );

                // Bounding box
                let min_x = p0.0.min(p1.0).min(p2.0).max(0.0).floor() as i32;
                let max_x = p0.0.max(p1.0).max(p2.0).min(vw as f32 - 1.0).ceil() as i32;
                let min_y = p0.1.min(p1.1).min(p2.1).max(0.0).floor() as i32;
                let max_y = p0.1.max(p1.1).max(p2.1).min(vh as f32 - 1.0).ceil() as i32;

                if max_x >= min_x && max_y >= min_y {
                    let w0_inv = 1.0 / v0.pos[3];
                    let w1_inv = 1.0 / v1.pos[3];
                    let w2_inv = 1.0 / v2.pos[3];

                    let v0_f32: &[f32] =
                        unsafe { std::slice::from_raw_parts(v0.varyings.as_ptr() as *const f32, 64) };
                    let v1_f32: &[f32] =
                        unsafe { std::slice::from_raw_parts(v1.varyings.as_ptr() as *const f32, 64) };
                    let v2_f32: &[f32] =
                        unsafe { std::slice::from_raw_parts(v2.varyings.as_ptr() as *const f32, 64) };

                    for y in min_y..=max_y {
                        for x in min_x..=max_x {
                            let (u, v, w) = barycentric((x as f32 + 0.5, y as f32 + 0.5), p0, p1, p2);
                            if u >= 0.0 && v >= 0.0 && w >= 0.0 {
                                // Interpolate depth (NDC z/w mapped to [0, 1])
                                let z0 = v0.pos[2] / v0.pos[3];
                                let z1 = v1.pos[2] / v1.pos[3];
                                let z2 = v2.pos[2] / v2.pos[3];
                                let depth_ndc = u * z0 + v * z1 + w * z2;
                                let depth = (depth_ndc + 1.0) * 0.5;

                                let fb_idx =
                                    (y as u32 * ctx_obj.default_framebuffer.width + x as u32) as usize;

                                if (0.0..=1.0).contains(&depth)
                                    && depth < ctx_obj.default_framebuffer.depth[fb_idx]
                                {
                                    ctx_obj.default_framebuffer.depth[fb_idx] = depth;

                                    // Perspective correct interpolation
                                    let w_interp_inv = u * w0_inv + v * w1_inv + w * w2_inv;
                                    let w_interp = 1.0 / w_interp_inv;

                                    let mut interp_f32 = [0.0f32; 64];
                                    for k in 0..64 {
                                        interp_f32[k] = (u * v0_f32[k] * w0_inv
                                            + v * v1_f32[k] * w1_inv
                                            + w * v2_f32[k] * w2_inv)
                                            * w_interp;
                                    }

                                    // Run FS
                                    let uniform_ptr: u32 = 0x1000;
                                    let varying_ptr: u32 = 0x3000;
                                    let private_ptr: u32 = 0x4000;
                                    let texture_ptr: u32 = 0x5000;

                                    unsafe {
                                        std::ptr::copy_nonoverlapping(
                                            interp_f32.as_ptr() as *const u8,
                                            varying_ptr as *mut u8,
                                            256,
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

                                    let color_idx = fb_idx * 4;
                                    if color_idx + 3 < ctx_obj.default_framebuffer.color.len() {
                                        ctx_obj.default_framebuffer.color[color_idx..color_idx + 4]
                                            .copy_from_slice(&color_u8);
                                    }
                                }
                            }
                        }
                    }
                }
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
        // GL_TRIANGLES
        for i in (0..vertices.len()).step_by(3) {
            if i + 2 >= vertices.len() {
                break;
            }
            let v0 = &vertices[i];
            let v1 = &vertices[i + 1];
            let v2 = &vertices[i + 2];

            // Screen coordinates (with perspective divide)
            let p0 = (
                vx as f32 + (v0.pos[0] / v0.pos[3] + 1.0) * 0.5 * vw as f32,
                vy as f32 + (v0.pos[1] / v0.pos[3] + 1.0) * 0.5 * vh as f32,
            );
            let p1 = (
                vx as f32 + (v1.pos[0] / v1.pos[3] + 1.0) * 0.5 * vw as f32,
                vy as f32 + (v1.pos[1] / v1.pos[3] + 1.0) * 0.5 * vh as f32,
            );
            let p2 = (
                vx as f32 + (v2.pos[0] / v2.pos[3] + 1.0) * 0.5 * vw as f32,
                vy as f32 + (v2.pos[1] / v2.pos[3] + 1.0) * 0.5 * vh as f32,
            );

            // Bounding box
            let min_x = p0.0.min(p1.0).min(p2.0).max(0.0).floor() as i32;
            let max_x = p0.0.max(p1.0).max(p2.0).min(vw as f32 - 1.0).ceil() as i32;
            let min_y = p0.1.min(p1.1).min(p2.1).max(0.0).floor() as i32;
            let max_y = p0.1.max(p1.1).max(p2.1).min(vh as f32 - 1.0).ceil() as i32;

            if max_x >= min_x && max_y >= min_y {
                let w0_inv = 1.0 / v0.pos[3];
                let w1_inv = 1.0 / v1.pos[3];
                let w2_inv = 1.0 / v2.pos[3];

                let v0_f32: &[f32] =
                    unsafe { std::slice::from_raw_parts(v0.varyings.as_ptr() as *const f32, 64) };
                let v1_f32: &[f32] =
                    unsafe { std::slice::from_raw_parts(v1.varyings.as_ptr() as *const f32, 64) };
                let v2_f32: &[f32] =
                    unsafe { std::slice::from_raw_parts(v2.varyings.as_ptr() as *const f32, 64) };

                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        let (u, v, w) = barycentric((x as f32 + 0.5, y as f32 + 0.5), p0, p1, p2);
                        if u >= 0.0 && v >= 0.0 && w >= 0.0 {
                            // Interpolate depth (NDC z/w mapped to [0, 1])
                            let z0 = v0.pos[2] / v0.pos[3];
                            let z1 = v1.pos[2] / v1.pos[3];
                            let z2 = v2.pos[2] / v2.pos[3];
                            let depth_ndc = u * z0 + v * z1 + w * z2;
                            let depth = (depth_ndc + 1.0) * 0.5;

                            let fb_idx =
                                (y as u32 * ctx_obj.default_framebuffer.width + x as u32) as usize;

                            if (0.0..=1.0).contains(&depth)
                                && depth < ctx_obj.default_framebuffer.depth[fb_idx]
                            {
                                ctx_obj.default_framebuffer.depth[fb_idx] = depth;

                                // Perspective correct interpolation
                                let w_interp_inv = u * w0_inv + v * w1_inv + w * w2_inv;
                                let w_interp = 1.0 / w_interp_inv;

                                let mut interp_f32 = [0.0f32; 64];
                                for k in 0..64 {
                                    interp_f32[k] = (u * v0_f32[k] * w0_inv
                                        + v * v1_f32[k] * w1_inv
                                        + w * v2_f32[k] * w2_inv)
                                        * w_interp;
                                }

                                // Run FS
                                let uniform_ptr: u32 = 0x1000;
                                let varying_ptr: u32 = 0x3000;
                                let private_ptr: u32 = 0x4000;
                                let texture_ptr: u32 = 0x5000;

                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        interp_f32.as_ptr() as *const u8,
                                        varying_ptr as *mut u8,
                                        256,
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

                                let color_idx = fb_idx * 4;
                                if color_idx + 3 < ctx_obj.default_framebuffer.color.len() {
                                    ctx_obj.default_framebuffer.color[color_idx..color_idx + 4]
                                        .copy_from_slice(&color_u8);
                                }
                            }
                        }
                    }
                }
            }
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
