pub(crate) use crate::wasm_gl_emu::device::GpuHandle;
pub(crate) use crate::wasm_gl_emu::device::GpuKernel;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

// Errno constants (must match JS constants if exposed)
pub const ERR_OK: u32 = 0;
pub const ERR_INVALID_HANDLE: u32 = 1;
pub const ERR_OOM: u32 = 2;
pub const ERR_INVALID_ARGS: u32 = 3;
pub const ERR_NOT_IMPLEMENTED: u32 = 4;
pub const ERR_GL: u32 = 5;
pub const ERR_INTERNAL: u32 = 6;
pub const ERR_INVALID_OPERATION: u32 = 7;
pub const ERR_INVALID_ENUM: u32 = 8;

// GL Error constants
pub const GL_NO_ERROR: u32 = 0;
pub const GL_INVALID_ENUM: u32 = 0x0500;
pub const GL_INVALID_VALUE: u32 = 0x0501;
pub const GL_INVALID_OPERATION: u32 = 0x0502;
pub const GL_OUT_OF_MEMORY: u32 = 0x0505;

pub const GL_ARRAY_BUFFER: u32 = 0x8892;
pub const GL_ELEMENT_ARRAY_BUFFER: u32 = 0x8893;
pub const GL_COPY_READ_BUFFER: u32 = 0x8F36;
pub const GL_COPY_WRITE_BUFFER: u32 = 0x8F37;
pub const GL_PIXEL_PACK_BUFFER: u32 = 0x88EB;
pub const GL_PIXEL_UNPACK_BUFFER: u32 = 0x88EC;
pub const GL_UNIFORM_BUFFER: u32 = 0x8A11;
pub const GL_TRANSFORM_FEEDBACK_BUFFER: u32 = 0x8C8E;

pub const GL_COMPILE_STATUS: u32 = 0x8B81;
pub const GL_LINK_STATUS: u32 = 0x8B82;
pub const GL_SHADER_TYPE: u32 = 0x8B4F;
pub const GL_DELETE_STATUS: u32 = 0x8B80;
pub const GL_INFO_LOG_LENGTH: u32 = 0x8B84;
pub const GL_ATTACHED_SHADERS: u32 = 0x8B85;
pub const GL_ACTIVE_UNIFORMS: u32 = 0x8B86;
pub const GL_ACTIVE_ATTRIBUTES: u32 = 0x8B89;

pub const GL_POINTS: u32 = 0x0000;
pub const GL_LINES: u32 = 0x0001;
pub const GL_LINE_LOOP: u32 = 0x0002;
pub const GL_LINE_STRIP: u32 = 0x0003;
pub const GL_TRIANGLES: u32 = 0x0004;
pub const GL_TRIANGLE_STRIP: u32 = 0x0005;
pub const GL_TRIANGLE_FAN: u32 = 0x0006;

pub const GL_BYTE: u32 = 0x1400;
pub const GL_UNSIGNED_BYTE: u32 = 0x1401;
pub const GL_SHORT: u32 = 0x1402;
pub const GL_UNSIGNED_SHORT: u32 = 0x1403;
pub const GL_INT: u32 = 0x1404;
pub const GL_UNSIGNED_INT: u32 = 0x1405;
pub const GL_FLOAT: u32 = 0x1406;
pub const GL_HALF_FLOAT: u32 = 0x140B;

pub const GL_TEXTURE_2D: u32 = 0x0DE1;
pub const GL_TEXTURE_3D: u32 = 0x806F;
pub const GL_TEXTURE_2D_ARRAY: u32 = 0x8C1A;

pub const GL_RGBA: u32 = 0x1908;
pub const GL_RGB: u32 = 0x1907;
pub const GL_RED: u32 = 0x1903;
pub const GL_RG: u32 = 0x8227;
pub const GL_DEPTH_COMPONENT: u32 = 0x1902;
pub const GL_UNSIGNED_INT_24_8: u32 = 0x84FA;
pub const GL_DEPTH24_STENCIL8: u32 = 0x88F0;

