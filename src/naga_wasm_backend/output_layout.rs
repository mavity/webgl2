//! Shader output layout calculations
//!
//! This module provides centralized logic for determining where shader outputs
//! should be written in memory. It implements the specification from
//! docs/1.7-shaders-returns.md.

use naga::{Binding, BuiltIn, ShaderStage};

/// Memory pointer global indices:
/// - 0: ATTR_PTR_GLOBAL (Vertex attributes)
/// - 1: UNIFORM_PTR_GLOBAL (Uniform data)
/// - 2: VARYING_PTR_GLOBAL (Inter-stage varyings)
/// - 3: PRIVATE_PTR_GLOBAL (Fragment outputs & private variables)
/// - 4: TEXTURE_PTR_GLOBAL (Texture references)
/// - 5: FRAME_SP_GLOBAL (Frame stack pointer for function calls)
pub const ATTR_PTR_GLOBAL: u32 = 0;
pub const UNIFORM_PTR_GLOBAL: u32 = 1;
pub const VARYING_PTR_GLOBAL: u32 = 2;
pub const PRIVATE_PTR_GLOBAL: u32 = 3;
pub const TEXTURE_PTR_GLOBAL: u32 = 4;
pub const FRAME_SP_GLOBAL: u32 = 5;

/// Region offsets within the scratch memory.
/// Each region is allocated 16KB for safe overflow-free access.
pub const ATTR_PTR_OFFSET: u32 = 0x0000;
pub const UNIFORM_PTR_OFFSET: u32 = 0x4000;
pub const VARYING_PTR_OFFSET: u32 = 0x8000;
pub const PRIVATE_PTR_OFFSET: u32 = 0xC000;
pub const TEXTURE_PTR_OFFSET: u32 = 0x10000;
pub const FRAME_STACK_OFFSET: u32 = 0x20000;

// Texture Descriptor Offsets (matches WebGPU backend metadata structure)
pub const TEX_WIDTH_OFFSET: u64 = 0;
pub const TEX_HEIGHT_OFFSET: u64 = 4;
pub const TEX_DATA_PTR_OFFSET: u64 = 8;
pub const TEX_DEPTH_OFFSET: u64 = 12;
pub const TEX_FORMAT_OFFSET: u64 = 16;
pub const TEX_BPP_OFFSET: u64 = 20;
pub const TEX_WRAP_S_OFFSET: u64 = 24;
pub const TEX_WRAP_T_OFFSET: u64 = 28;
pub const TEX_WRAP_R_OFFSET: u64 = 32;
pub const TEX_LAYOUT_OFFSET: u64 = 36;
pub const TEX_MIN_FILTER_OFFSET: u64 = 40;
pub const TEX_MAG_FILTER_OFFSET: u64 = 44;

/// Frame stack configuration.
pub const FRAME_STACK_SIZE: u32 = 0x20000; // 128KB size

/// Context Block configuration for uniforms and handles.
pub const MAX_BINDINGS_PER_GROUP: u32 = 16;
pub const MAX_GROUPS: u32 = 4;
pub const CONTEXT_BLOCK_SIZE: u32 = MAX_GROUPS * MAX_BINDINGS_PER_GROUP * 4; // 64 bindings * 4 bytes = 256 bytes
pub const BINDING_POINTER_SIZE: u32 = 4;

/// Dynamic layout of scratch memory regions.
#[derive(Debug, Clone, Copy)]
pub struct ScratchLayout {
    pub base: u32,
    pub attr_ptr: u32,
    pub uniform_ptr: u32,
    pub varying_ptr: u32,
    pub private_ptr: u32,
    pub texture_ptr: u32,
    pub frame_stack_base: u32,
}

impl ScratchLayout {
    pub fn new(base: u32) -> Self {
        Self {
            base,
            attr_ptr: base + ATTR_PTR_OFFSET,
            uniform_ptr: base + UNIFORM_PTR_OFFSET,
            varying_ptr: base + VARYING_PTR_OFFSET,
            private_ptr: base + PRIVATE_PTR_OFFSET,
            texture_ptr: base + TEXTURE_PTR_OFFSET,
            frame_stack_base: base + FRAME_STACK_OFFSET,
        }
    }
}

