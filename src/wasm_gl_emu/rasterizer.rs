//! Triangle rasterizer - shared between WebGL2 and WebGPU
//!
//! This module provides a driver-agnostic software rasterizer that can be used
//! by both WebGL2 and WebGPU implementations. It handles vertex fetching,
//! barycentric interpolation, and fragment shading.

use crate::wasm_gl_emu::device::{GpuHandle, GpuKernel, StorageLayout};
use crate::wasm_gl_emu::framebuffer::ColorAttachment;
use crate::webgl2_context::types::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WasmVec4(pub f32, pub f32, pub f32, pub f32);

#[cfg(target_arch = "wasm32")]
use crate::wasm_sync_turbo_globals;

type VsEntryFn = extern "C" fn(i32, i32, i32);
type FsEntryFn = extern "C" fn(i32, i32);

pub type PrepareTexturesFn<'a> = dyn Fn(&ShaderMemoryLayout) + 'a;

/// Vertex data after vertex shader execution
#[derive(Clone)]
pub struct ProcessedVertex {
    /// Clip-space position [x, y, z, w]
    pub position: [f32; 4],
    /// Varying data (stored as raw u32 bits to avoid NaN canonicalization)
    /// For float varyings, these are the bit patterns of f32 values
    /// For integer varyings, these are the actual integer values
    pub varyings: Vec<u32>,
}

/// Memory pointers for shader execution
/// This replaces hardcoded memory offsets with flexible pointers
#[derive(Debug, Clone, Copy, Default)]
pub struct ShaderMemoryLayout {
    /// Pointer to attribute data (vertex shader input)
    pub attr_ptr: u32,
    /// Pointer to uniform data
    pub uniform_ptr: u32,
    /// Pointer to varying data (VS output / FS input)
    pub varying_ptr: u32,
    /// Pointer to private/local shader data
    pub private_ptr: u32,
    /// Pointer to texture metadata
    pub texture_ptr: u32,
    /// Pointer to frame stack
    pub frame_sp: u32,
}

impl ShaderMemoryLayout {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Render state for a draw call
pub struct RenderState<'a> {
    /// Context handle
    pub ctx_handle: u32,
    /// Memory layout for shaders
    pub memory: ShaderMemoryLayout,
    /// Viewport (x, y, width, height)
    pub viewport: (i32, i32, u32, u32),
    /// Scissor (x, y, width, height)
    pub scissor: (i32, i32, u32, u32),
    /// Whether scissor test is enabled
    pub scissor_enabled: bool,
    /// Uniform data buffer
    pub uniform_data: &'a [u8],
    /// Texture metadata preparation callback
    pub prepare_textures: Option<Box<PrepareTexturesFn<'a>>>,
    /// Blend state
    pub blend: BlendState,
    /// Color mask
    pub color_mask: ColorMaskState,
    /// Depth state
    pub depth: DepthState,
    /// Stencil state
    pub stencil: StencilState,
    /// Face culling enabled
    pub cull_face_enabled: bool,
    /// Cull mode (GL_FRONT, GL_BACK, GL_FRONT_AND_BACK)
    pub cull_face_mode: u32,
    /// Front face winding (GL_CW, GL_CCW)
    pub front_face: u32,
}

/// Interface for fetching vertex attributes
pub trait VertexFetcher {
    /// Fetch attributes for a specific vertex and instance
    /// Writes data directly to the destination buffer (which maps to attr_ptr)
    fn fetch(&self, kernel: &GpuKernel, vertex_index: u32, instance_index: u32, dest: &mut [u8]);
}

/// Color target for rasterization
pub enum ColorTarget<'a> {
    /// Target is a buffer managed by the kernel
    Handle(GpuHandle),
    /// Target is a raw byte slice (used by WebGPU backend)
    Raw(&'a mut [u8]),
}

/// Depth/Stencil target for rasterization
pub enum DepthStencilTarget<'a> {
    /// Target is a buffer managed by the kernel
    Handle(GpuHandle),
    /// Target is a raw F32/U8 slice (used by default framebuffer)
    Raw {
        depth: &'a mut [f32],
        stencil: &'a mut [u8],
    },
}

/// Pipeline configuration for rasterization
/// Decouples from WebGL's Program object to support WebGPU
pub struct VaryingDebug {
    pub name: String,
    pub location: u32,
    pub type_code: u8,   // 0=float, 1=int, 2=uint
    pub components: u32, // number of scalar components
}

/// Color mask state
#[derive(Clone, Copy, Debug)]
pub struct ColorMaskState {
    pub r: bool,
    pub g: bool,
    pub b: bool,
    pub a: bool,
}

impl Default for ColorMaskState {
    fn default() -> Self {
        Self {
            r: true,
            g: true,
            b: true,
            a: true,
        }
    }
}

/// Stencil state for one face (front/back)
#[derive(Clone, Copy, Debug)]
pub struct StencilFaceState {
    pub func: u32,       // Default GL_ALWAYS
    pub ref_val: i32,    // Default 0
    pub mask: u32,       // Default all 1s
    pub fail: u32,       // Default GL_KEEP
    pub zfail: u32,      // Default GL_KEEP
    pub zpass: u32,      // Default GL_KEEP
    pub write_mask: u32, // Default all 1s
}

impl Default for StencilFaceState {
    fn default() -> Self {
        Self {
            func: GL_ALWAYS,
            ref_val: 0,
            mask: 0xFFFFFFFF,
            fail: GL_KEEP,
            zfail: GL_KEEP,
            zpass: GL_KEEP,
            write_mask: 0xFFFFFFFF,
        }
    }
}

/// Stencil test state
#[derive(Clone, Copy, Debug, Default)]
pub struct StencilState {
    pub enabled: bool,
    pub front: StencilFaceState,
    pub back: StencilFaceState,
}

/// Depth test state
#[derive(Clone, Copy, Debug)]
pub struct DepthState {
    pub enabled: bool,
    pub func: u32,  // GL_LESS
    pub mask: bool, // true
}

impl Default for DepthState {
    fn default() -> Self {
        Self {
            enabled: false,
            func: GL_LESS,
            mask: true,
        }
    }
}

/// Blend state for rasterization
#[derive(Clone, Copy, Debug)]
pub struct BlendState {
    pub enabled: bool,
    pub src_rgb: u32,
    pub dst_rgb: u32,
    pub src_alpha: u32,
    pub dst_alpha: u32,
    pub eq_rgb: u32,
    pub eq_alpha: u32,
    pub color: [f32; 4],
}

