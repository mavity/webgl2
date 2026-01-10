//! Shader output layout calculations
//!
//! This module provides centralized logic for determining where shader outputs
//! should be written in memory. It implements the specification from
//! docs/1.7-shaders-returns.md.
//!
//! # Memory Layout
//!
//! ## Vertex Shader Outputs
//! - **Position**: `varying_ptr + 0` (vec4, 16 bytes)
//! - **Location(n)**: `varying_ptr + (n+1)*16` bytes
//! - **PointSize**: Part of position handling or location slot
//!
//! ## Fragment Shader Outputs
//! - **Location(n)**: `private_ptr + n*16` bytes (color outputs, up to ~1KB for 64 targets)
//! - **FragDepth**: `private_ptr + 0x1000` (f32, 4 bytes at offset 4096)
//!
//! The FragDepth offset is intentionally placed at 4KB to avoid any collision with
//! color outputs, allowing shaders to safely output both colors and depth.
//!
//! # Design Rationale
//!
//! ## Slot Sizes
//!
//! **64-byte slots** for attributes and uniforms:
//! - Accommodates up to mat4 (16 floats * 4 bytes = 64 bytes)
//! - Matches WebGL''s vertex attribute stride limits (max 255 bytes, but 64 is practical)
//! - Provides natural alignment for all GLSL/WGSL types
//! - Simplifies pointer arithmetic (power of 2 minus one bit)
//!
//! **16-byte slots** for varyings and fragment outputs:
//! - Aligns with vec4 size (the largest interpolatable type)
//! - Fragment outputs are always scalar/vec types, never matrices
//! - Reduces memory footprint for inter-stage communication
//! - GPU-friendly alignment (128-bit boundaries)
//!
//! ## Offset Strategy
//!
//! **`(location + 1) * 16`** for vertex shader varyings:
//! - Reserves offset 0 for `@builtin(position)` (mandatory vertex output)
//! - User-defined varyings start at offset 16, avoiding position collision
//! - Simplifies fragment shader varying reads (same offsets, no translation needed)
//!
//! ## Memory Pointer Indices
//!
//! - **0 (ATTR_PTR_GLOBAL)**: Vertex shader input attributes
//! - **1 (UNIFORM_PTR_GLOBAL)**: Uniform data (read-only, both stages)
//! - **2 (VARYING_PTR_GLOBAL)**: Inter-stage communication (VS writes, FS reads)
//! - **3 (PRIVATE_PTR_GLOBAL)**: Fragment shader outputs (colors, depth)
//!
//! ## Future Optimization
//!
//! The layout uses fixed slots for simplicity and compatibility.
//! A future "Deterministic Packed Layout" could reduce memory usage by:
//! - Packing scalar/vec2 types into smaller slots
//! - Eliminating padding between struct members
//! - Computing exact sizes from Naga type information
//!
//! Current approach provides:
//! - Predictable offsets (no runtime calculation)
//! - Natural alignment (no unaligned access)
//! - Simple implementation (minimal complexity)
//! - Zero-cost abstraction (inlined functions)

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

/// Frame stack memory layout.
/// The frame stack is used for LIFO allocation of function call frames.
/// Base address is chosen to avoid collision with other memory regions.
///
/// Memory layout (640KB total, 10 pages):
/// - 0x00000 - 0x7FFFF: General shader memory (attributes, uniforms, varyings, private, textures)
/// - 0x80000 - 0x9FFFF: Frame stack (128KB dedicated region)
pub const FRAME_STACK_BASE: u32 = 0x80000; // 512KB offset
pub const FRAME_STACK_SIZE: u32 = 0x20000; // 128KB size

