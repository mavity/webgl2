//! Comprehensive tests for rasterizer components

use super::*;
use crate::wasm_gl_emu::OwnedFramebuffer;

#[test]
fn test_processed_vertex_creation() {
    let vertex = ProcessedVertex {
        position: [1.0, 2.0, 3.0, 4.0],
        varyings: vec![0.5f32.to_bits(), 0.6f32.to_bits(), 0.7f32.to_bits()],
    };

    assert_eq!(vertex.position[0], 1.0);
    assert_eq!(vertex.varyings.len(), 3);
    assert_eq!(f32::from_bits(vertex.varyings[2]), 0.7);
}

#[test]
fn test_shader_memory_layout_custom() {
    let layout = ShaderMemoryLayout {
        attr_ptr: 0x1000,
        uniform_ptr: 0x2000,
        varying_ptr: 0x3000,
        private_ptr: 0x4000,
        texture_ptr: 0x5000,
    };

    assert_eq!(layout.attr_ptr, 0x1000);
    assert_eq!(layout.uniform_ptr, 0x2000);
}

#[test]
fn test_raster_pipeline_custom() {
    let pipeline = RasterPipeline {
        vertex_shader_type: 100,
        fragment_shader_type: 200,
        memory: ShaderMemoryLayout::default(),
        flat_varyings_mask: 0,
        vs_table_idx: None,
        fs_table_idx: None,
    };

    assert_eq!(pipeline.vertex_shader_type, 100);
    assert_eq!(pipeline.fragment_shader_type, 200);
}

#[test]
fn test_screen_position() {
    let pos = [2.0, 3.0, 1.0, 2.0]; // Clip space
    let (x, y) = screen_position(&pos, 0, 0, 800, 600);

    // (2/2 + 1) * 0.5 * 800 = (1 + 1) * 0.5 * 800 = 800
    assert_eq!(x, 800.0);
    // (3/2 + 1) * 0.5 * 600 = (1.5 + 1) * 0.5 * 600 = 750
    assert_eq!(y, 750.0);
}

#[test]
fn test_barycentric_vertices() {
    let p0 = (0.0, 0.0);
    let p1 = (10.0, 0.0);
    let p2 = (5.0, 10.0);

    // Test at vertex p0
    let (u, v, w) = barycentric(p0, p0, p1, p2);
    assert!((u - 1.0).abs() < 0.01);
    assert!(v.abs() < 0.01);
    assert!(w.abs() < 0.01);

    // Test at vertex p1
    let (u, v, w) = barycentric(p1, p0, p1, p2);
    assert!(u.abs() < 0.01);
    assert!((v - 1.0).abs() < 0.01);
    assert!(w.abs() < 0.01);
}

#[test]
fn test_barycentric_edge() {
    let p0 = (0.0, 0.0);
    let p1 = (10.0, 0.0);
    let p2 = (5.0, 10.0);

    // Midpoint of edge p0-p1
    let (u, v, w) = barycentric((5.0, 0.0), p0, p1, p2);
    assert!((u - 0.5).abs() < 0.01);
    assert!((v - 0.5).abs() < 0.01);
    assert!(w.abs() < 0.01);
}

#[test]
fn test_barycentric_degenerate() {
    // Degenerate triangle (all points on a line)
    let p0 = (0.0, 0.0);
    let p1 = (5.0, 0.0);
    let p2 = (10.0, 0.0);

    let (u, v, w) = barycentric((5.0, 0.0), p0, p1, p2);
    assert!(u < 0.0 || v < 0.0 || w < 0.0); // Should be invalid
}

#[test]
fn test_rasterizer_draw_point() {
    let rasterizer = Rasterizer::default();
    let mut kernel = GpuKernel::new();
    let mut owned_fb = OwnedFramebuffer::new(&mut kernel, 100, 100);
    let mut fb = owned_fb.as_framebuffer(&mut kernel);
    let state = RenderState {
        ctx_handle: 0,
        memory: ShaderMemoryLayout::default(),
        viewport: (0, 0, 100, 100),
        scissor: (0, 0, 100, 100),
        scissor_enabled: false,
        uniform_data: &[],
        prepare_textures: None,
        blend: BlendState::default(),
        color_mask: ColorMaskState::default(),
        depth: DepthState::default(),
        stencil: StencilState::default(),
    };

    // Draw a point at (50, 50)
    rasterizer.draw_point(&mut fb, 50.0, 50.0, &[255, 0, 0, 255], &state);

    // Check the pixel was written
    let idx = fb.get_pixel_offset(50, 50, 0);
    assert_eq!(fb.color[idx], 255);
    assert_eq!(fb.color[idx + 1], 0);
    assert_eq!(fb.color[idx + 2], 0);
    assert_eq!(fb.color[idx + 3], 255);
}

