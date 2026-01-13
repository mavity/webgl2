//! Triangle rasterizer - shared between WebGL2 and WebGPU
//!
//! This module provides a driver-agnostic software rasterizer that can be used
//! by both WebGL2 and WebGPU implementations. It handles vertex fetching,
//! barycentric interpolation, and fragment shading.

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
#[derive(Clone, Copy, Debug)]
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
}

impl Default for ShaderMemoryLayout {
    fn default() -> Self {
        // Default WebGL-compatible memory layout
        Self {
            attr_ptr: 0x2000,
            uniform_ptr: 0x1000,
            varying_ptr: 0x3000,
            private_ptr: 0x4000,
            texture_ptr: 0x5000,
        }
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
    /// Uniform data buffer
    pub uniform_data: &'a [u8],
    /// Texture metadata preparation callback
    pub prepare_textures: Option<Box<dyn Fn(u32) + 'a>>,
    /// Blend state
    pub blend: BlendState,
}

/// Interface for fetching vertex attributes
pub trait VertexFetcher {
    /// Fetch attributes for a specific vertex and instance
    /// Writes data directly to the destination buffer (which maps to attr_ptr)
    fn fetch(&self, vertex_index: u32, instance_index: u32, dest: &mut [u8]);
}

/// Pipeline configuration for rasterization
/// Decouples from WebGL's Program object to support WebGPU
pub struct VaryingDebug {
    pub name: String,
    pub location: u32,
    pub type_code: u8,   // 0=float, 1=int, 2=uint
    pub components: u32, // number of scalar components
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
            src_rgb: 1,       // GL_ONE
            dst_rgb: 0,       // GL_ZERO
            src_alpha: 1,     // GL_ONE
            dst_alpha: 0,     // GL_ZERO
            eq_rgb: 0x8006,   // GL_FUNC_ADD
            eq_alpha: 0x8006, // GL_FUNC_ADD
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
}

impl Default for RasterPipeline {
    fn default() -> Self {
        Self {
            vertex_shader_type: 0x8B31,   // GL_VERTEX_SHADER
            fragment_shader_type: 0x8B30, // GL_FRAGMENT_SHADER
            memory: ShaderMemoryLayout::default(),
            flat_varyings_mask: 0,
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
        0 => [0.0, 0.0, 0.0, 0.0],                                          // ZERO
        1 => [1.0, 1.0, 1.0, 1.0],                                          // ONE
        0x0300 => src,                                                      // SRC_COLOR
        0x0301 => [1.0 - src[0], 1.0 - src[1], 1.0 - src[2], 1.0 - src[3]], // ONE_MINUS_SRC_COLOR
        0x0302 => [src[3], src[3], src[3], src[3]],                         // SRC_ALPHA
        0x0303 => [1.0 - src[3], 1.0 - src[3], 1.0 - src[3], 1.0 - src[3]], // ONE_MINUS_SRC_ALPHA
        0x0304 => [dst[3], dst[3], dst[3], dst[3]],                         // DST_ALPHA
        0x0305 => [1.0 - dst[3], 1.0 - dst[3], 1.0 - dst[3], 1.0 - dst[3]], // ONE_MINUS_DST_ALPHA
        0x0306 => dst,                                                      // DST_COLOR
        0x0307 => [1.0 - dst[0], 1.0 - dst[1], 1.0 - dst[2], 1.0 - dst[3]], // ONE_MINUS_DST_COLOR
        0x0308 => [alpha_sat, alpha_sat, alpha_sat, 1.0],                   // SRC_ALPHA_SATURATE
        0x8001 => constant,                                                 // CONSTANT_COLOR
        0x8002 => [
            1.0 - constant[0],
            1.0 - constant[1],
            1.0 - constant[2],
            1.0 - constant[3],
        ], // ONE_MINUS_CONSTANT_COLOR
        0x8003 => [constant[3], constant[3], constant[3], constant[3]],     // CONSTANT_ALPHA
        0x8004 => [
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
        0x8006 => src * s_factor + dst * d_factor, // FUNC_ADD
        0x800A => src * s_factor - dst * d_factor, // FUNC_SUBTRACT
        0x800B => dst * d_factor - src * s_factor, // FUNC_REVERSE_SUBTRACT
        0x8007 => src.min(dst),                    // MIN
        0x8008 => src.max(dst),                    // MAX
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

/// Software triangle rasterizer
#[derive(Default)]
pub struct Rasterizer {}

pub struct DrawConfig<'a> {
    pub fb: &'a mut super::Framebuffer<'a>,
    pub pipeline: &'a RasterPipeline,
    pub state: &'a RenderState<'a>,
    pub vertex_fetcher: &'a dyn VertexFetcher,
    pub vertex_count: usize,
    pub instance_count: usize,
    pub first_vertex: usize,
    pub first_instance: usize,
    pub indices: Option<&'a [u32]>,
    pub mode: u32,
}

impl Rasterizer {
    /// Draw a single point to the framebuffer
    pub fn draw_point(
        &self,
        fb: &mut super::Framebuffer,
        x: f32,
        y: f32,
        color: [u8; 4],
        state: &RenderState,
    ) {
        let ix = x as i32;
        let iy = y as i32;
        if ix >= 0 && ix < fb.width as i32 && iy >= 0 && iy < fb.height as i32 {
            let idx = ((iy as u32 * fb.width + ix as u32) * 4) as usize;
            if idx + 3 < fb.color.len() {
                let existing = [
                    fb.color[idx],
                    fb.color[idx + 1],
                    fb.color[idx + 2],
                    fb.color[idx + 3],
                ];
                let blended = blend_pixel(color, existing, &state.blend);
                fb.color[idx..idx + 4].copy_from_slice(&blended);
            }
        }
    }

    /// Draw a triangle to the framebuffer (simple, no interpolation)
    pub fn draw_triangle(
        &self,
        fb: &mut super::Framebuffer,
        p0: (f32, f32),
        p1: (f32, f32),
        p2: (f32, f32),
        color: [u8; 4],
    ) {
        let min_x = p0.0.min(p1.0).min(p2.0).max(0.0).floor() as i32;
        let max_x = p0.0.max(p1.0).max(p2.0).min(fb.width as f32 - 1.0).ceil() as i32;
        let min_y = p0.1.min(p1.1).min(p2.1).max(0.0).floor() as i32;
        let max_y = p0.1.max(p1.1).max(p2.1).min(fb.height as f32 - 1.0).ceil() as i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;

                if is_inside(px, py, p0, p1, p2) {
                    let idx = ((y as u32 * fb.width + x as u32) * 4) as usize;
                    if idx + 3 < fb.color.len() {
                        fb.color[idx..idx + 4].copy_from_slice(&color);
                    }
                }
            }
        }
    }

    /// Rasterize a triangle with perspective-correct interpolation
    /// This is the core rasterization function extracted from drawing.rs
    pub fn rasterize_triangle(
        &self,
        fb: &mut super::Framebuffer,
        v0: &ProcessedVertex,
        v1: &ProcessedVertex,
        v2: &ProcessedVertex,
        pipeline: &RasterPipeline,
        state: &RenderState,
    ) {
        let (vx, vy, vw, vh) = state.viewport;

        // Screen coordinates (with perspective divide)
        let p0 = screen_position(&v0.position, vx, vy, vw, vh);
        let p1 = screen_position(&v1.position, vx, vy, vw, vh);
        let p2 = screen_position(&v2.position, vx, vy, vw, vh);

        // Bounding box
        let min_x = p0.0.min(p1.0).min(p2.0).max(0.0).floor() as i32;
        let max_x = p0.0.max(p1.0).max(p2.0).min(vw as f32 - 1.0).ceil() as i32;
        let min_y = p0.1.min(p1.1).min(p2.1).max(0.0).floor() as i32;
        let max_y = p0.1.max(p1.1).max(p2.1).min(vh as f32 - 1.0).ceil() as i32;

        if max_x < min_x || max_y < min_y {
            return;
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
                let (u, v, w) = barycentric((x as f32 + 0.5, y as f32 + 0.5), p0, p1, p2);

                if u >= 0.0 && v >= 0.0 && w >= 0.0 {
                    // Interpolate depth (NDC z/w mapped to [0, 1])
                    let z0 = v0.position[2] / v0.position[3];
                    let z1 = v1.position[2] / v1.position[3];
                    let z2 = v2.position[2] / v2.position[3];
                    let depth_ndc = u * z0 + v * z1 + w * z2;
                    let depth = (depth_ndc + 1.0) * 0.5;

                    let fb_idx = (y as u32 * fb.width + x as u32) as usize;

                    // Depth test
                    if (0.0..=1.0).contains(&depth) && depth < fb.depth[fb_idx] {
                        fb.depth[fb_idx] = depth;

                        // Perspective correct interpolation of varyings
                        let w_interp_inv = u * w0_inv + v * w1_inv + w * w2_inv;
                        let w_interp = 1.0 / w_interp_inv;

                        for (k, varying) in interp_varyings.iter_mut().enumerate() {
                            if (pipeline.flat_varyings_mask & (1 << k)) != 0 {
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
                        let color = self.execute_fragment_shader(&interp_varyings, pipeline, state);

                        // Write color to framebuffer
                        let color_idx = fb_idx * 4;
                        if color_idx + 3 < fb.color.len() {
                            let existing = [
                                fb.color[color_idx],
                                fb.color[color_idx + 1],
                                fb.color[color_idx + 2],
                                fb.color[color_idx + 3],
                            ];
                            let blended = blend_pixel(color, existing, &state.blend);
                            fb.color[color_idx..color_idx + 4].copy_from_slice(&blended);
                        }
                    }
                }
            }
        }
    }

    /// Execute fragment shader and return RGBA color
    fn execute_fragment_shader(
        &self,
        varyings: &[u32],
        pipeline: &RasterPipeline,
        state: &RenderState,
    ) -> [u8; 4] {
        // Copy varyings to shader memory as raw bits
        unsafe {
            std::ptr::copy_nonoverlapping(
                varyings.as_ptr() as *const u8,
                pipeline.memory.varying_ptr as *mut u8,
                varyings.len() * 4,
            );
        }

        // Execute fragment shader
        crate::js_execute_shader(
            state.ctx_handle,
            pipeline.fragment_shader_type,
            0,
            pipeline.memory.uniform_ptr,
            pipeline.memory.varying_ptr,
            pipeline.memory.private_ptr,
            pipeline.memory.texture_ptr,
        );

        // Read color from private memory
        let mut color_bytes = [0u8; 16];
        unsafe {
            std::ptr::copy_nonoverlapping(
                pipeline.memory.private_ptr as *const u8,
                color_bytes.as_mut_ptr(),
                16,
            );
        }

        let c: [f32; 4] = unsafe { std::mem::transmute(color_bytes) };

        [
            (c[0].clamp(0.0, 1.0) * 255.0) as u8,
            (c[1].clamp(0.0, 1.0) * 255.0) as u8,
            (c[2].clamp(0.0, 1.0) * 255.0) as u8,
            (c[3].clamp(0.0, 1.0) * 255.0) as u8,
        ]
    }

    /// Draw primitives
    pub fn draw(&self, config: DrawConfig) {
        let (vx, vy, vw, vh) = config.state.viewport;

        // Allocate attribute buffer (enough for 16 locations * 16 floats = 1024 bytes)
        // This should match the size expected by the shader
        let mut attr_buffer = vec![0u8; 1024];

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
                    idxs[i]
                } else {
                    (config.first_vertex + i) as u32
                };

                // Fetch attributes
                config
                    .vertex_fetcher
                    .fetch(vertex_id, actual_instance_id as u32, &mut attr_buffer);

                // Copy attributes to shader memory
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        attr_buffer.as_ptr(),
                        config.pipeline.memory.attr_ptr as *mut u8,
                        attr_buffer.len(),
                    );
                    // Copy uniforms
                    std::ptr::copy_nonoverlapping(
                        config.state.uniform_data.as_ptr(),
                        config.pipeline.memory.uniform_ptr as *mut u8,
                        config.state.uniform_data.len(),
                    );

                    // Prepare textures
                    if let Some(ref prepare) = config.state.prepare_textures {
                        prepare(config.pipeline.memory.texture_ptr);
                    }
                }

