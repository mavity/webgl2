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
//! The layout uses fixed 16-byte (vec4) slots for simplicity and compatibility.
//! A future optimization could implement "Deterministic Packed Layout" to reduce
//! memory usage, but the current approach provides:
//! - Predictable offsets
//! - Alignment guarantees
//! - Simple implementation
//! - Zero-cost abstraction (inlined)

use naga::{Binding, BuiltIn, ShaderStage};

/// Memory pointer global indices
pub const VARYING_PTR_GLOBAL: u32 = 2;
pub const PRIVATE_PTR_GLOBAL: u32 = 3;

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
/// ```ignore
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
/// ```
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
        assert_eq!(offset, 0);
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
}