/// Compute the memory destination for a shader output binding.
#[inline]
pub fn compute_output_destination(binding: &Binding, stage: ShaderStage) -> (u32, u32) {
    match binding {
        Binding::BuiltIn(BuiltIn::Position { .. }) => (0, VARYING_PTR_GLOBAL),
        Binding::BuiltIn(BuiltIn::PointSize) => (16, VARYING_PTR_GLOBAL),
        Binding::Location { location, .. } => match stage {
            ShaderStage::Vertex => ((location + 2) * 16, VARYING_PTR_GLOBAL),
            ShaderStage::Fragment => (location * 16, PRIVATE_PTR_GLOBAL),
            _ => (0, 0),
        },
        _ => (0, 0),
    }
}

/// Compute the memory offset for reading shader input arguments.
#[inline]
pub fn compute_input_offset(location: u32, stage: ShaderStage) -> (u32, u32) {
    match stage {
        ShaderStage::Vertex => (location * 64, ATTR_PTR_GLOBAL),
        ShaderStage::Fragment => ((location + 2) * 16, VARYING_PTR_GLOBAL),
        _ => (0, 0),
    }
}

/// Returns the index into the context block for a (group, binding) pair.
#[inline]
pub fn get_context_block_index(group: u32, binding: u32) -> u32 {
    (group * MAX_BINDINGS_PER_GROUP + binding) * BINDING_POINTER_SIZE
}

/// Returns the index into the context block for a WebGL uniform location.
#[inline]
pub fn get_webgl_context_block_index(location: u32) -> u32 {
    get_context_block_index(0, location)
}

/// Returns the data offset for a WebGL uniform location.
#[inline]
pub fn get_webgl_uniform_data_offset(location: u32) -> u32 {
    CONTEXT_BLOCK_SIZE + location * 64
}

#[inline]
pub fn compute_texture_offset(location: u32) -> (u32, u32) {
    (get_webgl_context_block_index(location), TEXTURE_PTR_GLOBAL)
}

#[inline]
pub fn compute_uniform_offset(location: u32) -> (u32, u32) {
    (get_webgl_context_block_index(location), UNIFORM_PTR_GLOBAL)
}

/// Description of the data layout within a uniform slot
#[derive(Debug, Clone, Copy)]
pub enum UniformLayout {
    Scalar,
    Vector(u32),
    Matrix(u32),
    Struct,
}

/// Build a map of resource (group, binding) to its memory offset and layout
pub fn get_uniform_map(
    module: &naga::Module,
    _info: &naga::valid::ModuleInfo,
    _stage: naga::ShaderStage,
) -> std::collections::HashMap<(u32, u32), (u32, UniformLayout)> {
    use std::collections::HashMap;
    let mut map = HashMap::new();

    for (_, var) in module.global_variables.iter() {
        if let Some(rb) = &var.binding {
            let group = rb.group;
            let binding = rb.binding;
            if var.space == naga::AddressSpace::Uniform || var.space == naga::AddressSpace::Handle {
                let resource_index = group * MAX_BINDINGS_PER_GROUP + binding;
                let context_offset = resource_index * 4;

                let layout = match &module.types[var.ty].inner {
                    naga::TypeInner::Scalar { .. } => UniformLayout::Scalar,
                    naga::TypeInner::Vector { size, .. } => UniformLayout::Vector(*size as u32),
                    naga::TypeInner::Matrix { columns, .. } => {
                        UniformLayout::Matrix(*columns as u32)
                    }
                    naga::TypeInner::Image { .. } | naga::TypeInner::Sampler { .. } => {
                        UniformLayout::Scalar
                    }
                    _ => UniformLayout::Struct,
                };

                map.insert((group, binding), (context_offset, layout));
            }
        }
    }
    map
}

/// Validate that a binding is supported for the given shader stage.
pub fn is_binding_valid(binding: &Binding, stage: ShaderStage) -> bool {
    match (binding, stage) {
        (Binding::BuiltIn(BuiltIn::Position { .. }), ShaderStage::Vertex) => true,
        (Binding::BuiltIn(BuiltIn::FragDepth), ShaderStage::Fragment) => true,
        (Binding::BuiltIn(BuiltIn::PointSize), ShaderStage::Vertex) => true,
        (Binding::Location { .. }, ShaderStage::Vertex | ShaderStage::Fragment) => true,
        _ => false,
    }
}
