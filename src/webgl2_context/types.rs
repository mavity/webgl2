use std::cell::RefCell;
use std::collections::HashMap;
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

pub const GL_COMPILE_STATUS: u32 = 0x8B81;
pub const GL_LINK_STATUS: u32 = 0x8B82;
pub const GL_SHADER_TYPE: u32 = 0x8B4F;
pub const GL_DELETE_STATUS: u32 = 0x8B80;
pub const GL_INFO_LOG_LENGTH: u32 = 0x8B84;
pub const GL_ATTACHED_SHADERS: u32 = 0x8B85;
pub const GL_ACTIVE_UNIFORMS: u32 = 0x8B86;
pub const GL_ACTIVE_ATTRIBUTES: u32 = 0x8B89;

pub const GL_BYTE: u32 = 0x1400;
pub const GL_UNSIGNED_BYTE: u32 = 0x1401;
pub const GL_SHORT: u32 = 0x1402;
pub const GL_UNSIGNED_SHORT: u32 = 0x1403;
pub const GL_INT: u32 = 0x1404;
pub const GL_UNSIGNED_INT: u32 = 0x1405;
pub const GL_FLOAT: u32 = 0x1406;

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
pub const GL_RGBA8: u32 = 0x8058;
pub const GL_STENCIL_INDEX8: u32 = 0x8D48;

// Handle constants
pub(crate) const INVALID_HANDLE: u32 = 0;
pub(crate) const FIRST_HANDLE: u32 = 1;

// Last error buffer (thread-local would be better, but we're single-threaded WASM)
thread_local! {
    pub(crate) static LAST_ERROR: RefCell<String> = const { RefCell::new(String::new()) };
}

/// A WebGL2 texture resource
#[derive(Clone)]
pub(crate) struct Texture {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) data: Vec<u8>, // RGBA u8
    pub(crate) min_filter: u32,
    pub(crate) mag_filter: u32,
    pub(crate) wrap_s: u32,
    pub(crate) wrap_t: u32,
}

/// A WebGL2 renderbuffer resource
#[derive(Clone)]
pub(crate) struct Renderbuffer {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) internal_format: u32,
    pub(crate) data: Vec<u8>,
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
    pub(crate) data: Vec<u8>,
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
pub(crate) struct Program {
    pub(crate) attached_shaders: Vec<u32>,
    pub(crate) linked: bool,
    pub(crate) info_log: String,
    pub(crate) attributes: HashMap<String, i32>,
    pub(crate) attribute_bindings: HashMap<String, u32>,
    pub(crate) uniforms: HashMap<String, i32>,
    pub(crate) vs_module: Option<Arc<naga::Module>>,
    pub(crate) fs_module: Option<Arc<naga::Module>>,
    pub(crate) vs_info: Option<Arc<naga::valid::ModuleInfo>>,
    pub(crate) fs_info: Option<Arc<naga::valid::ModuleInfo>>,
    pub(crate) vs_wasm: Option<Vec<u8>>,
    pub(crate) fs_wasm: Option<Vec<u8>>,
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
    pub(crate) bound_framebuffer: Option<u32>,
    pub(crate) bound_renderbuffer: Option<u32>,
    pub(crate) bound_array_buffer: Option<u32>,
    pub(crate) bound_vertex_array: u32,
    pub(crate) current_program: Option<u32>,

    pub(crate) uniform_data: Vec<u8>,

    // Software rendering state
    pub default_framebuffer: crate::wasm_gl_emu::OwnedFramebuffer,
    pub rasterizer: crate::wasm_gl_emu::Rasterizer,

    // State
    pub(crate) clear_color: [f32; 4],
    pub(crate) viewport: (i32, i32, u32, u32),
    pub(crate) scissor_box: (i32, i32, u32, u32),
    pub(crate) scissor_test_enabled: bool,
    pub(crate) depth_test_enabled: bool,
    pub(crate) depth_func: u32,
    pub(crate) blend_enabled: bool,
    pub(crate) active_texture_unit: u32,
    pub(crate) texture_units: Vec<Option<u32>>,
    pub(crate) gl_error: u32,
    pub verbosity: u32, // 0: None, 1: Error, 2: Info, 3: Debug
}

