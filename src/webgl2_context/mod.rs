pub mod types;
pub mod registry;
pub mod state;
pub mod textures;
pub mod buffers;
pub mod framebuffers;
pub mod shaders;
pub mod vaos;
pub mod drawing;
pub mod renderbuffers;

pub use registry::{
    create_context, destroy_context, wasm_alloc, wasm_free, set_last_error, last_error_ptr,
    last_error_len,
};
pub use state::*;
pub use textures::*;
pub use buffers::*;
pub use framebuffers::*;
pub use shaders::*;
pub use vaos::*;
pub use drawing::*;
pub use renderbuffers::*;
pub use types::{
    ERR_OK, ERR_INVALID_HANDLE, ERR_OOM, ERR_INVALID_ARGS, ERR_NOT_IMPLEMENTED, ERR_GL,
    ERR_INTERNAL, ERR_INVALID_OPERATION, ERR_INVALID_ENUM, GL_NO_ERROR, GL_INVALID_ENUM,
    GL_INVALID_VALUE, GL_INVALID_OPERATION, GL_OUT_OF_MEMORY, GL_ARRAY_BUFFER,
    GL_ELEMENT_ARRAY_BUFFER, GL_COMPILE_STATUS, GL_LINK_STATUS, GL_SHADER_TYPE, GL_DELETE_STATUS,
    GL_INFO_LOG_LENGTH, GL_ATTACHED_SHADERS, GL_ACTIVE_UNIFORMS, GL_ACTIVE_ATTRIBUTES,
    GL_TEXTURE_MAG_FILTER, GL_TEXTURE_MIN_FILTER, GL_TEXTURE_WRAP_S, GL_TEXTURE_WRAP_T,
    GL_VIEWPORT, GL_COLOR_CLEAR_VALUE, GL_BUFFER_SIZE, GL_COLOR_BUFFER_BIT,
    GL_RENDERBUFFER, GL_FRAMEBUFFER, GL_DEPTH_COMPONENT16, GL_DEPTH_STENCIL,
    GL_RGBA4, GL_RGB565, GL_RGB5_A1, GL_STENCIL_INDEX8,
};