/// Compute the memory destination for a shader output binding.
///
/// # Parameters
/// - `binding`: The Naga binding describing where the output goes
/// - `stage`: The shader stage (Vertex or Fragment)
///
/// # Returns
/// A tuple of `(byte_offset, global_pointer_index)` where:
/// - `byte_offset`: Offset in bytes from the base pointer
/// - `global_pointer_index`: Which memory pointer global to use (2=varying, 3=private)
///
/// # Examples
///
/// ``ignore
/// // Vertex shader position output
/// let (offset, ptr) = compute_output_destination(
///     &Binding::BuiltIn(BuiltIn::Position),
///     ShaderStage::Vertex
/// );
/// assert_eq!(offset, 0);
/// assert_eq!(ptr, VARYING_PTR_GLOBAL);
///
/// // Fragment shader color output at location 0
/// let (offset, ptr) = compute_output_destination(
///     &Binding::Location { location: 0, interpolation: None, sampling: None },
///     ShaderStage::Fragment
/// );
/// assert_eq!(offset, 0);
/// assert_eq!(ptr, PRIVATE_PTR_GLOBAL);
/// ``
#[inline]
pub fn compute_output_destination(binding: &Binding, stage: ShaderStage) -> (u32, u32) {
    match binding {
        // Position is always at offset 0 in the varying buffer
        Binding::BuiltIn(BuiltIn::Position { .. }) => (0, VARYING_PTR_GLOBAL),

        // FragDepth goes to a dedicated offset in private buffer
        // IMPORTANT: FragDepth is placed at a high offset (0x1000 = 4096) to avoid
        // collisions with color outputs. This allows both FragDepth and color targets
        // to coexist in the same shader.
        // Color outputs: private_ptr + 0, 16, 32, ... (up to ~1024 bytes for 64 color targets)
        // FragDepth: private_ptr + 4096
        Binding::BuiltIn(BuiltIn::FragDepth) => (0x1000, PRIVATE_PTR_GLOBAL),

        // PointSize could be handled here if needed
        Binding::BuiltIn(BuiltIn::PointSize) => {
            // PointSize is typically written as part of vertex position handling
            // For now, we place it after position (offset 16)
            (16, VARYING_PTR_GLOBAL)
        }

        // Location-based outputs are context-sensitive
        Binding::Location { location, .. } => {
            match stage {
                ShaderStage::Vertex => {
                    // Vertex varyings: skip position (offset 0) and use subsequent slots
                    // Location(0) -> offset 16, Location(1) -> offset 32, etc.
                    ((location + 1) * 16, VARYING_PTR_GLOBAL)
                }
                ShaderStage::Fragment => {
                    // Fragment outputs (colors) go to private buffer
                    // Location(0) -> offset 0, Location(1) -> offset 16, etc.
                    (location * 16, PRIVATE_PTR_GLOBAL)
                }
                ShaderStage::Compute | ShaderStage::Task | ShaderStage::Mesh => {
                    // Compute, Task, and Mesh shaders don't have location-based outputs
                    // This is an error condition - shader validation should have caught this
                    // Return (0, 0) which will be handled by the caller as "no output"
                    tracing::warn!(
                        "Location-based output binding in {:?} shader (unsupported)",
                        stage
                    );
                    (0, 0)
                }
            }
        }

        // Unknown/unsupported bindings
        // Log a warning to help with debugging, but don't panic since the shader
        // may still be valid (e.g., using a newer Naga feature we don't support yet)
        _ => {
            tracing::warn!("Unsupported binding type in shader output: {:?}", binding);
            (0, 0)
        }
    }
}

/// Compute the memory offset for reading shader input arguments.
///
/// # Parameters
/// - `location`: The location index of the input
/// - `stage`: The shader stage (Vertex or Fragment)
///
/// # Returns
/// A tuple of `(byte_offset, global_pointer_index)` where:
/// - `byte_offset`: Offset in bytes from the base pointer
/// - `global_pointer_index`: Which memory pointer global to use (0=attr, 2=varying)
///
/// # Layout
/// - **Vertex Shader Inputs (Attributes)**: `attr_ptr + location * 64`
///   - Uses 64-byte slots for attribute data
///   - Pointer global index 0 (ATTR_PTR_GLOBAL)
///
/// - **Fragment Shader Inputs (Varyings)**: `varying_ptr + (location + 1) * 16`
///   - Uses 16-byte slots, offset by 1 to skip position at offset 0
///   - Pointer global index 2 (VARYING_PTR_GLOBAL)
///
/// # Examples
///
/// ``ignore
/// // Vertex shader attribute at location 0
/// let (offset, ptr) = compute_input_offset(0, ShaderStage::Vertex);
/// assert_eq!(offset, 0);
/// assert_eq!(ptr, ATTR_PTR_GLOBAL);
///
/// // Fragment shader varying at location 0
/// let (offset, ptr) = compute_input_offset(0, ShaderStage::Fragment);
/// assert_eq!(offset, 16);
/// assert_eq!(ptr, VARYING_PTR_GLOBAL);
/// ``
#[inline]
pub fn compute_input_offset(location: u32, stage: ShaderStage) -> (u32, u32) {
    match stage {
        ShaderStage::Vertex => {
            // Vertex attributes use 64-byte slots
            (location * 64, ATTR_PTR_GLOBAL)
        }
        ShaderStage::Fragment => {
            // Fragment varyings use 16-byte slots, offset by 1 to skip position
            ((location + 1) * 16, VARYING_PTR_GLOBAL)
        }
        _ => {
            tracing::warn!("Input offset requested for unsupported stage: {:?}", stage);
            (0, 0)
        }
    }
}