pub const GL_RGBA8: u32 = 0x8058;
pub const GL_RGB8: u32 = 0x8051;
pub const GL_R32F: u32 = 0x822E;
pub const GL_RG32F: u32 = 0x8230;
pub const GL_RGBA32F: u32 = 0x8814;
pub const GL_DEPTH_COMPONENT24: u32 = 0x81A6;
pub const GL_DEPTH_COMPONENT32F: u32 = 0x8CAC;

pub const GL_VERTEX_ATTRIB_ARRAY_ENABLED: u32 = 0x8622;
pub const GL_VERTEX_ATTRIB_ARRAY_SIZE: u32 = 0x8623;
pub const GL_VERTEX_ATTRIB_ARRAY_STRIDE: u32 = 0x8624;
pub const GL_VERTEX_ATTRIB_ARRAY_TYPE: u32 = 0x8625;
pub const GL_VERTEX_ATTRIB_ARRAY_NORMALIZED: u32 = 0x886A;
pub const GL_VERTEX_ATTRIB_ARRAY_POINTER: u32 = 0x8645;
pub const GL_VERTEX_ATTRIB_ARRAY_BUFFER_BINDING: u32 = 0x889F;
pub const GL_VERTEX_ATTRIB_ARRAY_DIVISOR: u32 = 0x88FE;
pub const GL_VERTEX_ATTRIB_ARRAY_INTEGER: u32 = 0x88FD;
pub const GL_CURRENT_VERTEX_ATTRIB: u32 = 0x8626;

pub const GL_TEXTURE_MAG_FILTER: u32 = 0x2800;
pub const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
pub const GL_TEXTURE_WRAP_S: u32 = 0x2802;
pub const GL_TEXTURE_WRAP_T: u32 = 0x2803;
pub const GL_TEXTURE_WRAP_R: u32 = 0x8072;

pub const GL_NEAREST: u32 = 0x2600;
pub const GL_LINEAR: u32 = 0x2601;
pub const GL_NEAREST_MIPMAP_NEAREST: u32 = 0x2700;
pub const GL_LINEAR_MIPMAP_NEAREST: u32 = 0x2701;
pub const GL_NEAREST_MIPMAP_LINEAR: u32 = 0x2702;
pub const GL_LINEAR_MIPMAP_LINEAR: u32 = 0x2703;
pub const GL_REPEAT: u32 = 0x2901;
pub const GL_CLAMP_TO_EDGE: u32 = 0x812F;
pub const GL_MIRRORED_REPEAT: u32 = 0x8370;

pub const GL_VERTEX_SHADER: u32 = 0x8B31;
pub const GL_FRAGMENT_SHADER: u32 = 0x8B30;

pub const GL_LESS: u32 = 0x0201;
pub const GL_EQUAL: u32 = 0x0202;
pub const GL_LEQUAL: u32 = 0x0203;
pub const GL_GREATER: u32 = 0x0204;
pub const GL_NOTEQUAL: u32 = 0x0205;
pub const GL_GEQUAL: u32 = 0x0206;
pub const GL_ALWAYS: u32 = 0x0207;
pub const GL_NEVER: u32 = 0x0200;

pub const GL_STENCIL_TEST: u32 = 0x0B90;
pub const GL_DEPTH_TEST: u32 = 0x0B71;
pub const GL_BLEND: u32 = 0x0BE2;
pub const GL_CULL_FACE: u32 = 0x0B44;
pub const GL_SCISSOR_TEST: u32 = 0x0C11;

pub const GL_KEEP: u32 = 0x1E00;
pub const GL_REPLACE: u32 = 0x1E01;
pub const GL_INCR: u32 = 0x1E02;
pub const GL_DECR: u32 = 0x1E03;
pub const GL_INVERT: u32 = 0x150A;
pub const GL_INCR_WRAP: u32 = 0x8507;
pub const GL_DECR_WRAP: u32 = 0x8508;