impl Default for BlendState {
    fn default() -> Self {
        Self {
            enabled: false,
            src_rgb: GL_ONE,
            dst_rgb: GL_ZERO,
            src_alpha: GL_ONE,
            dst_alpha: GL_ZERO,
            eq_rgb: GL_FUNC_ADD,
            eq_alpha: GL_FUNC_ADD,
            color: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

pub struct RasterPipeline {
    /// Shader function table indices or identifiers
    pub vertex_shader_type: u32,
    pub fragment_shader_type: u32,
    /// Memory layout for this pipeline
    pub memory: ShaderMemoryLayout,
    /// Bitmask of flat varyings (1 = flat, 0 = smooth)
    pub flat_varyings_mask: u64,
    /// Function table index for vertex shader (if available)
    pub vs_table_idx: Option<u32>,
    /// Function table index for fragment shader (if available)
    pub fs_table_idx: Option<u32>,
}

impl RasterPipeline {
    /// Computes the flat varyings mask from a fragment shader Naga module
    pub fn compute_flat_varyings_mask(fs_module: &naga::Module) -> u64 {
        let mut mask = 0u64;
        for ep in &fs_module.entry_points {
            if ep.stage == naga::ShaderStage::Fragment {
                for arg in &ep.function.arguments {
                    let mut make_flat = false;
                    let mut location = None;

                    if let Some(naga::Binding::Location {
                        location: loc,
                        interpolation,
                        ..
                    }) = &arg.binding
                    {
                        location = Some(*loc);
                        if let Some(interp) = interpolation {
                            if *interp == naga::Interpolation::Flat {
                                make_flat = true;
                            }
                        }
                    }

                    if !make_flat {
                        let ty = &fs_module.types[arg.ty];
                        match &ty.inner {
                            naga::TypeInner::Scalar(scalar) => {
                                if scalar.kind == naga::ScalarKind::Sint
                                    || scalar.kind == naga::ScalarKind::Uint
                                {
                                    make_flat = true;
                                }
                            }
                            naga::TypeInner::Vector { scalar, .. } => {
                                if scalar.kind == naga::ScalarKind::Sint
                                    || scalar.kind == naga::ScalarKind::Uint
                                {
                                    make_flat = true;
                                }
                            }
                            _ => {}
                        }
                    }

                    if make_flat {
                        if let Some(loc) = location {
                            let ty = &fs_module.types[arg.ty];
                            let components = match &ty.inner {
                                naga::TypeInner::Scalar(_) => 1,
                                naga::TypeInner::Vector { size, .. } => match size {
                                    naga::VectorSize::Bi => 2,
                                    naga::VectorSize::Tri => 3,
                                    naga::VectorSize::Quad => 4,
                                },
                                _ => 1,
                            };

                            let start_bit = (loc + 2) * 4;
                            for i in 0..components {
                                let bit = start_bit + i;
                                if bit < 64 {
                                    mask |= 1u64 << bit;
                                }
                            }
                        }
                    }
                }
            }
        }
        mask
    }
}

impl Default for RasterPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl RasterPipeline {
    pub fn new() -> Self {
        Self {
            vertex_shader_type: GL_VERTEX_SHADER,
            fragment_shader_type: GL_FRAGMENT_SHADER,
            memory: ShaderMemoryLayout::new(),
            flat_varyings_mask: 0,
            vs_table_idx: None,
            fs_table_idx: None,
        }
    }
}

fn get_factor(
    factor: u32,
    src: [f32; 4],
    dst: [f32; 4],
    constant: [f32; 4],
    alpha_sat: f32,
) -> [f32; 4] {
    match factor {
        GL_ZERO => [0.0, 0.0, 0.0, 0.0], // ZERO
        GL_ONE => [1.0, 1.0, 1.0, 1.0],  // ONE
        GL_SRC_COLOR => src,             // SRC_COLOR
        GL_ONE_MINUS_SRC_COLOR => [1.0 - src[0], 1.0 - src[1], 1.0 - src[2], 1.0 - src[3]], // ONE_MINUS_SRC_COLOR
        GL_SRC_ALPHA => [src[3], src[3], src[3], src[3]], // SRC_ALPHA
        GL_ONE_MINUS_SRC_ALPHA => [1.0 - src[3], 1.0 - src[3], 1.0 - src[3], 1.0 - src[3]], // ONE_MINUS_SRC_ALPHA
        GL_DST_ALPHA => [dst[3], dst[3], dst[3], dst[3]], // DST_ALPHA
        GL_ONE_MINUS_DST_ALPHA => [1.0 - dst[3], 1.0 - dst[3], 1.0 - dst[3], 1.0 - dst[3]], // ONE_MINUS_DST_ALPHA
        GL_DST_COLOR => dst, // DST_COLOR
        GL_ONE_MINUS_DST_COLOR => [1.0 - dst[0], 1.0 - dst[1], 1.0 - dst[2], 1.0 - dst[3]], // ONE_MINUS_DST_COLOR
        GL_SRC_ALPHA_SATURATE => [alpha_sat, alpha_sat, alpha_sat, 1.0], // SRC_ALPHA_SATURATE
        GL_CONSTANT_COLOR => constant,                                   // CONSTANT_COLOR
        GL_ONE_MINUS_CONSTANT_COLOR => [
            1.0 - constant[0],
            1.0 - constant[1],
            1.0 - constant[2],
            1.0 - constant[3],
        ], // ONE_MINUS_CONSTANT_COLOR
        GL_CONSTANT_ALPHA => [constant[3], constant[3], constant[3], constant[3]], // CONSTANT_ALPHA
        GL_ONE_MINUS_CONSTANT_ALPHA => [
            1.0 - constant[3],
            1.0 - constant[3],
            1.0 - constant[3],
            1.0 - constant[3],
        ], // ONE_MINUS_CONSTANT_ALPHA
        _ => [0.0, 0.0, 0.0, 0.0],
    }
}

fn blend_channel(src: f32, dst: f32, s_factor: f32, d_factor: f32, eq: u32) -> f32 {
    match eq {
        GL_FUNC_ADD => src * s_factor + dst * d_factor, // FUNC_ADD
        GL_FUNC_SUBTRACT => src * s_factor - dst * d_factor, // FUNC_SUBTRACT
        GL_FUNC_REVERSE_SUBTRACT => dst * d_factor - src * s_factor, // FUNC_REVERSE_SUBTRACT
        GL_MIN => src.min(dst),                         // MIN
        GL_MAX => src.max(dst),                         // MAX
        _ => src,
    }
}

fn blend_pixel(src: [u8; 4], dst: [u8; 4], state: &BlendState) -> [u8; 4] {
    if !state.enabled {
        return src;
    }

    let src_f = [
        src[0] as f32 / 255.0,
        src[1] as f32 / 255.0,
        src[2] as f32 / 255.0,
        src[3] as f32 / 255.0,
    ];
    let dst_f = [
        dst[0] as f32 / 255.0,
        dst[1] as f32 / 255.0,
        dst[2] as f32 / 255.0,
        dst[3] as f32 / 255.0,
    ];

    let alpha_sat = src_f[3].min(1.0 - dst_f[3]);
    let s_factor_rgb = get_factor(state.src_rgb, src_f, dst_f, state.color, alpha_sat);
    let d_factor_rgb = get_factor(state.dst_rgb, src_f, dst_f, state.color, alpha_sat);
    let s_factor_a = get_factor(state.src_alpha, src_f, dst_f, state.color, alpha_sat);
    let d_factor_a = get_factor(state.dst_alpha, src_f, dst_f, state.color, alpha_sat);

    let r = blend_channel(
        src_f[0],
        dst_f[0],
        s_factor_rgb[0],
        d_factor_rgb[0],
        state.eq_rgb,
    );
    let g = blend_channel(
        src_f[1],
        dst_f[1],
        s_factor_rgb[1],
        d_factor_rgb[1],
        state.eq_rgb,
    );
    let b = blend_channel(
        src_f[2],
        dst_f[2],
        s_factor_rgb[2],
        d_factor_rgb[2],
        state.eq_rgb,
    );
    let a = blend_channel(
        src_f[3],
        dst_f[3],
        s_factor_a[3],
        d_factor_a[3],
        state.eq_alpha,
    );

    [
        (r.clamp(0.0, 1.0) * 255.0) as u8,
        (g.clamp(0.0, 1.0) * 255.0) as u8,
        (b.clamp(0.0, 1.0) * 255.0) as u8,
        (a.clamp(0.0, 1.0) * 255.0) as u8,
    ]
}

fn blend_pixel_f32(src: [f32; 4], dst: [f32; 4], state: &BlendState) -> [f32; 4] {
    if !state.enabled {
        return src;
    }

    let alpha_sat = src[3].min(1.0 - dst[3]);
    let s_factor_rgb = get_factor(state.src_rgb, src, dst, state.color, alpha_sat);
    let d_factor_rgb = get_factor(state.dst_rgb, src, dst, state.color, alpha_sat);
    let s_factor_a = get_factor(state.src_alpha, src, dst, state.color, alpha_sat);
    let d_factor_a = get_factor(state.dst_alpha, src, dst, state.color, alpha_sat);

    [
        blend_channel(
            src[0],
            dst[0],
            s_factor_rgb[0],
            d_factor_rgb[0],
            state.eq_rgb,
        ),
        blend_channel(
            src[1],
            dst[1],
            s_factor_rgb[1],
            d_factor_rgb[1],
            state.eq_rgb,
        ),
        blend_channel(
            src[2],
            dst[2],
            s_factor_rgb[2],
            d_factor_rgb[2],
            state.eq_rgb,
        ),
        blend_channel(src[3], dst[3], s_factor_a[3], d_factor_a[3], state.eq_alpha),
    ]
}

/// Software triangle rasterizer
pub struct Rasterizer {}

impl Rasterizer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Rasterizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Interface for fetching indices
pub trait IndexBuffer {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn get(&self, i: usize) -> u32;
}

impl IndexBuffer for [u32] {
    fn len(&self) -> usize {
        self.len()
    }
    fn get(&self, i: usize) -> u32 {
        self[i]
    }
}

impl IndexBuffer for Vec<u32> {
    fn len(&self) -> usize {
        self.len()
    }
    fn get(&self, i: usize) -> u32 {
        self[i]
    }
}

pub struct DrawConfig<'a> {
    pub color_targets: Vec<ColorTarget<'a>>,
    pub depth_stencil_target: DepthStencilTarget<'a>,
    pub width: u32,
    pub height: u32,
    pub internal_formats: Vec<u32>,
    pub pipeline: &'a RasterPipeline,
    pub state: &'a RenderState<'a>,
    pub vertex_fetcher: &'a dyn VertexFetcher,
    pub vertex_count: usize,
    pub instance_count: usize,
    pub first_vertex: usize,
    pub first_instance: usize,
    pub indices: Option<&'a dyn IndexBuffer>,
    pub mode: u32,
}

impl Rasterizer {
    /// Draw a single point to the framebuffer
    pub fn draw_point(
        &self,
        fb: &mut crate::wasm_gl_emu::Framebuffer,
        x: f32,
        y: f32,
        colors: &[Vec<u8>],
        state: &RenderState,
    ) {
        let ix = x as i32;
        let iy = y as i32;
        if ix >= 0 && ix < fb.width as i32 && iy >= 0 && iy < fb.height as i32 {
            let width = fb.width;
            let height = fb.height;
            let layout = fb.layout;

            for (i, attachment) in fb.color_attachments.iter_mut().enumerate() {
                if let Some(att) = attachment {
                    let color = &colors[i];
                    let color_idx = crate::wasm_gl_emu::Framebuffer::get_pixel_offset_params(
                        ix as u32,
                        iy as u32,
                        0,
                        att.internal_format,
                        width,
                        height,
                        layout,
                    );
                    if color_idx + color.len() <= att.data.len() {
                        if att.internal_format == GL_RGBA8 {
                            let existing = [
                                att.data[color_idx],
                                att.data[color_idx + 1],
                                att.data[color_idx + 2],
                                att.data[color_idx + 3],
                            ];
                            let color_arr = [color[0], color[1], color[2], color[3]];
                            let blended = blend_pixel(color_arr, existing, &state.blend);

                            if state.color_mask.r {
                                att.data[color_idx] = blended[0];
                            }
                            if state.color_mask.g {
                                att.data[color_idx + 1] = blended[1];
                            }
                            if state.color_mask.b {
                                att.data[color_idx + 2] = blended[2];
                            }
                            if state.color_mask.a {
                                att.data[color_idx + 3] = blended[3];
                            }
                        } else {
                            att.data[color_idx..color_idx + color.len()].copy_from_slice(color);
                        }
                    }
                }
            }
        }
    }

    /// Draw a triangle to the framebuffer (simple, no interpolation)
    pub fn draw_triangle(
        &self,
        fb: &mut crate::wasm_gl_emu::Framebuffer,
        p0: (f32, f32),
        p1: (f32, f32),
        p2: (f32, f32),
        colors: &[Vec<u8>],
    ) {
        let min_x = p0.0.min(p1.0).min(p2.0).max(0.0).floor() as i32;
        let max_x = p0.0.max(p1.0).max(p2.0).min(fb.width as f32 - 1.0).ceil() as i32;
        let min_y = p0.1.min(p1.1).min(p2.1).max(0.0).floor() as i32;
        let max_y = p0.1.max(p1.1).max(p2.1).min(fb.height as f32 - 1.0).ceil() as i32;

        let fb_w = fb.width;
        let fb_h = fb.height;
        let fb_layout = fb.layout;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;

                if is_inside(px, py, p0, p1, p2) {
                    for (i, attachment) in fb.color_attachments.iter_mut().enumerate() {
                        if let Some(att) = attachment {
                            let color = &colors[i];
                            let idx = crate::wasm_gl_emu::Framebuffer::get_pixel_offset_params(
                                x as u32,
                                y as u32,
                                0,
                                att.internal_format,
                                fb_w,
                                fb_h,
                                fb_layout,
                            );
                            if idx + color.len() <= att.data.len() {
                                att.data[idx..idx + color.len()].copy_from_slice(color);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Rasterize a triangle with perspective-correct interpolation
    /// This is the core rasterization function extracted from drawing.rs
    #[allow(clippy::too_many_arguments)]
    pub fn rasterize_triangle(
        &self,
        fb: &mut crate::wasm_gl_emu::Framebuffer,
        v0: &ProcessedVertex,
        v1: &ProcessedVertex,
        v2: &ProcessedVertex,
        pipeline: &RasterPipeline,
        state: &RenderState,
        internal_formats: &[u32],
    ) {
        let (vx, vy, vw, vh) = state.viewport;

        // Scissor limit
        let (mut limit_x0, mut limit_y0, mut limit_x1, mut limit_y1) =
            (0, 0, fb.width as i32, fb.height as i32);

        if state.scissor_enabled {
            let (sx, sy, sw, sh) = state.scissor;
            limit_x0 = limit_x0.max(sx);
            limit_y0 = limit_y0.max(sy);
            limit_x1 = limit_x1.min(sx + sw as i32);
            limit_y1 = limit_y1.min(sy + sh as i32);
        }

        // Screen coordinates (with perspective divide)
        let p0 = screen_position(&v0.position, vx, vy, vw, vh);
        let p1 = screen_position(&v1.position, vx, vy, vw, vh);
        let p2 = screen_position(&v2.position, vx, vy, vw, vh);

        // Bounding box
        let min_x = (p0.0.min(p1.0).min(p2.0).floor() as i32).max(limit_x0);
        let max_x = (p0.0.max(p1.0).max(p2.0).ceil() as i32).min(limit_x1 - 1);
        let min_y = (p0.1.min(p1.1).min(p2.1).floor() as i32).max(limit_y0);
        let max_y = (p0.1.max(p1.1).max(p2.1).ceil() as i32).min(limit_y1 - 1);

        if max_x < min_x || max_y < min_y {
            return;
        }

        // Calculate triangle area to determine facing
        let tri_area = (p1.0 - p0.0) * (p2.1 - p0.1) - (p1.1 - p0.1) * (p2.0 - p0.0);

        // Standard OpenGL: CCW is front by default.
        // With Y-up in screen space, CCW area sign is positive.
        let is_ccw = tri_area > 0.0;
        let is_front = if state.front_face == crate::webgl2_context::types::GL_CCW {
            is_ccw
        } else {
            !is_ccw
        };

        if state.cull_face_enabled {
            let should_cull = if state.cull_face_mode == crate::webgl2_context::types::GL_FRONT {
                is_front
            } else if state.cull_face_mode == crate::webgl2_context::types::GL_BACK {
                !is_front
            } else {
                state.cull_face_mode == crate::webgl2_context::types::GL_FRONT_AND_BACK
            };
            if should_cull {
                return;
            }
        }

        // Perspective correction factors
        let w0_inv = 1.0 / v0.position[3];
        let w1_inv = 1.0 / v1.position[3];
        let w2_inv = 1.0 / v2.position[3];

        // Pre-allocate varyings buffer to avoid allocation per pixel
        let varying_count = v0
            .varyings
            .len()
            .min(v1.varyings.len())
            .min(v2.varyings.len());

        let mut interp_varyings = vec![0u32; varying_count];

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let px = (x as f32) + 0.5;
                let py = (y as f32) + 0.5;
                let (u, v, w) = barycentric((px, py), p0, p1, p2);

                if u >= 0.0 && v >= 0.0 && w >= 0.0 {
                    let fb_idx = fb.get_pixel_index(x as u32, y as u32, 0);

                    // --- Stencil Test ---
                    if state.stencil.enabled && !fb.stencil.is_empty() {
                        let current_stencil = fb.stencil[fb_idx];
                        let face_state = if is_front {
                            &state.stencil.front
                        } else {
                            &state.stencil.back
                        };

                        // Compare
                        if !compare_stencil(
                            face_state.func,
                            face_state.ref_val,
                            current_stencil,
                            face_state.mask,
                        ) {
                            // Fail
                            let new_val = apply_stencil_op(
                                face_state.fail,
                                current_stencil,
                                face_state.ref_val,
                            );
                            fb.stencil[fb_idx] = (current_stencil & !face_state.write_mask as u8)
                                | (new_val & face_state.write_mask as u8);
                            continue;
                        }
                    }

                    // --- Depth Test ---
                    // Depth is interpolated linearly in screen space for GL
                    let z0 = v0.position[2] / v0.position[3];
                    let z1 = v1.position[2] / v1.position[3];
                    let z2 = v2.position[2] / v2.position[3];
                    let depth_ndc = u * z0 + v * z1 + w * z2;
                    let depth = (depth_ndc + 1.0) * 0.5;

                    // Check bounds [0,1]
                    if !(0.0..=1.0).contains(&depth) {
                        continue;
                    }

                    // Determine depth comparison result
                    let depth_pass = if state.depth.enabled && !fb.depth.is_empty() {
                        compare_depth(state.depth.func, depth, fb.depth[fb_idx])
                    } else {
                        // When depth test is disabled or no buffer exists, it always passes
                        true
                    };

                    // Handle Depth Fail / Pass for Stencil
                    if state.stencil.enabled && !fb.stencil.is_empty() {
                        let current_stencil = fb.stencil[fb_idx];
                        let face_state = if is_front {
                            &state.stencil.front
                        } else {
                            &state.stencil.back
                        };

                        if !depth_pass {
                            // ZFail
                            let new_val = apply_stencil_op(
                                face_state.zfail,
                                current_stencil,
                                face_state.ref_val,
                            );
                            fb.stencil[fb_idx] = (current_stencil & !face_state.write_mask as u8)
                                | (new_val & face_state.write_mask as u8);
                            continue; // Discard
                        } else {
                            // Pass (ZPass)
                            let new_val = apply_stencil_op(
                                face_state.zpass,
                                current_stencil,
                                face_state.ref_val,
                            );
                            fb.stencil[fb_idx] = (current_stencil & !face_state.write_mask as u8)
                                | (new_val & face_state.write_mask as u8);
                        }
                    } else if !depth_pass {
                        continue; // Discard if depth failed and no stencil updates needed
                    }

                    // --- Write Depth ---
                    // Write depth when mask is true AND depth test passed
                    if state.depth.mask && depth_pass && !fb.depth.is_empty() {
                        fb.depth[fb_idx] = depth;
                    }

                    // --- Fragment Shader & Color Write ---
                    // Perspective correct interpolation of varyings
                    let w_interp_inv = u * w0_inv + v * w1_inv + w * w2_inv;
                    let w_interp = 1.0 / w_interp_inv;

                    for (k, varying) in interp_varyings.iter_mut().enumerate() {
                        if (pipeline.flat_varyings_mask & (1u64 << k)) != 0 {
                            // Flat shading: copy raw bits from provoking vertex (v2)
                            *varying = v2.varyings[k];
                        } else {
                            // Smooth shading: interpolate as floats, then store as bits
                            let v0_f = f32::from_bits(v0.varyings[k]);
                            let v1_f = f32::from_bits(v1.varyings[k]);
                            let v2_f = f32::from_bits(v2.varyings[k]);
                            let interp_f =
                                (u * v0_f * w0_inv + v * v1_f * w1_inv + w * v2_f * w2_inv)
                                    * w_interp;
                            *varying = interp_f.to_bits();
                        }
                    }

                    // Execute fragment shader and get color
                    let colors = self.execute_fragment_shader(
                        &interp_varyings,
                        pipeline,
                        state,
                        internal_formats,
                    );

                    let fb_w = fb.width;
                    let fb_h = fb.height;
                    let fb_layout = fb.layout;

                    for (i, attachment) in fb.color_attachments.iter_mut().enumerate() {
                        if let Some(att) = attachment {
                            let color = &colors[i];
                            let color_idx =
                                crate::wasm_gl_emu::Framebuffer::get_pixel_offset_params(
                                    x as u32,
                                    y as u32,
                                    0,
                                    att.internal_format,
                                    fb_w,
                                    fb_h,
                                    fb_layout,
                                );

                            if color_idx + color.len() <= att.data.len() {
                                // For 32-bit formats, write directly with optional blending
                                if att.internal_format == GL_R32F
                                    || att.internal_format == GL_RG32F
                                    || att.internal_format == GL_RGBA32F
                                    || att.internal_format == GL_R32UI
                                    || att.internal_format == GL_RG32UI
                                    || att.internal_format == GL_RGBA32UI
                                    || att.internal_format == GL_R32I
                                    || att.internal_format == GL_RG32I
                                    || att.internal_format == GL_RGBA32I
                                    || att.internal_format == GL_R8UI
                                    || att.internal_format == GL_RG8UI
                                    || att.internal_format == GL_RGBA8UI
                                    || att.internal_format == GL_R8I
                                    || att.internal_format == GL_RG8I
                                    || att.internal_format == GL_RGBA8I
                                    || att.internal_format == GL_R16UI
                                    || att.internal_format == GL_RG16UI
                                    || att.internal_format == GL_RGBA16UI
                                    || att.internal_format == GL_R16I
                                    || att.internal_format == GL_RG16I
                                    || att.internal_format == GL_RGBA16I
                                {
                                    // GL_R32F, GL_RG32F, GL_RGBA32F
                                    if state.blend.enabled
                                        && (att.internal_format == GL_R32F
                                            || att.internal_format == GL_RG32F
                                            || att.internal_format == GL_RGBA32F)
                                    {
                                        let existing: [f32; 4] = match att.internal_format {
                                            GL_R32F => {
                                                let v = f32::from_ne_bytes(
                                                    att.data[color_idx..color_idx + 4]
                                                        .try_into()
                                                        .unwrap(),
                                                );
                                                [v, 0.0, 0.0, 1.0]
                                            }
                                            GL_RG32F => {
                                                let v0 = f32::from_ne_bytes(
                                                    att.data[color_idx..color_idx + 4]
                                                        .try_into()
                                                        .unwrap(),
                                                );
                                                let v1 = f32::from_ne_bytes(
                                                    att.data[color_idx + 4..color_idx + 8]
                                                        .try_into()
                                                        .unwrap(),
                                                );
                                                [v0, v1, 0.0, 1.0]
                                            }
                                            GL_RGBA32F => {
                                                let v0 = f32::from_ne_bytes(
                                                    att.data[color_idx..color_idx + 4]
                                                        .try_into()
                                                        .unwrap(),
                                                );
                                                let v1 = f32::from_ne_bytes(
                                                    att.data[color_idx + 4..color_idx + 8]
                                                        .try_into()
                                                        .unwrap(),
                                                );
                                                let v2 = f32::from_ne_bytes(
                                                    att.data[color_idx + 8..color_idx + 12]
                                                        .try_into()
                                                        .unwrap(),
                                                );
                                                let v3 = f32::from_ne_bytes(
                                                    att.data[color_idx + 12..color_idx + 16]
                                                        .try_into()
                                                        .unwrap(),
                                                );
                                                [v0, v1, v2, v3]
                                            }
                                            _ => [0.0, 0.0, 0.0, 1.0],
                                        };

                                        let src_color: [f32; 4] = match att.internal_format {
                                            GL_R32F => [
                                                f32::from_ne_bytes(color[0..4].try_into().unwrap()),
                                                0.0,
                                                0.0,
                                                1.0,
                                            ],
                                            GL_RG32F => [
                                                f32::from_ne_bytes(color[0..4].try_into().unwrap()),
                                                f32::from_ne_bytes(color[4..8].try_into().unwrap()),
                                                0.0,
                                                1.0,
                                            ],
                                            GL_RGBA32F => [
                                                f32::from_ne_bytes(color[0..4].try_into().unwrap()),
                                                f32::from_ne_bytes(color[4..8].try_into().unwrap()),
                                                f32::from_ne_bytes(
                                                    color[8..12].try_into().unwrap(),
                                                ),
                                                f32::from_ne_bytes(
                                                    color[12..16].try_into().unwrap(),
                                                ),
                                            ],
                                            _ => [0.0, 0.0, 0.0, 1.0],
                                        };

                                        let blended =
                                            blend_pixel_f32(src_color, existing, &state.blend);

                                        // Write back blended
                                        match att.internal_format {
                                            GL_R32F => {
                                                if state.color_mask.r {
                                                    att.data[color_idx..color_idx + 4]
                                                        .copy_from_slice(&blended[0].to_ne_bytes());
                                                }
                                            }
                                            GL_RG32F => {
                                                if state.color_mask.r {
                                                    att.data[color_idx..color_idx + 4]
                                                        .copy_from_slice(&blended[0].to_ne_bytes());
                                                }
                                                if state.color_mask.g {
                                                    att.data[color_idx + 4..color_idx + 8]
                                                        .copy_from_slice(&blended[1].to_ne_bytes());
                                                }
                                            }
                                            GL_RGBA32F => {
                                                if state.color_mask.r {
                                                    att.data[color_idx..color_idx + 4]
                                                        .copy_from_slice(&blended[0].to_ne_bytes());
                                                }
                                                if state.color_mask.g {
                                                    att.data[color_idx + 4..color_idx + 8]
                                                        .copy_from_slice(&blended[1].to_ne_bytes());
                                                }
                                                if state.color_mask.b {
                                                    att.data[color_idx + 8..color_idx + 12]
                                                        .copy_from_slice(&blended[2].to_ne_bytes());
                                                }
                                                if state.color_mask.a {
                                                    att.data[color_idx + 12..color_idx + 16]
                                                        .copy_from_slice(&blended[3].to_ne_bytes());
                                                }
                                            }
                                            _ => {}
                                        }
                                    } else {
                                        // Clamp to mask if not blending
                                        match att.internal_format {
                                            GL_R32F | GL_R32UI | GL_R32I => {
                                                if state.color_mask.r {
                                                    att.data[color_idx..color_idx + 4]
                                                        .copy_from_slice(&color[0..4]);
                                                }
                                            }
                                            GL_RG32F | GL_RG32UI | GL_RG32I => {
                                                if state.color_mask.r {
                                                    att.data[color_idx..color_idx + 4]
                                                        .copy_from_slice(&color[0..4]);
                                                }
                                                if state.color_mask.g {
                                                    att.data[color_idx + 4..color_idx + 8]
                                                        .copy_from_slice(&color[4..8]);
                                                }
                                            }
                                            GL_RGBA32F | GL_RGBA32UI | GL_RGBA32I => {
                                                if state.color_mask.r {
                                                    att.data[color_idx..color_idx + 4]
                                                        .copy_from_slice(&color[0..4]);
                                                }
                                                if state.color_mask.g {
                                                    att.data[color_idx + 4..color_idx + 8]
                                                        .copy_from_slice(&color[4..8]);
                                                }
                                                if state.color_mask.b {
                                                    att.data[color_idx + 8..color_idx + 12]
                                                        .copy_from_slice(&color[8..12]);
                                                }
                                                if state.color_mask.a {
                                                    att.data[color_idx + 12..color_idx + 16]
                                                        .copy_from_slice(&color[12..16]);
                                                }
                                            }
                                            GL_R16UI | GL_R16I => {
                                                if state.color_mask.r {
                                                    att.data[color_idx..color_idx + 2]
                                                        .copy_from_slice(&color[0..2]);
                                                }
                                            }
                                            GL_RG16UI | GL_RG16I => {
                                                if state.color_mask.r {
                                                    att.data[color_idx..color_idx + 2]
                                                        .copy_from_slice(&color[0..2]);
                                                }
                                                if state.color_mask.g {
                                                    att.data[color_idx + 2..color_idx + 4]
                                                        .copy_from_slice(&color[2..4]);
                                                }
                                            }
                                            GL_RGBA16UI | GL_RGBA16I => {
                                                if state.color_mask.r {
                                                    att.data[color_idx..color_idx + 2]
                                                        .copy_from_slice(&color[0..2]);
                                                }
                                                if state.color_mask.g {
                                                    att.data[color_idx + 2..color_idx + 4]
                                                        .copy_from_slice(&color[2..4]);
                                                }
                                                if state.color_mask.b {
                                                    att.data[color_idx + 4..color_idx + 6]
                                                        .copy_from_slice(&color[4..6]);
                                                }
                                                if state.color_mask.a {
                                                    att.data[color_idx + 6..color_idx + 8]
                                                        .copy_from_slice(&color[6..8]);
                                                }
                                            }
                                            GL_R8UI | GL_R8I => {
                                                if state.color_mask.r {
                                                    att.data[color_idx] = color[0];
                                                }
                                            }
                                            GL_RG8UI | GL_RG8I => {
                                                if state.color_mask.r {
                                                    att.data[color_idx] = color[0];
                                                }
                                                if state.color_mask.g {
                                                    att.data[color_idx + 1] = color[1];
                                                }
                                            }
                                            GL_RGBA8UI | GL_RGBA8I => {
                                                if state.color_mask.r {
                                                    att.data[color_idx] = color[0];
                                                }
                                                if state.color_mask.g {
                                                    att.data[color_idx + 1] = color[1];
                                                }
                                                if state.color_mask.b {
                                                    att.data[color_idx + 2] = color[2];
                                                }
                                                if state.color_mask.a {
                                                    att.data[color_idx + 3] = color[3];
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                } else if att.internal_format == GL_RGBA8 {
                                    // GL_RGBA8: Use quantized blending
                                    let existing = [
                                        att.data[color_idx],
                                        att.data[color_idx + 1],
                                        att.data[color_idx + 2],
                                        att.data[color_idx + 3],
                                    ];
                                    let color_arr = [color[0], color[1], color[2], color[3]];
                                    let blended = blend_pixel(color_arr, existing, &state.blend);

                                    // Color Mask
                                    if state.color_mask.r {
                                        att.data[color_idx] = blended[0];
                                    }
                                    if state.color_mask.g {
                                        att.data[color_idx + 1] = blended[1];
                                    }
                                    if state.color_mask.b {
                                        att.data[color_idx + 2] = blended[2];
                                    }
                                    if state.color_mask.a {
                                        att.data[color_idx + 3] = blended[3];
                                    }
                                } else {
                                    // Float formats: Direct write (no blending yet)
                                    att.data[color_idx..color_idx + color.len()]
                                        .copy_from_slice(color);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Execute fragment shader and return color (format-aware)
    fn execute_fragment_shader(
        &self,
        varyings: &[u32],
        pipeline: &RasterPipeline,
        _state: &RenderState,
        target_formats: &[u32],
    ) -> Vec<Vec<u8>> {
        // Turbo FS: Direct call with register arguments (Tier 2) and results (Tier 3)
        let fs_table_idx = pipeline.fs_table_idx.unwrap_or(0);

        let varying_in_ptr = varyings.as_ptr() as i32;
        let private_ptr = pipeline.memory.private_ptr as i32;

        if cfg!(target_arch = "wasm32") && fs_table_idx > 0 {
            unsafe {
                crate::ACTIVE_UNIFORM_PTR = pipeline.memory.uniform_ptr;
                crate::ACTIVE_VARYING_PTR = varying_in_ptr as u32;
                crate::ACTIVE_PRIVATE_PTR = private_ptr as u32;
                crate::ACTIVE_TEXTURE_PTR = pipeline.memory.texture_ptr;
            }
            let fs_func: FsEntryFn = unsafe { core::mem::transmute(fs_table_idx as usize) };
            fs_func(varying_in_ptr, private_ptr);
        }

        let mut results = Vec::with_capacity(target_formats.len());

        for (i, &format) in target_formats.iter().enumerate() {
            // Read color from private memory (each location is 16 bytes)
            let mut color_bytes = [0u8; 16];
            unsafe {
                std::ptr::copy_nonoverlapping(
                    (pipeline.memory.private_ptr + (i as u32 * 16)) as *const u8,
                    color_bytes.as_mut_ptr(),
                    16,
                );
            }
            let c: [f32; 4] = unsafe { std::mem::transmute(color_bytes) };

            // Format-aware output
            let output = match format {
                GL_R32F | GL_R32UI | GL_R32I => {
                    // 1 channel × 4 bytes
                    color_bytes[0..4].to_vec()
                }
                GL_RG32F | GL_RG32UI | GL_RG32I => {
                    // 2 channels × 4 bytes
                    color_bytes[0..8].to_vec()
                }
                GL_RGBA32F | GL_RGBA32UI | GL_RGBA32I => {
                    // 4 channels × 4 bytes
                    color_bytes[0..16].to_vec()
                }
                GL_R16UI | GL_R16I => color_bytes[0..2].to_vec(),
                GL_RG16UI | GL_RG16I => {
                    let mut res = Vec::with_capacity(4);
                    res.extend_from_slice(&color_bytes[0..2]);
                    res.extend_from_slice(&color_bytes[4..6]);
                    res
                }
                GL_RGBA16UI | GL_RGBA16I => {
                    let mut res = Vec::with_capacity(8);
                    res.extend_from_slice(&color_bytes[0..2]);
                    res.extend_from_slice(&color_bytes[4..6]);
                    res.extend_from_slice(&color_bytes[8..10]);
                    res.extend_from_slice(&color_bytes[12..14]);
                    res
                }
                GL_R8UI | GL_R8I => {
                    vec![color_bytes[0]]
                }
                GL_RG8UI | GL_RG8I => {
                    vec![color_bytes[0], color_bytes[4]]
                }
                GL_RGBA8UI | GL_RGBA8I => {
                    vec![
                        color_bytes[0],
                        color_bytes[4],
                        color_bytes[8],
                        color_bytes[12],
                    ]
                }
                _ => {
                    // GL_RGBA8: Quantize to u8.
                    // Note: We use clamp/scale for Unorm formats only.
                    vec![
                        (c[0].clamp(0.0, 1.0) * 255.0) as u8,
                        (c[1].clamp(0.0, 1.0) * 255.0) as u8,
                        (c[2].clamp(0.0, 1.0) * 255.0) as u8,
                        (c[3].clamp(0.0, 1.0) * 255.0) as u8,
                    ]
                }
            };
            results.push(output);
        }

        results
    }

    /// Draw primitives
    pub fn draw(&self, kernel: &mut GpuKernel, mut config: DrawConfig) {
        let (_vx, _vy, _vw, _vh) = config.state.viewport;

        // Tier 1 "Turbo" ABI: Set up the shared shader environment ONCE per draw call.
        // These are imported by the shader WASM modules to avoid per-call setup.
        // Write to them here to communicate with the shader WASM modules.
        // For wasm targets the host provides mutable WebAssembly.Global objects
        // (turboGlobals). We synchronize them once per draw via a single import
        // call to avoid per-vertex JS overhead.
        unsafe {
            crate::ACTIVE_ATTR_PTR = config.state.memory.attr_ptr;
            crate::ACTIVE_UNIFORM_PTR = config.state.memory.uniform_ptr;
            crate::ACTIVE_VARYING_PTR = config.state.memory.varying_ptr;
            crate::ACTIVE_PRIVATE_PTR = config.state.memory.private_ptr;
            crate::ACTIVE_TEXTURE_PTR = config.state.memory.texture_ptr;
            crate::ACTIVE_FRAME_SP = config.state.memory.frame_sp;
        }

        #[cfg(target_arch = "wasm32")]
        unsafe {
            // Call into the host to sync the WebAssembly.Global objects
            wasm_sync_turbo_globals(
                config.state.memory.attr_ptr,
                config.state.memory.uniform_ptr,
                config.state.memory.varying_ptr,
                config.state.memory.private_ptr,
                config.state.memory.texture_ptr,
                config.state.memory.frame_sp,
            );
        }

        // 0. Preparation: Copy uniforms and fix up pointers ONCE per draw call.
        // Doing this inside the vertex loop causes cumulative additions to relative offsets.
        unsafe {
            // Copy uniforms
            let copy_len = config.state.uniform_data.len().min(16384); // TODO: magic number is not appropriate
            let uniform_ptr = config.state.memory.uniform_ptr;
            if config.state.uniform_data.as_ptr() as u32 != uniform_ptr {
                std::ptr::copy_nonoverlapping(
                    config.state.uniform_data.as_ptr(),
                    uniform_ptr as *mut u8,
                    copy_len,
                );
            }

            // Fix up context block pointers: add the base address to the relative offsets
            let ctx_block = uniform_ptr as *mut u32;
            for i in 0..64 {
                let current_val = *ctx_block.add(i);
                if current_val > 0 && current_val < 16384 {
                    // TODO: magic number is not appropriate
                    // It's a relative offset
                    *ctx_block.add(i) = current_val + uniform_ptr;
                }
            }

            // Prepare textures
            if let Some(ref prepare) = config.state.prepare_textures {
                prepare(&config.state.memory);
            }
        }

        for instance_id in 0..config.instance_count {
            let actual_instance_id = config.first_instance + instance_id;
            let mut vertices = Vec::with_capacity(config.vertex_count);

            // 1. Run Vertex Shader for all vertices
            let count = if let Some(idxs) = config.indices {
                idxs.len()
            } else {
                config.vertex_count
            };

            for i in 0..count {
                let vertex_id = if let Some(idxs) = config.indices {
                    idxs.get(i)
                } else {
                    (config.first_vertex + i) as u32
                };

                // Fetch attributes directly into the shader memory
                let attr_ptr = config.state.memory.attr_ptr;
                let attr_dest =
                    unsafe { std::slice::from_raw_parts_mut(attr_ptr as *mut u8, 1024) };
                config.vertex_fetcher.fetch(
                    kernel,
                    vertex_id,
                    actual_instance_id as u32,
                    attr_dest,
                );

                // Turbo VS: Direct call with register arguments (Tier 2)
                let vs_table_idx = config.pipeline.vs_table_idx.unwrap_or(0);

                // Pre-allocate varying buffer for this vertex (256 bytes = 64 u32s)
                let mut varyings_u32 = vec![0u32; 64];
                let varying_out_ptr = varyings_u32.as_mut_ptr() as i32;

                if cfg!(target_arch = "wasm32") && vs_table_idx > 0 {
                    unsafe {
                        crate::ACTIVE_VARYING_PTR = varying_out_ptr as u32;
                    }
                    let vs_func: VsEntryFn = unsafe { core::mem::transmute(vs_table_idx as usize) };
                    vs_func(vertex_id as i32, actual_instance_id as i32, varying_out_ptr);
                }

                // gl_Position is expected at the start of varyings (Varying location 0)
                let pos = [
                    f32::from_bits(varyings_u32[0]),
                    f32::from_bits(varyings_u32[1]),
                    f32::from_bits(varyings_u32[2]),
                    f32::from_bits(varyings_u32[3]),
                ];

                vertices.push(ProcessedVertex {
                    position: pos,
                    varyings: varyings_u32,
                });
            }

            let mut fb_attachments = Vec::with_capacity(config.color_targets.len());
            let fb_layout = if config.color_targets.is_empty() {
                StorageLayout::Linear
            } else {
                match &config.color_targets[0] {
                    ColorTarget::Handle(handle) => {
                        if handle.is_valid() {
                            kernel.get_buffer(*handle).expect("buffer lost").layout
                        } else {
                            StorageLayout::Linear
                        }
                    }
                    ColorTarget::Raw(_) => StorageLayout::Linear,
                };
                // Re-match because of borrow checker (we used kernel immutably above)
                match &config.color_targets[0] {
                    ColorTarget::Handle(handle) => {
                        if handle.is_valid() {
                            kernel.get_buffer(*handle).expect("buffer lost").layout
                        } else {
                            StorageLayout::Linear
                        }
                    }
                    ColorTarget::Raw(_) => StorageLayout::Linear,
                }
            };

            // Use unsafe to circumvent borrow checker for multi-target buffer access.
            // This is safe as long as the handles in config.color_targets are distinct.
            let kernel_raw = kernel as *mut GpuKernel;
            for (i, target) in config.color_targets.iter_mut().enumerate() {
                let attachment = match target {
                    ColorTarget::Handle(handle) => {
                        if !handle.is_valid() {
                            None
                        } else {
                            let color_buffer = unsafe {
                                (*kernel_raw)
                                    .get_buffer_mut(*handle)
                                    .expect("color buffer lost")
                            };
                            Some(ColorAttachment {
                                data: color_buffer.data.as_mut_slice(),
                                internal_format: config.internal_formats[i],
                            })
                        }
                    }
                    ColorTarget::Raw(data) => Some(ColorAttachment {
                        data,
                        internal_format: config.internal_formats[i],
                    }),
                };
                fb_attachments.push(attachment);
            }

            let (target_depth, target_stencil) = match &mut config.depth_stencil_target {
                DepthStencilTarget::Handle(handle) => {
                    let ds_buffer = kernel.get_buffer_mut(*handle).expect("ds buffer lost");
                    let width = config.width as usize;
                    let height = config.height as usize;
                    let data = &mut ds_buffer.data;

                    if data.len() >= width * height * 4 {
                        let d_slice = unsafe {
                            std::slice::from_raw_parts_mut(
                                data.as_mut_ptr() as *mut f32,
                                data.len() / 4, // TODO: suspicious magic number
                            )
                        };
                        let s_slice = if data.len() >= width * height * 5 {
                            // TODO: suspicious magic numbers
                            &mut data[width * height * 4..width * height * 5] // TODO: suspicious magic numbers
                        } else {
                            &mut []
                        };
                        (d_slice, s_slice)
                    } else {
                        (&mut [] as &mut [f32], &mut [] as &mut [u8])
                    }
                }
                DepthStencilTarget::Raw { depth, stencil } => {
                    (*depth as &mut [f32], *stencil as &mut [u8])
                }
            };

            let mut fb = crate::wasm_gl_emu::Framebuffer {
                width: config.width,
                height: config.height,
                color_attachments: fb_attachments,
                depth: target_depth,
                stencil: target_stencil,
                layout: fb_layout,
            };

            self.rasterize_all(
                &mut fb,
                &vertices,
                config.mode,
                config.pipeline,
                config.state,
                &config.internal_formats,
            );
        }
    }

    fn rasterize_all(
        &self,
        fb: &mut crate::wasm_gl_emu::Framebuffer,
        vertices: &[ProcessedVertex],
        mode: u32,
        pipeline: &RasterPipeline,
        state: &RenderState,
        internal_formats: &[u32],
    ) {
        if mode == GL_POINTS {
            // GL_POINTS
            for v in vertices {
                let (_vx, _vy, _vw, _vh) = state.viewport;
                let screen_x =
                    _vx as f32 + (v.position[0] / v.position[3] + 1.0) * 0.5 * _vw as f32;
                let screen_y =
                    _vy as f32 + (v.position[1] / v.position[3] + 1.0) * 0.5 * _vh as f32;

                // Run FS
                let colors =
                    self.execute_fragment_shader(&v.varyings, pipeline, state, internal_formats);
                self.draw_point(fb, screen_x, screen_y, &colors, state);
            }
        } else if mode == GL_TRIANGLES {
            // GL_TRIANGLES
            for i in (0..vertices.len()).step_by(3) {
                if i + 2 >= vertices.len() {
                    break;
                }
                let v0 = &vertices[i];
                let v1 = &vertices[i + 1];
                let v2 = &vertices[i + 2];

                self.rasterize_triangle(fb, v0, v1, v2, pipeline, state, internal_formats);
            }
        } else if mode == GL_TRIANGLE_STRIP {
            // GL_TRIANGLE_STRIP
            for i in 0..vertices.len().saturating_sub(2) {
                let (v0, v1, v2) = if i % 2 == 0 {
                    (&vertices[i], &vertices[i + 1], &vertices[i + 2])
                } else {
                    (&vertices[i + 1], &vertices[i], &vertices[i + 2])
                };
                self.rasterize_triangle(fb, v0, v1, v2, pipeline, state, internal_formats);
            }
        }
    }
}

/// Calculate screen position from clip-space position
fn screen_position(pos: &[f32; 4], vx: i32, vy: i32, vw: u32, vh: u32) -> (f32, f32) {
    (
        vx as f32 + (pos[0] / pos[3] + 1.0) * 0.5 * vw as f32,
        vy as f32 + (pos[1] / pos[3] + 1.0) * 0.5 * vh as f32,
    )
}

/// Calculate barycentric coordinates
pub fn barycentric(p: (f32, f32), a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> (f32, f32, f32) {
    let area = (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0);
    if area.abs() < 1e-6 {
        return (-1.0, -1.0, -1.0);
    }
    let w0 = ((b.0 - p.0) * (c.1 - p.1) - (b.1 - p.1) * (c.0 - p.0)) / area;
    let w1 = ((c.0 - p.0) * (a.1 - p.1) - (c.1 - p.1) * (a.0 - p.0)) / area;
    let w2 = 1.0 - w0 - w1;
    (w0, w1, w2)
}

fn is_inside(px: f32, py: f32, p0: (f32, f32), p1: (f32, f32), p2: (f32, f32)) -> bool {
    let edge0 = (px - p0.0) * (p1.1 - p0.1) - (py - p0.1) * (p1.0 - p0.0);
    let edge1 = (px - p1.0) * (p2.1 - p1.1) - (py - p1.1) * (p2.0 - p1.0);
    let edge2 = (px - p2.0) * (p0.1 - p2.1) - (py - p2.1) * (p0.0 - p2.0);

    (edge0 >= 0.0 && edge1 >= 0.0 && edge2 >= 0.0) || (edge0 <= 0.0 && edge1 <= 0.0 && edge2 <= 0.0)
}

fn compare_depth(func: u32, incoming: f32, current: f32) -> bool {
    match func {
        GL_NEVER => false,                  // GL_NEVER
        GL_LESS => incoming < current,      // GL_LESS
        GL_EQUAL => incoming == current,    // GL_EQUAL
        GL_LEQUAL => incoming <= current,   // GL_LEQUAL
        GL_GREATER => incoming > current,   // GL_GREATER
        GL_NOTEQUAL => incoming != current, // GL_NOTEQUAL
        GL_GEQUAL => incoming >= current,   // GL_GEQUAL
        GL_ALWAYS => true,                  // GL_ALWAYS
        _ => false,
    }
}

fn compare_stencil(func: u32, ref_val: i32, current: u8, mask: u32) -> bool {
    let c = (current as u32) & mask;
    let r = (ref_val as u32) & mask;
    match func {
        GL_NEVER => false,     // GL_NEVER
        GL_LESS => r < c,      // GL_LESS
        GL_EQUAL => r == c,    // GL_EQUAL
        GL_LEQUAL => r <= c,   // GL_LEQUAL
        GL_GREATER => r > c,   // GL_GREATER
        GL_NOTEQUAL => r != c, // GL_NOTEQUAL
        GL_GEQUAL => r >= c,   // GL_GEQUAL
        GL_ALWAYS => true,     // GL_ALWAYS
        _ => false,
    }
}

fn apply_stencil_op(op: u32, current: u8, ref_val: i32) -> u8 {
    let c = current as i32;
    match op {
        GL_ZERO => 0,                                  // GL_ZERO
        GL_KEEP => current,                            // GL_KEEP
        GL_REPLACE => ref_val as u8,                   // GL_REPLACE
        GL_INCR => c.saturating_add(1).min(255) as u8, // GL_INCR
        GL_DECR => c.saturating_sub(1).max(0) as u8,   // GL_DECR
        GL_INVERT => !current,                         // GL_INVERT
        GL_INCR_WRAP => ((c + 1) % 256) as u8,         // GL_INCR_WRAP
        GL_DECR_WRAP => ((c - 1 + 256) % 256) as u8,   // GL_DECR_WRAP
        _ => current,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_barycentric_inside() {
        let p0 = (0.0, 0.0);
        let p1 = (10.0, 0.0);
        let p2 = (5.0, 10.0);

        // Center of triangle
        let (u, v, w) = barycentric((5.0, 3.0), p0, p1, p2);
        assert!(u >= 0.0 && v >= 0.0 && w >= 0.0);
        assert!((u + v + w - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_barycentric_outside() {
        let p0 = (0.0, 0.0);
        let p1 = (10.0, 0.0);
        let p2 = (5.0, 10.0);

        // Point outside triangle
        let (u, v, w) = barycentric((20.0, 20.0), p0, p1, p2);
        assert!(u < 0.0 || v < 0.0 || w < 0.0);
    }

    #[test]
    fn test_shader_memory_layout_new() {
        let layout = ShaderMemoryLayout::new(0);
        assert!(layout.attr_ptr > 0);
        assert!(layout.uniform_ptr > 0);
        assert!(layout.varying_ptr > 0);
    }

    #[test]
    fn test_raster_pipeline_new() {
        let pipeline = RasterPipeline::new(0);
        assert_eq!(pipeline.vertex_shader_type, GL_VERTEX_SHADER);
        assert_eq!(pipeline.fragment_shader_type, GL_FRAGMENT_SHADER);
    }
}

#[cfg(test)]
#[path = "rasterizer_tests.rs"]
mod rasterizer_tests;
