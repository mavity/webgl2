//! Triangle rasterizer

/// Software triangle rasterizer
pub struct Rasterizer {}

impl Rasterizer {
    pub fn new() -> Self {
        Self {}
    }

    /// Draw a single point to the framebuffer
    pub fn draw_point(&self, fb: &mut super::Framebuffer, x: f32, y: f32, color: [u8; 4]) {
        let ix = x as i32;
        let iy = y as i32;
        if ix >= 0 && ix < fb.width as i32 && iy >= 0 && iy < fb.height as i32 {
            let idx = ((iy as u32 * fb.width + ix as u32) * 4) as usize;
            if idx + 3 < fb.color.len() {
                fb.color[idx..idx + 4].copy_from_slice(&color);
            }
        }
    }

    /// Draw a triangle to the framebuffer
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
}

fn is_inside(px: f32, py: f32, p0: (f32, f32), p1: (f32, f32), p2: (f32, f32)) -> bool {
    let edge0 = (px - p0.0) * (p1.1 - p0.1) - (py - p0.1) * (p1.0 - p0.0);
    let edge1 = (px - p1.0) * (p2.1 - p1.1) - (py - p1.1) * (p2.0 - p1.0);
    let edge2 = (px - p2.0) * (p0.1 - p2.1) - (py - p2.1) * (p0.0 - p2.0);

    (edge0 >= 0.0 && edge1 >= 0.0 && edge2 >= 0.0) || (edge0 <= 0.0 && edge1 <= 0.0 && edge2 <= 0.0)
}