pub const GL_FUNC_ADD: u32 = 0x8006;
pub const GL_FUNC_SUBTRACT: u32 = 0x800A;
pub const GL_FUNC_REVERSE_SUBTRACT: u32 = 0x800B;
pub const GL_MIN: u32 = 0x8007;
pub const GL_MAX: u32 = 0x8008;

pub const GL_ZERO: u32 = 0;
pub const GL_ONE: u32 = 1;
pub const GL_SRC_COLOR: u32 = 0x0300;
pub const GL_ONE_MINUS_SRC_COLOR: u32 = 0x0301;
pub const GL_SRC_ALPHA: u32 = 0x0302;
pub const GL_ONE_MINUS_SRC_ALPHA: u32 = 0x0303;
pub const GL_DST_ALPHA: u32 = 0x0304;
pub const GL_ONE_MINUS_DST_ALPHA: u32 = 0x0305;
pub const GL_DST_COLOR: u32 = 0x0306;
pub const GL_ONE_MINUS_DST_COLOR: u32 = 0x0307;
pub const GL_SRC_ALPHA_SATURATE: u32 = 0x0308;
pub const GL_CONSTANT_COLOR: u32 = 0x8001;
pub const GL_ONE_MINUS_CONSTANT_COLOR: u32 = 0x8002;
pub const GL_CONSTANT_ALPHA: u32 = 0x8003;
pub const GL_ONE_MINUS_CONSTANT_ALPHA: u32 = 0x8004;

pub const GL_VIEWPORT: u32 = 0x0BA2;
pub const GL_COLOR_CLEAR_VALUE: u32 = 0x0C22;
pub const GL_BUFFER_SIZE: u32 = 0x8764;
pub const GL_COLOR_BUFFER_BIT: u32 = 0x00004000;
pub const GL_RENDERBUFFER: u32 = 0x8D41;
pub const GL_FRAMEBUFFER: u32 = 0x8D40;
pub const GL_DEPTH_COMPONENT16: u32 = 0x81A5;
pub const GL_DEPTH_STENCIL: u32 = 0x84F9;
pub const GL_RGBA4: u32 = 0x8056;
pub const GL_RGB565: u32 = 0x8D62;
pub const GL_RGB5_A1: u32 = 0x8057;
pub const GL_STENCIL_INDEX8: u32 = 0x8D48;

// Handle constants
pub(crate) const INVALID_HANDLE: u32 = 0;
pub(crate) const FIRST_HANDLE: u32 = 1;

/// A single mipmap level for a texture
#[derive(Clone, Debug)]
pub(crate) struct MipLevel {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) depth: u32,
    pub(crate) internal_format: u32,
    pub(crate) gpu_handle: GpuHandle,
}

/// A WebGL2 texture resource
#[derive(Clone)]
pub(crate) struct Texture {
    pub(crate) levels: BTreeMap<usize, MipLevel>,
    pub(crate) internal_format: u32,
    pub(crate) min_filter: u32,
    pub(crate) mag_filter: u32,
    pub(crate) wrap_s: u32,
    pub(crate) wrap_t: u32,
    pub(crate) wrap_r: u32,
}

/// A WebGL2 renderbuffer resource
#[derive(Clone)]
pub(crate) struct Renderbuffer {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) internal_format: u32,
    pub(crate) gpu_handle: GpuHandle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Attachment {
    Texture(u32),
    Renderbuffer(u32),
}

/// A WebGL2 framebuffer resource
#[derive(Clone)]
pub(crate) struct FramebufferObj {
    pub(crate) color_attachment: Option<Attachment>,
    pub(crate) depth_attachment: Option<Attachment>,
    pub(crate) stencil_attachment: Option<Attachment>,
}

/// A WebGL2 buffer resource
#[derive(Clone)]
pub(crate) struct Buffer {
    pub(crate) gpu_handle: crate::wasm_gl_emu::GpuHandle,
    pub(crate) usage: u32,
}