#[test]
fn test_rasterizer_draw_point_out_of_bounds() {
    let rasterizer = Rasterizer::default();
    let mut kernel = GpuKernel::new();
    let mut owned_fb = OwnedFramebuffer::new(&mut kernel, 100, 100);
    let mut fb = owned_fb.as_framebuffer(&mut kernel);
    let state = RenderState {
        ctx_handle: 0,
        memory: ShaderMemoryLayout::default(),
        viewport: (0, 0, 100, 100),
        scissor: (0, 0, 100, 100),
        scissor_enabled: false,
        uniform_data: &[],
        prepare_textures: None,
        blend: BlendState::default(),
        color_mask: ColorMaskState::default(),
        depth: DepthState::default(),
        stencil: StencilState::default(),
    };

    // Try to draw outside framebuffer
    rasterizer.draw_point(&mut fb, -10.0, -10.0, &[255, 0, 0, 255], &state);
    rasterizer.draw_point(&mut fb, 200.0, 200.0, &[255, 0, 0, 255], &state);

    // Framebuffer should remain unchanged (all zeros)
    let all_zero = fb.color.iter().all(|&x| x == 0);
    assert!(all_zero);
}

#[test]
fn test_rasterizer_draw_simple_triangle() {
    let rasterizer = Rasterizer::default();
    let mut kernel = GpuKernel::new();
    let mut owned_fb = OwnedFramebuffer::new(&mut kernel, 100, 100);
    let mut fb = owned_fb.as_framebuffer(&mut kernel);

    // Draw a small triangle
    let p0 = (10.0, 10.0);
    let p1 = (20.0, 10.0);
    let p2 = (15.0, 20.0);

    rasterizer.draw_triangle(&mut fb, p0, p1, p2, [0, 255, 0, 255]);

    // Check that some pixels were written (triangle center should be colored)
    let idx = fb.get_pixel_offset(15, 15, 0);
    assert_eq!(fb.color[idx + 1], 255); // Green channel
}

#[test]
fn test_is_inside_function() {
    let p0 = (0.0, 0.0);
    let p1 = (10.0, 0.0);
    let p2 = (5.0, 10.0);

    // Point inside
    assert!(is_inside(5.0, 3.0, p0, p1, p2));

    // Point outside
    assert!(!is_inside(20.0, 20.0, p0, p1, p2));

    // Point on edge
    assert!(is_inside(5.0, 0.0, p0, p1, p2));
}

#[test]
fn test_render_state_creation() {
    let uniform_data = vec![1u8, 2, 3, 4];
    let state = RenderState {
        ctx_handle: 1,
        memory: ShaderMemoryLayout::default(),
        viewport: (0, 0, 800, 600),
        scissor: (0, 0, 800, 600),
        scissor_enabled: false,
        uniform_data: &uniform_data,
        prepare_textures: None,
        blend: BlendState::default(),
        color_mask: ColorMaskState::default(),
        depth: DepthState::default(),
        stencil: StencilState::default(),
    };

    assert_eq!(state.viewport.2, 800);
    assert_eq!(state.uniform_data.len(), 4);
}

#[test]
fn test_perspective_interpolation_setup() {
    let v0 = ProcessedVertex {
        position: [0.0, 0.0, 0.0, 1.0],
        varyings: vec![1.0f32.to_bits(), 0.0f32.to_bits(), 0.0f32.to_bits()],
    };
    let v1 = ProcessedVertex {
        position: [1.0, 0.0, 0.0, 1.0],
        varyings: vec![0.0f32.to_bits(), 1.0f32.to_bits(), 0.0f32.to_bits()],
    };
    let v2 = ProcessedVertex {
        position: [0.5, 1.0, 0.0, 1.0],
        varyings: vec![0.0f32.to_bits(), 0.0f32.to_bits(), 1.0f32.to_bits()],
    };

    // Just verify we can create vertices with varyings
    assert_eq!(v0.varyings[0], 1.0f32.to_bits());
    assert_eq!(v1.varyings[1], 1.0f32.to_bits());
    assert_eq!(v2.varyings[2], 1.0f32.to_bits());
}