                // Execute Vertex Shader
                crate::js_execute_shader(
                    config.state.ctx_handle,
                    config.pipeline.vertex_shader_type,
                    config.pipeline.memory.attr_ptr,
                    config.pipeline.memory.uniform_ptr,
                    config.pipeline.memory.varying_ptr,
                    config.pipeline.memory.private_ptr,
                    config.pipeline.memory.texture_ptr,
                );

                // Capture position and varyings
                let mut pos_bytes = [0u8; 16];
                let mut varying_bytes = vec![0u8; 256]; // Capture first 256 bytes of varyings
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        config.pipeline.memory.varying_ptr as *const u8,
                        pos_bytes.as_mut_ptr(),
                        16,
                    );
                    std::ptr::copy_nonoverlapping(
                        config.pipeline.memory.varying_ptr as *const u8,
                        varying_bytes.as_mut_ptr(),
                        256,
                    );
                }
                let pos: [f32; 4] = unsafe { std::mem::transmute(pos_bytes) };

                // Read varyings as raw u32 bits to avoid NaN canonicalization
                let varyings_u32: Vec<u32> =
                    unsafe { std::slice::from_raw_parts(varying_bytes.as_ptr() as *const u32, 64) }
                        .to_vec();

                vertices.push(ProcessedVertex {
                    position: pos,
                    varyings: varyings_u32,
                });
            }

            // 2. Rasterize
            if config.mode == 0x0000 {
                // GL_POINTS
                for v in &vertices {
                    let screen_x =
                        vx as f32 + (v.position[0] / v.position[3] + 1.0) * 0.5 * vw as f32;
                    let screen_y =
                        vy as f32 + (v.position[1] / v.position[3] + 1.0) * 0.5 * vh as f32;

                    // Run FS
                    let color =
                        self.execute_fragment_shader(&v.varyings, config.pipeline, config.state);
                    self.draw_point(config.fb, screen_x, screen_y, color, config.state);
                }
            } else if config.mode == 0x0004 {
                // GL_TRIANGLES
                for i in (0..vertices.len()).step_by(3) {
                    if i + 2 >= vertices.len() {
                        break;
                    }
                    let v0 = &vertices[i];
                    let v1 = &vertices[i + 1];
                    let v2 = &vertices[i + 2];

                    self.rasterize_triangle(config.fb, v0, v1, v2, config.pipeline, config.state);
                }
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
    fn test_shader_memory_layout_default() {
        let layout = ShaderMemoryLayout::default();
        assert_eq!(layout.attr_ptr, 0x2000);
        assert_eq!(layout.uniform_ptr, 0x1000);
        assert_eq!(layout.varying_ptr, 0x3000);
    }

    #[test]
    fn test_raster_pipeline_default() {
        let pipeline = RasterPipeline::default();
        assert_eq!(pipeline.vertex_shader_type, 0x8B31);
        assert_eq!(pipeline.fragment_shader_type, 0x8B30);
    }
}

#[cfg(test)]
#[path = "rasterizer_tests.rs"]
mod rasterizer_tests;