/// A WebGL2 shader resource
#[derive(Clone)]
pub(crate) struct Shader {
    pub(crate) type_: u32,
    pub(crate) source: String,
    pub(crate) compiled: bool,
    pub(crate) info_log: String,
    pub(crate) module: Option<Arc<naga::Module>>,
    pub(crate) info: Option<Arc<naga::valid::ModuleInfo>>,
}

/// A WebGL2 program resource
#[derive(Debug, Clone)]
pub(crate) struct ActiveInfo {
    pub(crate) name: String,
    pub(crate) size: i32,
    pub(crate) type_: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct Program {
    pub(crate) attached_shaders: Vec<u32>,
    pub(crate) linked: bool,
    pub(crate) info_log: String,
    pub(crate) attributes: HashMap<String, i32>,
    pub(crate) attribute_bindings: HashMap<String, u32>,
    pub(crate) uniforms: HashMap<String, i32>,
    pub(crate) uniform_types: HashMap<String, (u8, u32)>,
    pub(crate) active_attributes: Vec<ActiveInfo>,
    pub(crate) active_uniforms: Vec<ActiveInfo>,
    pub(crate) vs_module: Option<Arc<naga::Module>>,
    pub(crate) fs_module: Option<Arc<naga::Module>>,
    pub(crate) vs_info: Option<Arc<naga::valid::ModuleInfo>>,
    pub(crate) fs_info: Option<Arc<naga::valid::ModuleInfo>>,
    pub(crate) vs_wasm: Option<Vec<u8>>,
    pub(crate) fs_wasm: Option<Vec<u8>>,
    pub(crate) vs_stub: Option<String>,
    pub(crate) fs_stub: Option<String>,
    // Varying meta populated at link time (name -> location)
    pub(crate) varying_locations: HashMap<String, u32>,
    // Varying types populated at link time (name -> (type_code, components))
    // type_code: 0=float, 1=int (signed), 2=uint
    pub(crate) varying_types: HashMap<String, (u8, u32)>,
    // Attribute types populated at link time (name -> (type_code, components))
    pub(crate) attribute_types: HashMap<String, (u8, u32)>,
    /// Function table indices for direct calling
    pub(crate) vs_table_idx: Option<u32>,
    pub(crate) fs_table_idx: Option<u32>,
}

/// A WebGL2 vertex array object
#[derive(Clone)]
pub(crate) struct VertexArray {
    pub(crate) attributes: Vec<VertexAttribute>,
    pub(crate) element_array_buffer: Option<u32>,
}

impl Default for VertexArray {
    fn default() -> Self {
        VertexArray {
            attributes: vec![VertexAttribute::default(); 16],
            element_array_buffer: None,
        }
    }
}

/// A WebGL2 vertex attribute
#[derive(Clone)]
pub(crate) struct VertexAttribute {
    pub(crate) enabled: bool,
    pub(crate) size: i32,
    pub(crate) type_: u32,
    pub(crate) normalized: bool,
    pub(crate) stride: i32,
    pub(crate) offset: u32,
    pub(crate) buffer: Option<u32>,
    pub(crate) default_value: [u32; 4], // Store as raw bits
    pub(crate) divisor: u32,
    pub(crate) is_integer: bool,
    pub(crate) current_value_type: u32, // GL_FLOAT, GL_INT, or GL_UNSIGNED_INT
}

impl Default for VertexAttribute {
    fn default() -> Self {
        VertexAttribute {
            enabled: false,
            size: 4,
            type_: 0x1406, // GL_FLOAT
            normalized: false,
            stride: 0,
            offset: 0,
            buffer: None,
            default_value: [0, 0, 0, 0x3F800000], // 0.0, 0.0, 0.0, 1.0 (as float bits)
            divisor: 0,
            is_integer: false,
            current_value_type: 0x1406, // GL_FLOAT
        }
    }
}

/// Per-context state
pub struct Context {
    pub(crate) textures: HashMap<u32, Texture>,
    pub(crate) framebuffers: HashMap<u32, FramebufferObj>,
    pub(crate) buffers: HashMap<u32, Buffer>,
    pub(crate) shaders: HashMap<u32, Shader>,
    pub(crate) programs: HashMap<u32, Program>,
    pub(crate) vertex_arrays: HashMap<u32, VertexArray>,
    pub(crate) renderbuffers: HashMap<u32, Renderbuffer>,

