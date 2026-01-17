use super::registry::{clear_last_error, get_registry, set_last_error};
use super::types::*;
use crate::wasm_gl_emu::rasterizer::{
    RasterPipeline, RenderState, ShaderMemoryLayout, VertexFetcher,
};

use std::collections::HashMap;

fn get_flat_varyings_mask(ctx: &Context) -> u64 {
    let mut mask = 0u64;
    if let Some(program_id) = ctx.current_program {
        if let Some(program) = ctx.programs.get(&program_id) {
            if let Some(ref fs_module) = program.fs_module {
                // Check entry point arguments
                for ep in fs_module.entry_points.iter() {
                    if ep.stage == naga::ShaderStage::Fragment {
                        for arg in &ep.function.arguments {
                            // If the shader explicitly marked this varying as Flat, preserve that.
                            let mut make_flat = false;
                            if let Some(naga::Binding::Location {
                                location: _,
                                interpolation: Some(interp),
                                ..
                            }) = &arg.binding
                            {
                                if *interp == naga::Interpolation::Flat {
                                    make_flat = true;
                                }
                            }

                            // Additionally, integer/unsigned integer varyings must be flat by the spec.
                            // Detect integer scalar/vector types and mark them flat as well.
                            let ty = &fs_module.types[arg.ty];
                            match ty.inner {
                                naga::TypeInner::Scalar(scalar) => {
                                    if scalar.kind == naga::ScalarKind::Sint
                                        || scalar.kind == naga::ScalarKind::Uint
                                    {
                                        make_flat = true;
                                    }
                                }
                                naga::TypeInner::Vector {
                                    size: _, scalar, ..
                                } => {
                                    if scalar.kind == naga::ScalarKind::Sint
                                        || scalar.kind == naga::ScalarKind::Uint
                                    {
                                        make_flat = true;
                                    }
                                }
                                _ => {}
                            }

                            if make_flat {
                                let start_bit = if let Some(name) = &arg.name {
                                    if let Some(&loc) = program.varying_locations.get(name) {
                                        (loc + 1) * 4
                                    } else if let Some(naga::Binding::Location {
                                        location, ..
                                    }) = &arg.binding
                                    {
                                        (location + 1) * 4
                                    } else {
                                        4
                                    }
                                } else if let Some(naga::Binding::Location { location, .. }) =
                                    &arg.binding
                                {
                                    (location + 1) * 4
                                } else {
                                    4
                                };
                                let count = match ty.inner {
                                    naga::TypeInner::Vector { size, .. } => size as u32,
                                    naga::TypeInner::Matrix { columns, rows, .. } => {
                                        columns as u32 * rows as u32
                                    }
                                    _ => 1,
                                };
                                for i in 0..count {
                                    if start_bit + i < 64 {
                                        mask |= 1 << (start_bit + i);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    mask
}

struct WebGLVertexFetcher<'a> {
    vertex_arrays: &'a HashMap<u32, VertexArray>,
    bound_vertex_array: u32,
    buffers: &'a HashMap<u32, Buffer>,
}

impl<'a> VertexFetcher for WebGLVertexFetcher<'a> {
    fn fetch(&self, vertex_index: u32, instance_index: u32, dest: &mut [u8]) {
        let mut attr_data = vec![0u32; 16 * 4];
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

        // Vertex attribute layout handled via `output_layout::compute_input_offset` (do not duplicate layout math here).

        for (loc, chunk) in attr_data.chunks(4).enumerate() {
            let dest_offset = crate::naga_wasm_backend::output_layout::compute_input_offset(
                loc as u32,
                naga::ShaderStage::Vertex,
            )
            .0 as usize;
            if dest_offset + 16 <= dest.len() {
                let bytes = unsafe { std::slice::from_raw_parts(chunk.as_ptr() as *const u8, 16) };
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

    // Get table indices from program
    let (vs_table_idx, fs_table_idx) = if let Some(prog) = ctx_obj.programs.get(&_program_id) {
        (prog.vs_table_idx, prog.fs_table_idx)
    } else {
        (None, None)
    };

    let (vx, vy, vw, vh) = ctx_obj.viewport;

    // Create pipeline configuration
    let mask = get_flat_varyings_mask(ctx_obj);
    // Build pipeline and include varying debug info if shaders debugging is enabled
    let pipeline = RasterPipeline {
        flat_varyings_mask: mask,
        vs_table_idx,
        fs_table_idx,
        ..Default::default()
    };

    // Prepare textures once
    ctx_obj.prepare_texture_metadata(pipeline.memory.texture_ptr);

    let state = RenderState {
        ctx_handle: ctx,
        memory: ShaderMemoryLayout::default(),
        viewport: (vx, vy, vw, vh),
        uniform_data: &ctx_obj.uniform_data,
        prepare_textures: None,
        blend: ctx_obj.blend_state,
        color_mask: ctx_obj.color_mask,
        depth: ctx_obj.depth_state,
        stencil: ctx_obj.stencil_state,
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

    let (target_handle, target_w, target_h, target_fmt) = ctx_obj.get_current_color_attachment_info();
    let kernel = &mut ctx_obj.kernel;
    let target_buffer = kernel.get_buffer_mut(target_handle).expect("target buffer lost");

    let mut fb = crate::wasm_gl_emu::Framebuffer {
        width: target_w,
        height: target_h,
        internal_format: target_fmt,
        color: &mut target_buffer.data,
        depth: &mut ctx_obj.default_framebuffer.depth,
        stencil: &mut ctx_obj.default_framebuffer.stencil,
    };

    ctx_obj
        .rasterizer
        .draw(crate::wasm_gl_emu::rasterizer::DrawConfig {
            fb: &mut fb,
            pipeline: &pipeline,
            state: &state,
            vertex_fetcher: &fetcher,
            vertex_count: count as usize,
            instance_count: instance_count as usize,
            first_vertex: first as usize,
            first_instance: 0,
            indices: None,
            mode,
        });

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

    // Get table indices from program
    let (vs_table_idx, fs_table_idx) = if let Some(prog) = ctx_obj.programs.get(&_program_id) {
        (prog.vs_table_idx, prog.fs_table_idx)
    } else {
        (None, None)
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
                    GL_UNSIGNED_BYTE => {
                        // GL_UNSIGNED_BYTE
                        let off = (offset as usize) + i as usize;
                        if off < data.len() {
                            data[off] as u32
                        } else {
                            0
                        }
                    }
                    GL_UNSIGNED_SHORT => {
                        // GL_UNSIGNED_SHORT
                        let off = (offset as usize) + (i as usize) * 2;
                        if off + 2 <= data.len() {
                            u16::from_ne_bytes([data[off], data[off + 1]]) as u32
                        } else {
                            0
                        }
                    }
                    GL_UNSIGNED_INT => {
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
    let pipeline = RasterPipeline {
        flat_varyings_mask: get_flat_varyings_mask(ctx_obj),
        vs_table_idx,
        fs_table_idx,
        ..Default::default()
    };

    // Prepare textures once
    ctx_obj.prepare_texture_metadata(pipeline.memory.texture_ptr);

    let state = RenderState {
        ctx_handle: ctx,
        memory: ShaderMemoryLayout::default(),
        viewport: (vx, vy, vw, vh),
        uniform_data: &ctx_obj.uniform_data,
        prepare_textures: None,
        blend: ctx_obj.blend_state,
        color_mask: ctx_obj.color_mask,
        depth: ctx_obj.depth_state,
        stencil: ctx_obj.stencil_state,
    };

    let fetcher = WebGLVertexFetcher {
        vertex_arrays: &ctx_obj.vertex_arrays,
        bound_vertex_array: ctx_obj.bound_vertex_array,
        buffers: &ctx_obj.buffers,
    };

    let (target_handle, target_w, target_h, target_fmt) = ctx_obj.get_current_color_attachment_info();
    let kernel = &mut ctx_obj.kernel;
    let target_buffer = kernel.get_buffer_mut(target_handle).expect("target buffer lost");

    let mut fb = crate::wasm_gl_emu::Framebuffer {
        width: target_w,
        height: target_h,
        internal_format: target_fmt,
        color: &mut target_buffer.data,
        depth: &mut ctx_obj.default_framebuffer.depth,
        stencil: &mut ctx_obj.default_framebuffer.stencil,
    };

    ctx_obj
        .rasterizer
        .draw(crate::wasm_gl_emu::rasterizer::DrawConfig {
            fb: &mut fb,
            pipeline: &pipeline,
            state: &state,
            vertex_fetcher: &fetcher,
            vertex_count: count as usize,
            instance_count: instance_count as usize,
            first_vertex: 0,
            first_instance: 0,
            indices: Some(&indices),
            mode,
        });

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
    format: u32,
    type_: u32,
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

    // Get the source handle and dimensions
    let (src_handle, src_width, src_height, _src_format) =
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
                    if let Some(level) = tex.levels.get(&0) {
                        (
                            level.gpu_handle,
                            level.width,
                            level.height,
                            level.internal_format,
                        )
                    } else {
                        set_last_error("texture incomplete");
                        return ERR_INVALID_OPERATION;
                    }
                }
                Some(Attachment::Renderbuffer(rb_handle)) => {
                    let rb = match ctx_obj.renderbuffers.get(&rb_handle) {
                        Some(r) => r,
                        None => {
                            set_last_error("attached renderbuffer not found");
                            return ERR_INVALID_HANDLE;
                        }
                    };
                    (rb.gpu_handle, rb.width, rb.height, rb.internal_format)
                }
                None => {
                    set_last_error("framebuffer has no color attachment");
                    return ERR_INVALID_ARGS;
                }
            }
        } else {
            (
                ctx_obj.default_framebuffer.gpu_handle,
                ctx_obj.default_framebuffer.width,
                ctx_obj.default_framebuffer.height,
                ctx_obj.default_framebuffer.internal_format,
            )
        };

    // Debug: when reading tiny buffers, print their contents to help debugging
    if src_width == 1 && src_height == 1 {}

    // Calculate bytes per pixel based on format and type
    let bytes_per_pixel = if type_ == GL_FLOAT {
        // GL_FLOAT
        match format {
            GL_RED => 4,   // GL_RED: 1 channel
            GL_RG => 8,    // GL_RG: 2 channels
            GL_RGBA => 16, // GL_RGBA: 4 channels
            _ => 4,        // Default
        }
    } else {
        // GL_UNSIGNED_BYTE or other
        match format {
            GL_RGBA => 4, // GL_RGBA: 4 bytes
            _ => 4,
        }
    };

    // Verify output buffer size
    let expected_size = (width as u64)
        .saturating_mul(height as u64)
        .saturating_mul(bytes_per_pixel as u64);
    if (dest_len as u64) < expected_size {
        set_last_error("output buffer size too small");
        return ERR_INVALID_ARGS;
    }

    // Read pixels and write to destination
    let dest_slice =
        unsafe { std::slice::from_raw_parts_mut(dest_ptr as *mut u8, dest_len as usize) };

    let src_buffer = match ctx_obj.kernel.get_buffer(src_handle) {
        Some(b) => b,
        None => {
            set_last_error("source buffer not found in kernel");
            return ERR_INVALID_OPERATION;
        }
    };

    let dst_format = if type_ == GL_FLOAT {
        match format {
            GL_RED => wgpu_types::TextureFormat::R32Float,
            GL_RG => wgpu_types::TextureFormat::Rg32Float,
            GL_RGBA => wgpu_types::TextureFormat::Rgba32Float,
            _ => wgpu_types::TextureFormat::Rgba32Float,
        }
    } else {
        wgpu_types::TextureFormat::Rgba8Unorm
    };

    crate::wasm_gl_emu::imaging::TransferEngine::read_pixels(
        &crate::wasm_gl_emu::imaging::TransferRequest {
            src_buffer,
            dst_format,
            dst_layout: crate::wasm_gl_emu::device::StorageLayout::Linear,
            x,
            y,
            width,
            height,
        },
        dest_slice,
    );

    ERR_OK
}
