pub mod buffers;
pub mod drawing;
pub mod framebuffers;
pub mod registry;
pub mod renderbuffers;
pub mod shaders;
pub mod state;
pub mod textures;
pub mod types;
pub mod vaos;

pub use buffers::*;
pub use drawing::*;
pub use framebuffers::*;
pub use registry::{
    create_context, destroy_context, last_error_len, last_error_ptr, set_last_error, wasm_alloc,
    wasm_free,
};
pub use renderbuffers::*;
pub use shaders::*;
pub use state::*;
pub use textures::*;
pub use types::{
    ERR_GL, ERR_INTERNAL, ERR_INVALID_ARGS, ERR_INVALID_ENUM, ERR_INVALID_HANDLE,
    ERR_INVALID_OPERATION, ERR_NOT_IMPLEMENTED, ERR_OK, ERR_OOM, GL_ACTIVE_ATTRIBUTES,
    GL_ACTIVE_UNIFORMS, GL_ARRAY_BUFFER, GL_ATTACHED_SHADERS, GL_BUFFER_SIZE, GL_COLOR_BUFFER_BIT,
    GL_COLOR_CLEAR_VALUE, GL_COMPILE_STATUS, GL_DELETE_STATUS, GL_DEPTH_COMPONENT16,
    GL_DEPTH_STENCIL, GL_ELEMENT_ARRAY_BUFFER, GL_FRAMEBUFFER, GL_INFO_LOG_LENGTH, GL_INVALID_ENUM,
    GL_INVALID_OPERATION, GL_INVALID_VALUE, GL_LINK_STATUS, GL_NO_ERROR, GL_OUT_OF_MEMORY,
    GL_RENDERBUFFER, GL_RGB565, GL_RGB5_A1, GL_RGBA4, GL_SHADER_TYPE, GL_STENCIL_INDEX8,
    GL_TEXTURE_MAG_FILTER, GL_TEXTURE_MIN_FILTER, GL_TEXTURE_WRAP_S, GL_TEXTURE_WRAP_T,
    GL_VIEWPORT,
};
pub use vaos::*;