    pub(crate) next_texture_handle: u32,
    pub(crate) next_framebuffer_handle: u32,
    pub(crate) next_buffer_handle: u32,
    pub(crate) next_shader_handle: u32,
    pub(crate) next_program_handle: u32,
    pub(crate) next_vertex_array_handle: u32,
    pub(crate) next_renderbuffer_handle: u32,

    pub(crate) bound_texture: Option<u32>,
    pub(crate) bound_read_framebuffer: Option<u32>,
    pub(crate) bound_draw_framebuffer: Option<u32>,
    pub(crate) bound_renderbuffer: Option<u32>,
    pub(crate) buffer_bindings: HashMap<u32, Option<u32>>,
    pub(crate) bound_vertex_array: u32,
    pub(crate) current_program: Option<u32>,

    pub(crate) uniform_data: Vec<u8>,

    // Software rendering state
    pub kernel: GpuKernel,
    pub default_framebuffer: crate::wasm_gl_emu::OwnedFramebuffer,
    pub rasterizer: crate::wasm_gl_emu::Rasterizer,

    // State
    pub(crate) clear_color: [f32; 4],
    pub(crate) viewport: (i32, i32, u32, u32),
    pub(crate) scissor_box: (i32, i32, u32, u32),
    pub(crate) scissor_test_enabled: bool,
    pub(crate) blend_state: crate::wasm_gl_emu::rasterizer::BlendState,
    pub(crate) color_mask: crate::wasm_gl_emu::rasterizer::ColorMaskState,
    pub(crate) depth_state: crate::wasm_gl_emu::rasterizer::DepthState,
    pub(crate) stencil_state: crate::wasm_gl_emu::rasterizer::StencilState,
    pub(crate) active_texture_unit: u32,
    pub(crate) texture_units: Vec<Option<u32>>,
    pub(crate) gl_error: u32,
    pub debug_shaders: bool,
}

impl Context {
    pub fn set_error(&mut self, error: u32) {
        if self.gl_error == GL_NO_ERROR {
            self.gl_error = error;
        }
    }

    pub fn new(width: u32, height: u32) -> Self {
        let mut vertex_arrays = HashMap::new();
        vertex_arrays.insert(0, VertexArray::default());

        let mut kernel = GpuKernel::default();
        let default_framebuffer =
            crate::wasm_gl_emu::OwnedFramebuffer::new(&mut kernel, width, height);

        Context {
            textures: HashMap::new(),
            framebuffers: HashMap::new(),
            buffers: HashMap::new(),
            shaders: HashMap::new(),
            programs: HashMap::new(),
            vertex_arrays,
            renderbuffers: HashMap::new(),

            next_texture_handle: FIRST_HANDLE,
            next_framebuffer_handle: FIRST_HANDLE,
            next_buffer_handle: FIRST_HANDLE,
            next_shader_handle: FIRST_HANDLE,
            next_program_handle: FIRST_HANDLE,
            next_vertex_array_handle: FIRST_HANDLE,
            next_renderbuffer_handle: FIRST_HANDLE,

            bound_texture: None,
            bound_read_framebuffer: None,
            bound_draw_framebuffer: None,
            bound_renderbuffer: None,
            buffer_bindings: HashMap::new(),
            bound_vertex_array: 0,
            current_program: None,

            uniform_data: vec![0; 4096], // 4KB for uniforms

            kernel,
            default_framebuffer,
            rasterizer: crate::wasm_gl_emu::Rasterizer::default(),

            clear_color: [0.0, 0.0, 0.0, 0.0],
            viewport: (0, 0, width, height),
            scissor_box: (0, 0, width, height),
            scissor_test_enabled: false,
            blend_state: crate::wasm_gl_emu::rasterizer::BlendState::default(),
            color_mask: crate::wasm_gl_emu::rasterizer::ColorMaskState::default(),
            depth_state: crate::wasm_gl_emu::rasterizer::DepthState {
                enabled: false,
                func: 0x0201, // GL_LESS (Default). Wait, types.rs had 0x0203 LEQUAL? Standard is LESS.
                mask: true,
            },
            stencil_state: crate::wasm_gl_emu::rasterizer::StencilState::default(),
            active_texture_unit: 0,
            texture_units: vec![None; 16],
            gl_error: GL_NO_ERROR,
            debug_shaders: false,
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new(640, 480)
    }
}

impl Context {
    pub(crate) fn allocate_texture_handle(&mut self) -> u32 {
        let h = self.next_texture_handle;
        self.next_texture_handle = self.next_texture_handle.saturating_add(1);
        if self.next_texture_handle == 0 {
            self.next_texture_handle = FIRST_HANDLE;
        }
        h
    }