impl Default for Context {
    fn default() -> Self {
        let mut vertex_arrays = HashMap::new();
        vertex_arrays.insert(0, VertexArray::default());

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
            bound_framebuffer: None,
            bound_renderbuffer: None,
            bound_array_buffer: None,
            bound_vertex_array: 0,
            current_program: None,

            uniform_data: vec![0; 4096], // 4KB for uniforms

            default_framebuffer: crate::wasm_gl_emu::OwnedFramebuffer::new(640, 480),
            rasterizer: crate::wasm_gl_emu::Rasterizer::default(),

            clear_color: [0.0, 0.0, 0.0, 0.0],
            viewport: (0, 0, 640, 480),
            scissor_box: (0, 0, 640, 480),
            scissor_test_enabled: false,
            depth_test_enabled: false,
            depth_func: 0x0203, // GL_LEQUAL
            blend_enabled: false,
            active_texture_unit: 0,
            texture_units: vec![None; 16],
            gl_error: GL_NO_ERROR,
            verbosity: 2, // Default to Info
        }
    }
}

impl Context {
    pub(crate) fn log(&self, level: u32, s: &str) {
        if level <= self.verbosity {
            crate::js_log(level, s);
        }
    }

    pub(crate) fn log_static(verbosity: u32, level: u32, s: &str) {
        if level <= verbosity {
            crate::js_log(level, s);
        }
    }

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
        );
    }

    pub(crate) fn fetch_vertex_attributes_static(
        vertex_arrays: &HashMap<u32, VertexArray>,
        bound_vertex_array: u32,
        buffers: &HashMap<u32, Buffer>,
        vertex_id: u32,
        instance_id: u32,
        dest: &mut [u32],
    ) {
        let vao = match vertex_arrays.get(&bound_vertex_array) {
            Some(v) => v,
            None => return, // Should not happen as default VAO is always there
        };

        for (i, attr) in vao.attributes.iter().enumerate() {
            let base_idx = i * 4;
            if !attr.enabled {
                dest[base_idx..base_idx + 4].copy_from_slice(&attr.default_value);
                continue;
            }

            // Calculate effective index based on divisor
            let effective_index = if attr.divisor == 0 {
                vertex_id
            } else {
                instance_id / attr.divisor
            };

            if let Some(buffer_id) = attr.buffer {
                if let Some(buffer) = buffers.get(&buffer_id) {
                    let type_size = match attr.type_ {
                        0x1400 => 1, // GL_BYTE
                        0x1401 => 1, // GL_UNSIGNED_BYTE
                        0x1402 => 2, // GL_SHORT
                        0x1403 => 2, // GL_UNSIGNED_SHORT
                        0x1404 => 4, // GL_INT
                        0x1405 => 4, // GL_UNSIGNED_INT
                        0x1406 => 4, // GL_FLOAT
                        _ => 4,
                    };

                    let effective_stride = if attr.stride == 0 {
                        attr.size * type_size
                    } else {
                        attr.stride
                    };

                    let offset = attr.offset as usize
                        + (effective_index as usize * effective_stride as usize);

                    for component in 0..4 {
                        if component < attr.size as usize {
                            let src_off = offset + component * type_size as usize;
                            if src_off + type_size as usize <= buffer.data.len() {
                                let bits = if attr.is_integer {
                                    // Pure integer attribute
                                    match attr.type_ {
                                        0x1400 => (buffer.data[src_off] as i8) as i32 as u32,
                                        0x1401 => buffer.data[src_off] as u32,
                                        0x1402 => i16::from_le_bytes(
                                            buffer.data[src_off..src_off + 2].try_into().unwrap(),
                                        ) as i32
                                            as u32,
                                        0x1403 => u16::from_le_bytes(
                                            buffer.data[src_off..src_off + 2].try_into().unwrap(),
                                        ) as u32,
                                        0x1404 => u32::from_le_bytes(
                                            buffer.data[src_off..src_off + 4].try_into().unwrap(),
                                        ),
                                        0x1405 => u32::from_le_bytes(
                                            buffer.data[src_off..src_off + 4].try_into().unwrap(),
                                        ),
                                        _ => 0,
                                    }
                                } else {
                                    // Float or normalized attribute
                                    let val: f32 = match attr.type_ {
                                        0x1406 => f32::from_le_bytes(
                                            buffer.data[src_off..src_off + 4]
                                                .try_into()
                                                .unwrap_or([0; 4]),
                                        ),
                                        0x1400 => {
                                            // GL_BYTE
                                            let v = (buffer.data[src_off] as i8) as f32;
                                            if attr.normalized {
                                                // Signed byte normalization: max(c / 127.0, -1.0)
                                                v / 127.0
                                            } else {
                                                v
                                            }
                                        }
                                        0x1401 => {
                                            // GL_UNSIGNED_BYTE
                                            let v = buffer.data[src_off] as f32;
                                            if attr.normalized {
                                                v / 255.0
                                            } else {
                                                v
                                            }
                                        }
                                        0x1402 => {
                                            // GL_SHORT
                                            let v = i16::from_le_bytes(
                                                buffer.data[src_off..src_off + 2]
                                                    .try_into()
                                                    .unwrap(),
                                            )
                                                as f32;
                                            if attr.normalized {
                                                v / 32767.0
                                            } else {
                                                v
                                            }
                                        }
                                        0x1403 => {
                                            // GL_UNSIGNED_SHORT
                                            let v = u16::from_le_bytes(
                                                buffer.data[src_off..src_off + 2]
                                                    .try_into()
                                                    .unwrap(),
                                            )
                                                as f32;
                                            if attr.normalized {
                                                v / 65535.0
                                            } else {
                                                v
                                            }
                                        }
                                        0x1404 => {
                                            // GL_INT
                                            let v = i32::from_le_bytes(
                                                buffer.data[src_off..src_off + 4]
                                                    .try_into()
                                                    .unwrap(),
                                            )
                                                as f32;
                                            if attr.normalized {
                                                v / 2147483647.0
                                            } else {
                                                v
                                            }
                                        }
                                        0x1405 => {
                                            // GL_UNSIGNED_INT
                                            let v = u32::from_le_bytes(
                                                buffer.data[src_off..src_off + 4]
                                                    .try_into()
                                                    .unwrap(),
                                            )
                                                as f32;
                                            if attr.normalized {
                                                v / 4294967295.0
                                            } else {
                                                v
                                            }
                                        }
                                        _ => 0.0,
                                    };
                                    val.to_bits()
                                };
                                dest[base_idx + component] = bits;
                            } else {
                                dest[base_idx + component] = 0;
                            }
                        } else {
                            // Fill remaining components with default (0,0,0,1)
                            // For integer attributes, default is (0, 0, 0, 1)
                            // For float attributes, default is (0.0, 0.0, 0.0, 1.0)
                            // We store defaults in attr.default_value which are already bits
                            // But wait, default_value is [u32; 4].
                            // If we are filling from buffer, we should use 0/1 appropriate for the type?
                            // Actually, if size < 4, we should use defaults.
                            // But usually defaults are (0,0,0,1).
                            // Let's use the default_value from the attribute if we want to be correct,
                            // but standard says (0,0,0,1).
                            // The attr.default_value is used when !enabled.
                            // When enabled but component missing, it's (0,0,0,1).
                            if component == 3 {
                                dest[base_idx + component] =
                                    if attr.is_integer { 1 } else { 1.0f32.to_bits() };
                            } else {
                                dest[base_idx + component] = 0;
                            }
                        }
                    }
                } else {
                    dest[base_idx..base_idx + 4].copy_from_slice(&attr.default_value);
                }
            } else {
                dest[base_idx..base_idx + 4].copy_from_slice(&attr.default_value);
            }
        }
    }

    pub(crate) fn prepare_texture_metadata(&self, dest_ptr: u32) {
        for (i, tex_handle) in self.texture_units.iter().enumerate() {
            let offset = i * 32;
            if let Some(h) = tex_handle {
                if let Some(tex) = self.textures.get(h) {
                    unsafe {
                        let base = (dest_ptr + offset as u32) as *mut i32;
                        *base.offset(0) = tex.width as i32;
                        *base.offset(1) = tex.height as i32;
                        *base.offset(2) = tex.data.as_ptr() as i32;
                        // Rest is padding for now
                    }
                }
            }
        }
    }
}