/// Compute the memory offset for reading uniform data.
///
/// # Parameters
/// - `location`: The binding location of the uniform
///
/// # Returns
/// A tuple of `(byte_offset, global_pointer_index)` where:
/// - `byte_offset`: Offset in bytes from the uniform pointer
/// - `global_pointer_index`: Always UNIFORM_PTR_GLOBAL (1)
///
/// # Layout
/// Uniforms use 64-byte slots: `uniform_ptr + location * 64`
///
/// # Examples
///
/// ``ignore
/// let (offset, ptr) = compute_uniform_offset(0);
/// assert_eq!(offset, 0);
/// assert_eq!(ptr, UNIFORM_PTR_GLOBAL);
///
/// let (offset, ptr) = compute_uniform_offset(2);
/// assert_eq!(offset, 128);
/// assert_eq!(ptr, UNIFORM_PTR_GLOBAL);
/// ``
#[inline]
pub fn compute_texture_offset(location: u32) -> (u32, u32) {
    (location * 64, TEXTURE_PTR_GLOBAL)
}
pub fn compute_uniform_offset(location: u32) -> (u32, u32) {
    (location * 64, UNIFORM_PTR_GLOBAL)
}

/// Validate that a binding is supported for the given shader stage.
///
/// # Returns
/// `true` if the binding is valid for the stage, `false` otherwise.
pub fn is_binding_valid(binding: &Binding, stage: ShaderStage) -> bool {
    match (binding, stage) {
        // Position is only valid in vertex shaders
        (Binding::BuiltIn(BuiltIn::Position { .. }), ShaderStage::Vertex) => true,

        // FragDepth is only valid in fragment shaders
        (Binding::BuiltIn(BuiltIn::FragDepth), ShaderStage::Fragment) => true,

        // PointSize is only valid in vertex shaders
        (Binding::BuiltIn(BuiltIn::PointSize), ShaderStage::Vertex) => true,

        // Location-based outputs are valid in vertex and fragment shaders
        (Binding::Location { .. }, ShaderStage::Vertex | ShaderStage::Fragment) => true,

        // Everything else is invalid
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_position() {
        let (offset, ptr) = compute_output_destination(
            &Binding::BuiltIn(BuiltIn::Position { invariant: false }),
            ShaderStage::Vertex,
        );
        assert_eq!(offset, 0);
        assert_eq!(ptr, VARYING_PTR_GLOBAL);
    }

    #[test]
    fn test_vertex_varyings() {
        // Location 0 should be at offset 16 (after position)
        let (offset, ptr) = compute_output_destination(
            &Binding::Location {
                location: 0,
                interpolation: None,
                sampling: None,
            },
            ShaderStage::Vertex,
        );
        assert_eq!(offset, 16);
        assert_eq!(ptr, VARYING_PTR_GLOBAL);

        // Location 1 should be at offset 32
        let (offset, ptr) = compute_output_destination(
            &Binding::Location {
                location: 1,
                interpolation: None,
                sampling: None,
            },
            ShaderStage::Vertex,
        );
        assert_eq!(offset, 32);
        assert_eq!(ptr, VARYING_PTR_GLOBAL);
    }

    #[test]
    fn test_fragment_colors() {
        // Location 0 (first color output) at offset 0
        let (offset, ptr) = compute_output_destination(
            &Binding::Location {
                location: 0,
                interpolation: None,
                sampling: None,
            },
            ShaderStage::Fragment,
        );
        assert_eq!(offset, 0);
        assert_eq!(ptr, PRIVATE_PTR_GLOBAL);

        // Location 1 (second color output) at offset 16
        let (offset, ptr) = compute_output_destination(
            &Binding::Location {
                location: 1,
                interpolation: None,
                sampling: None,
            },
            ShaderStage::Fragment,
        );
        assert_eq!(offset, 16);
        assert_eq!(ptr, PRIVATE_PTR_GLOBAL);
    }

    #[test]
    fn test_fragment_depth() {
        let (offset, ptr) = compute_output_destination(
            &Binding::BuiltIn(BuiltIn::FragDepth),
            ShaderStage::Fragment,
        );
        assert_eq!(offset, 0x1000);
        assert_eq!(ptr, PRIVATE_PTR_GLOBAL);
    }

    #[test]
    fn test_binding_validation() {
        // Valid cases
        assert!(is_binding_valid(
            &Binding::BuiltIn(BuiltIn::Position { invariant: false }),
            ShaderStage::Vertex
        ));
        assert!(is_binding_valid(
            &Binding::BuiltIn(BuiltIn::FragDepth),
            ShaderStage::Fragment
        ));
        assert!(is_binding_valid(
            &Binding::Location {
                location: 0,
                interpolation: None,
                sampling: None,
            },
            ShaderStage::Vertex
        ));

        // Invalid cases
        assert!(!is_binding_valid(
            &Binding::BuiltIn(BuiltIn::Position { invariant: false }),
            ShaderStage::Fragment
        ));
        assert!(!is_binding_valid(
            &Binding::BuiltIn(BuiltIn::FragDepth),
            ShaderStage::Vertex
        ));
    }

    // ========== Step 1.1: New Tests for Input and Uniform Offsets ==========

    #[test]
    fn test_vertex_input_attributes() {
        // Attribute location 0
        let (offset, ptr) = compute_input_offset(0, ShaderStage::Vertex);
        assert_eq!(offset, 0);
        assert_eq!(ptr, ATTR_PTR_GLOBAL);

        // Attribute location 1 (64-byte stride)
        let (offset, ptr) = compute_input_offset(1, ShaderStage::Vertex);
        assert_eq!(offset, 64);
        assert_eq!(ptr, ATTR_PTR_GLOBAL);

        // Attribute location 5
        let (offset, ptr) = compute_input_offset(5, ShaderStage::Vertex);
        assert_eq!(offset, 320);
        assert_eq!(ptr, ATTR_PTR_GLOBAL);
    }

    #[test]
    fn test_fragment_input_varyings() {
        // Varying location 0 (skips position at 0, starts at 16)
        let (offset, ptr) = compute_input_offset(0, ShaderStage::Fragment);
        assert_eq!(offset, 16);
        assert_eq!(ptr, VARYING_PTR_GLOBAL);

        // Varying location 1 (16-byte stride)
        let (offset, ptr) = compute_input_offset(1, ShaderStage::Fragment);
        assert_eq!(offset, 32);
        assert_eq!(ptr, VARYING_PTR_GLOBAL);

        // Varying location 3
        let (offset, ptr) = compute_input_offset(3, ShaderStage::Fragment);
        assert_eq!(offset, 64);
        assert_eq!(ptr, VARYING_PTR_GLOBAL);
    }

    #[test]
    fn test_uniform_offsets() {
        // Uniform location 0
        let (offset, ptr) = compute_uniform_offset(0);
        assert_eq!(offset, 0);
        assert_eq!(ptr, UNIFORM_PTR_GLOBAL);

        // Uniform location 1 (64-byte stride)
        let (offset, ptr) = compute_uniform_offset(1);
        assert_eq!(offset, 64);
        assert_eq!(ptr, UNIFORM_PTR_GLOBAL);

        // Uniform location 10
        let (offset, ptr) = compute_uniform_offset(10);
        assert_eq!(offset, 640);
        assert_eq!(ptr, UNIFORM_PTR_GLOBAL);
    }

    #[test]
    fn test_input_offset_unsupported_stage() {
        // Compute shaders don't have traditional input locations
        let (offset, ptr) = compute_input_offset(0, ShaderStage::Compute);
        assert_eq!(offset, 0);
        assert_eq!(ptr, 0);
    }

    #[test]
    fn test_constant_values() {
        // Verify pointer global indices match expected values
        assert_eq!(ATTR_PTR_GLOBAL, 0);
        assert_eq!(UNIFORM_PTR_GLOBAL, 1);
        assert_eq!(VARYING_PTR_GLOBAL, 2);
        assert_eq!(PRIVATE_PTR_GLOBAL, 3);
    }
}