    pub(crate) fn allocate_framebuffer_handle(&mut self) -> u32 {
        let h = self.next_framebuffer_handle;
        self.next_framebuffer_handle = self.next_framebuffer_handle.saturating_add(1);
        if self.next_framebuffer_handle == 0 {
            self.next_framebuffer_handle = FIRST_HANDLE;
        }
        h
    }

    pub(crate) fn allocate_buffer_handle(&mut self) -> u32 {
        let h = self.next_buffer_handle;
        self.next_buffer_handle = self.next_buffer_handle.saturating_add(1);
        if self.next_buffer_handle == 0 {
            self.next_buffer_handle = FIRST_HANDLE;
        }
        h
    }

    pub(crate) fn allocate_shader_handle(&mut self) -> u32 {
        let h = self.next_shader_handle;
        self.next_shader_handle = self.next_shader_handle.saturating_add(1);
        if self.next_shader_handle == 0 {
            self.next_shader_handle = FIRST_HANDLE;
        }
        h
    }

    pub(crate) fn allocate_program_handle(&mut self) -> u32 {
        let h = self.next_program_handle;
        self.next_program_handle = self.next_program_handle.saturating_add(1);
        if self.next_program_handle == 0 {
            self.next_program_handle = FIRST_HANDLE;
        }
        h
    }

    pub(crate) fn allocate_vertex_array_handle(&mut self) -> u32 {
        let h = self.next_vertex_array_handle;
        self.next_vertex_array_handle = self.next_vertex_array_handle.saturating_add(1);
        if self.next_vertex_array_handle == 0 {
            self.next_vertex_array_handle = FIRST_HANDLE;
        }
        h
    }

    pub(crate) fn allocate_renderbuffer_handle(&mut self) -> u32 {
        let h = self.next_renderbuffer_handle;
        self.next_renderbuffer_handle = self.next_renderbuffer_handle.saturating_add(1);
        if self.next_renderbuffer_handle == 0 {
            self.next_renderbuffer_handle = FIRST_HANDLE;
        }
        h
    }

    #[allow(dead_code)]
    pub(crate) fn fetch_vertex_attributes(
        &self,
        vertex_id: u32,
        instance_id: u32,
        dest: &mut [u32],
    ) {
        Self::fetch_vertex_attributes_static(
            &self.vertex_arrays,
            self.bound_vertex_array,
            &self.buffers,
            vertex_id,
            instance_id,
            dest,
            &self.kernel,
        );
    }

