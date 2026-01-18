use super::registry::{clear_last_error, get_registry, set_last_error};
use super::types::*;
use crate::wasm_gl_emu::rasterizer::{
    RasterPipeline, RenderState, ShaderMemoryLayout, VertexFetcher,
};

fn ctx_get_program_flat_varyings_mask(ctx: &Context) -> u64 {
    if let Some(program_id) = ctx.current_program {
        if let Some(program) = ctx.programs.get(&program_id) {
            if let Some(ref fs_module) = program.fs_module {
                return crate::wasm_gl_emu::rasterizer::RasterPipeline::compute_flat_varyings_mask(
                    fs_module,
                );
            }
        }
    }
    0
}

struct WebGLVertexFetcher {
    bindings: Vec<crate::wasm_gl_emu::transfer::AttributeBinding>,
}

impl VertexFetcher for WebGLVertexFetcher {
    fn fetch(
        &self,
        _kernel: &crate::wasm_gl_emu::GpuKernel,
        vertex_index: u32,
        instance_index: u32,
        dest: &mut [u8],
    ) {
        let mut attr_data = [0u32; 64]; // 16 locations * 4 components
        crate::wasm_gl_emu::transfer::TransferEngine::fetch_vertex_batch(
            &self.bindings,
            vertex_index,
            instance_index,
            &mut attr_data,
        );

        // Copy to dest with correct layout
        dest.fill(0);
        for (loc, chunk) in attr_data.chunks(4).enumerate() {
            let (dest_offset, _) = crate::naga_wasm_backend::output_layout::compute_input_offset(
                loc as u32,
                naga::ShaderStage::Vertex,
            );
            let off = dest_offset as usize;
            if off + 16 <= dest.len() {
                unsafe {
                    let ptr = chunk.as_ptr() as *const u8;
                    dest[off..off + 16].copy_from_slice(std::slice::from_raw_parts(ptr, 16));
                }
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
    let mask = ctx_get_program_flat_varyings_mask(ctx_obj);
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
        scissor: ctx_obj.scissor_box,
        scissor_enabled: ctx_obj.scissor_test_enabled,
        uniform_data: &ctx_obj.uniform_data,
        prepare_textures: None,
        blend: ctx_obj.blend_state,
        color_mask: ctx_obj.color_mask,
        depth: ctx_obj.depth_state,
        stencil: ctx_obj.stencil_state,
    };

    let fetcher = WebGLVertexFetcher {
        bindings: ctx_obj.get_attribute_bindings(),
    };

    // Calculate indices for draw_arrays (just 0..count)
    // But draw takes indices: Option<&[u32]>.
    // If we pass None, it iterates 0..vertex_count.
    // But we have 'first' parameter.
    // We need to handle 'first'.
    // Rasterizer::draw iterates 0..vertex_count if indices is None.
    // It passes i as vertex_id.
    // If we want start from 'first', we should probably pass indices.

    let (target_handle, target_w, target_h, target_fmt) = ctx_obj.get_color_attachment_info(false);

    ctx_obj.rasterizer.draw(
        &mut ctx_obj.kernel,
        crate::wasm_gl_emu::rasterizer::DrawConfig {
            color_target: crate::wasm_gl_emu::rasterizer::ColorTarget::Handle(target_handle),
            width: target_w,
            height: target_h,
            internal_format: target_fmt,
            depth: &mut ctx_obj.default_framebuffer.depth,
            stencil: &mut ctx_obj.default_framebuffer.stencil,
            pipeline: &pipeline,
            state: &state,
            vertex_fetcher: &fetcher,
            vertex_count: count as usize,
            instance_count: instance_count as usize,
            first_vertex: first as usize,
            first_instance: 0,
            indices: None,
            mode,
        },
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

    let itype = match type_ {
        GL_UNSIGNED_BYTE => crate::wasm_gl_emu::IndexType::U8,
        GL_UNSIGNED_SHORT => crate::wasm_gl_emu::IndexType::U16,
        GL_UNSIGNED_INT => crate::wasm_gl_emu::IndexType::U32,
        _ => return ERR_INVALID_ENUM,
    };

    let (vx, vy, vw, vh) = ctx_obj.viewport;

    // Create pipeline configuration
    let pipeline = RasterPipeline {
        flat_varyings_mask: ctx_get_program_flat_varyings_mask(ctx_obj),
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
        scissor: ctx_obj.scissor_box,
        scissor_enabled: ctx_obj.scissor_test_enabled,
        uniform_data: &ctx_obj.uniform_data,
        prepare_textures: None,
        blend: ctx_obj.blend_state,
        color_mask: ctx_obj.color_mask,
        depth: ctx_obj.depth_state,
        stencil: ctx_obj.stencil_state,
    };

    let fetcher = WebGLVertexFetcher {
        bindings: ctx_obj.get_attribute_bindings(),
    };

    let (target_handle, target_w, target_h, target_fmt) = ctx_obj.get_color_attachment_info(false);

    // Prepare indices lazily if EBO is bound
    let lazy_indices = ebo_handle
        .and_then(|h| ctx_obj.buffers.get(&h))
        .and_then(|buf| ctx_obj.kernel.get_buffer(buf.gpu_handle))
        .map(|gpu_buf| crate::wasm_gl_emu::transfer::LazyIndexBuffer {
            src_ptr: gpu_buf.data.as_ptr(),
            src_len: gpu_buf.data.len(),
            index_type: itype,
            offset,
            count: count as u32,
        });

    ctx_obj.rasterizer.draw(
        &mut ctx_obj.kernel,
        crate::wasm_gl_emu::rasterizer::DrawConfig {
            color_target: crate::wasm_gl_emu::rasterizer::ColorTarget::Handle(target_handle),
            width: target_w,
            height: target_h,
            internal_format: target_fmt,
            depth: &mut ctx_obj.default_framebuffer.depth,
            stencil: &mut ctx_obj.default_framebuffer.stencil,
            pipeline: &pipeline,
            state: &state,
            vertex_fetcher: &fetcher,
            vertex_count: count as usize,
            instance_count: instance_count as usize,
            first_vertex: 0,
            first_instance: 0,
            indices: lazy_indices
                .as_ref()
                .map(|l| l as &dyn crate::wasm_gl_emu::rasterizer::IndexBuffer),
            mode,
        },
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
    let (src_handle, src_width, src_height, _src_format) = ctx_obj.get_color_attachment_info(true);

    if !src_handle.is_valid() {
        set_last_error("no color attachment to read from");
        return ERR_INVALID_OPERATION;
    }

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

    crate::wasm_gl_emu::TransferEngine::read_pixels(
        &crate::wasm_gl_emu::TransferRequest {
            src_buffer,
            dst_format,
            dst_layout: crate::wasm_gl_emu::StorageLayout::Linear,
            x,
            y,
            width,
            height,
        },
        dest_slice,
    );

    ERR_OK
}
