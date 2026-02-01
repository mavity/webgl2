/// WebGL2 rendering context state
pub struct WebGLState {
    pub viewport: Viewport,
    pub clear_color: [f32; 4],
    pub depth_test_enabled: bool,
    pub blend_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Default for WebGLState {
    fn default() -> Self {
        Self::new()
    }
}

impl WebGLState {
    pub fn new() -> Self {
        Self {
            viewport: Viewport {
                x: 0,
                y: 0,
                width: 800,
                height: 600,
            },
            clear_color: [0.0, 0.0, 0.0, 1.0],
            depth_test_enabled: false,
            blend_enabled: false,
        }
    }

    pub fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.viewport = Viewport {
            x,
            y,
            width,
            height,
        };
    }

    pub fn set_clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.clear_color = [r, g, b, a];
    }
}