    pub(crate) fn get_attribute_bindings(
        &self,
    ) -> Vec<crate::wasm_gl_emu::transfer::AttributeBinding> {
        let vao = &self.vertex_arrays[&self.bound_vertex_array];
        vao.attributes
            .iter()
            .map(|attr| {
                let (buffer_ptr, offset) = if attr.enabled {
                    let ptr = if let Some(buffer_id) = attr.buffer {
                        if let Some(buf_obj) = self.buffers.get(&buffer_id) {
                            if let Some(gpu_buf) = self.kernel.get_buffer(buf_obj.gpu_handle) {
                                gpu_buf.data.as_ptr()
                            } else {
                                std::ptr::null()
                            }
                        } else {
                            std::ptr::null()
                        }
                    } else {
                        std::ptr::null()
                    };
                    (ptr, attr.offset as usize)
                } else {
                    (std::ptr::null(), 0)
                };

                let type_size = match attr.type_ {
                    0x1401 | 0x1400 => 1, // BYTE, UNSIGNED_BYTE
                    0x1403 | 0x1402 => 2, // SHORT, UNSIGNED_SHORT
                    _ => 4,               // FLOAT, INT, UNSIGNED_INT
                };

                crate::wasm_gl_emu::transfer::AttributeBinding {
                    buffer_ptr,
                    type_: attr.type_,
                    size: attr.size,
                    normalized: attr.normalized,
                    is_integer: attr.is_integer,
                    offset,
                    stride: if attr.stride == 0 {
                        (attr.size as usize) * type_size
                    } else {
                        attr.stride as usize
                    },
                    type_size,
                    divisor: attr.divisor,
                    default_value: attr.default_value,
                }
            })
            .collect()
    }

    pub(crate) fn fetch_vertex_attributes_static(
        vertex_arrays: &HashMap<u32, VertexArray>,
        bound_vertex_array: u32,
        buffers: &HashMap<u32, Buffer>,
        vertex_id: u32,
        instance_id: u32,
        dest: &mut [u32],
        kernel: &crate::wasm_gl_emu::device::GpuKernel,
    ) {
        let vao = match vertex_arrays.get(&bound_vertex_array) {
            Some(v) => v,
            None => return,
        };

        let bindings: Vec<_> = vao
            .attributes
            .iter()
            .map(|attr| {
                let (buffer_ptr, offset) = if attr.enabled {
                    let ptr = if let Some(buffer_id) = attr.buffer {
                        if let Some(buf_obj) = buffers.get(&buffer_id) {
                            if let Some(gpu_buf) = kernel.get_buffer(buf_obj.gpu_handle) {
                                gpu_buf.data.as_ptr()
                            } else {
                                std::ptr::null()
                            }
                        } else {
                            std::ptr::null()
                        }
                    } else {
                        std::ptr::null()
                    };
                    (ptr, attr.offset as usize)
                } else {
                    (std::ptr::null(), 0)
                };

                let type_size = match attr.type_ {
                    0x1401 | 0x1400 => 1,
                    0x1403 | 0x1402 => 2,
                    _ => 4,
                };

                crate::wasm_gl_emu::transfer::AttributeBinding {
                    buffer_ptr,
                    type_: attr.type_,
                    size: attr.size,
                    normalized: attr.normalized,
                    is_integer: attr.is_integer,
                    offset,
                    stride: if attr.stride == 0 {
                        (attr.size as usize) * type_size
                    } else {
                        attr.stride as usize
                    },
                    type_size,
                    divisor: attr.divisor,
                    default_value: attr.default_value,
                }
            })
            .collect();

        crate::wasm_gl_emu::transfer::TransferEngine::fetch_vertex_batch(
            &bindings,
            vertex_id,
            instance_id,
            dest,
        );
    }

    pub(crate) fn prepare_texture_metadata(&self, dest_ptr: u32) {
        let mut bindings = Vec::with_capacity(self.texture_units.len());
        for tex_handle in &self.texture_units {
            let binding = if let Some(h) = tex_handle {
                if let Some(tex) = self.textures.get(h) {
                    if let Some(level0) = tex.levels.get(&0) {
                        Some(crate::wasm_gl_emu::device::TextureBinding {
                            width: level0.width,
                            height: level0.height,
                            depth: level0.depth,
                            format: level0.internal_format,
                            bytes_per_pixel: get_bytes_per_pixel(level0.internal_format),
                            wrap_s: tex.wrap_s,
                            wrap_t: tex.wrap_t,
                            wrap_r: tex.wrap_r,
                            gpu_handle: level0.gpu_handle,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            bindings.push(binding);
        }

        self.kernel.write_texture_metadata(&bindings, dest_ptr);
    }

    pub(crate) fn get_color_attachment_info(&self, read: bool) -> (GpuHandle, u32, u32, u32) {
        let fb_handle = if read {
            self.bound_read_framebuffer
        } else {
            self.bound_draw_framebuffer
        };
        if let Some(fb_handle) = fb_handle {
            if let Some(fb) = self.framebuffers.get(&fb_handle) {
                match fb.color_attachment {
                    Some(Attachment::Texture(tex_handle)) => {
                        if let Some(tex) = self.textures.get(&tex_handle) {
                            if let Some(level0) = tex.levels.get(&0) {
                                return (
                                    level0.gpu_handle,
                                    level0.width,
                                    level0.height,
                                    level0.internal_format,
                                );
                            }
                        }
                    }
                    Some(Attachment::Renderbuffer(rb_handle)) => {
                        if let Some(rb) = self.renderbuffers.get(&rb_handle) {
                            return (rb.gpu_handle, rb.width, rb.height, rb.internal_format);
                        }
                    }
                    None => {}
                }
            }
            (GpuHandle::invalid(), 0, 0, 0)
        } else {
            (
                self.default_framebuffer.gpu_handle,
                self.default_framebuffer.width,
                self.default_framebuffer.height,
                self.default_framebuffer.internal_format,
            )
        }
    }

    pub(crate) fn get_buffer_handle_for_target(&self, target: u32) -> Option<u32> {
        if target == GL_ELEMENT_ARRAY_BUFFER {
            return self
                .vertex_arrays
                .get(&self.bound_vertex_array)
                .and_then(|vao| vao.element_array_buffer);
        }
        self.buffer_bindings.get(&target).cloned().flatten()
    }
}

/// Get bytes per pixel for a given internal format
pub(crate) fn get_bytes_per_pixel(internal_format: u32) -> u32 {
    gl_to_wgt_format(internal_format)
        .block_copy_size(None)
        .unwrap_or(4)
}

/// Map GL internal format to wgt::TextureFormat
pub(crate) fn gl_to_wgt_format(internal_format: u32) -> wgpu_types::TextureFormat {
    match internal_format {
        GL_R32F => wgpu_types::TextureFormat::R32Float,
        GL_RG32F => wgpu_types::TextureFormat::Rg32Float,
        GL_RGBA32F => wgpu_types::TextureFormat::Rgba32Float,
        GL_RGBA8 | GL_RGB8 | GL_RGBA | GL_RGB => wgpu_types::TextureFormat::Rgba8Unorm,
        GL_RED => wgpu_types::TextureFormat::R8Unorm,
        GL_RG => wgpu_types::TextureFormat::Rg8Unorm,

        // Packed 16-bit formats.
        // WebGL2 requires these. We store them in distinct formats to tell them apart in the HAL.
        // Since Bgr565, Rgba4 and Rgb5a1 are missing in this wgpu version,
        // we use R16Uint, Rg8Uint and R16Sint as "unique placeholders".
        // They all have 2 bytes per pixel.
        GL_RGBA4 => wgpu_types::TextureFormat::Rg8Uint,
        GL_RGB565 => wgpu_types::TextureFormat::R16Uint,
        GL_RGB5_A1 => wgpu_types::TextureFormat::R16Sint,

        GL_DEPTH_COMPONENT16 => wgpu_types::TextureFormat::Depth16Unorm,
        GL_DEPTH_COMPONENT24 => wgpu_types::TextureFormat::Depth24Plus,
        GL_DEPTH_COMPONENT32F => wgpu_types::TextureFormat::Depth32Float,
        GL_DEPTH24_STENCIL8 => wgpu_types::TextureFormat::Depth24PlusStencil8,
        GL_STENCIL_INDEX8 => wgpu_types::TextureFormat::Stencil8,

        _ => wgpu_types::TextureFormat::Rgba8Unorm,
    }
}
