//! Private memory layout calculation for fragment shaders
//!
//! This module computes the exact memory layout for the PRIVATE_PTR region
//! used in fragment shaders, avoiding arbitrary magic numbers and ensuring
//! safe, tight packing of:
//! - Fragment color outputs (Location 0-N)
//! - Local variables
//! - FragDepth (if used)
//!
//! ## Key Improvements Over Previous Implementation
//!
//! **Before (Arbitrary Magic Numbers):**
//! - Fragment outputs: 0-1023 bytes (assumed, wasteful)
//! - Local variables: Start at 2048 (arbitrary)
//! - FragDepth: At 4096 (hardcoded)
//! - **NO bounds checking** - locals could overflow into FragDepth!
//!
//! **After (Calculated Layout):**
//! - Fragment outputs: Sized exactly based on max location
//! - Local variables: Calculated from actual variable types
//! - FragDepth: Moved to 32KB (0x8000) for safety
//! - **Validated** - shaders exceeding 64KB are rejected at compile time
//!
//! ## Memory Safety
//!
//! The layout is validated to ensure:
//! 1. Total private memory ≤ 64KB (leaves room before frame stack at 512KB)
//! 2. Local variables cannot overflow into FragDepth region
//! 3. Warnings for shaders using >8KB of local variables
//!
//! ## Current Limitations
//!
//! - Fragment output analysis is conservative (assumes location 0 exists)
//! - FragDepth is assumed to be potentially used (safe but may waste 4 bytes)
//! - Future: Proper introspection of Naga function result types

use super::BackendError;
use naga::proc::{Alignment, Layouter};
use naga::{Function, Module, ShaderStage};
use std::collections::HashMap;

/// Maximum allowed private memory size (64KB safety limit)
/// This leaves plenty of room before the frame stack at 0x80000 (512KB)
const MAX_PRIVATE_MEMORY: u32 = 0x10000; // 64KB

/// Alignment for memory regions (16 bytes for optimal vec4 access)
const MEMORY_ALIGNMENT: Alignment = Alignment::SIXTEEN;

/// Calculated memory layout for the PRIVATE_PTR region
#[derive(Debug, Clone)]
pub struct PrivateMemoryLayout {
    /// Size of fragment color output region (always aligned to MEMORY_ALIGNMENT)
    pub frag_outputs_size: u32,

    /// Start offset for local variables (aligned after frag outputs)
    pub locals_start: u32,

    /// Total size needed for all local variables
    pub locals_size: u32,

    /// Mapping from local variable handles to their calculated offsets
    pub local_offsets: HashMap<naga::Handle<naga::LocalVariable>, u32>,

    /// Offset for FragDepth if used (placed after locals, aligned)
    pub frag_depth_offset: Option<u32>,

    /// Total size of private memory region
    pub total_size: u32,
}

impl PrivateMemoryLayout {
    /// Compute the memory layout for a function
    ///
    /// # Arguments
    /// * `module` - The Naga module containing type information
    /// * `func` - The function to compute layout for
    /// * `stage` - The shader stage (only Fragment stage uses private memory)
    ///
    /// # Returns
    /// * `Ok(PrivateMemoryLayout)` - Calculated layout
    /// * `Err(BackendError)` - If the shader exceeds memory limits or has invalid configuration
    pub fn compute(
        module: &Module,
        func: &Function,
        stage: ShaderStage,
    ) -> Result<Self, BackendError> {
        let mut layouter = Layouter::default();
        layouter.update(module.to_ctx()).map_err(|e| {
            BackendError::TypeConversion(format!("Failed to compute type layout: {:?}", e))
        })?;

        // Only fragment shaders use private memory for outputs + locals
        // Vertex shaders write to varying buffer, not private buffer
        let (frag_outputs_size, uses_frag_depth) = if stage == ShaderStage::Fragment {
            Self::analyze_fragment_outputs(module, func)
        } else {
            (0, false)
        };

        // Start locals right after fragment outputs
        let mut current_offset = frag_outputs_size;
        let mut max_alignment = Alignment::ONE;

        // Calculate total size needed for all local variables
        // We pack them using their natural alignment from Naga's layouter
        let mut local_offsets = HashMap::new();
        for (handle, var) in func.local_variables.iter() {
            let layout = &layouter[var.ty];
            let alignment = layout.alignment;
            max_alignment = max_alignment.max(alignment);

            // Align current offset to this variable's requirements
            current_offset = alignment.round_up(current_offset);

            // Store offset before incrementing
            local_offsets.insert(handle, current_offset);

            // Increment offset by variable size
            current_offset += layout.size;
        }

        let locals_size = current_offset - frag_outputs_size;

        // Place FragDepth after locals if used
        let frag_depth_offset = if uses_frag_depth {
            // FragDepth is an f32, needs 4-byte alignment
            Some(Alignment::FOUR.round_up(current_offset))
        } else {
            None
        };

        // Calculate total size
        let total_size = if let Some(depth_offset) = frag_depth_offset {
            depth_offset + 4 // FragDepth is f32 (4 bytes)
        } else {
            current_offset
        };

        // Validate total size
        if total_size > MAX_PRIVATE_MEMORY {
            return Err(BackendError::UnsupportedFeature(format!(
                "Private memory exceeds 64KB limit: {} bytes (max: {} bytes). \
                 Fragment outputs: {} bytes, Locals: {} bytes, FragDepth: {}. \
                 Consider reducing the number of local variables or simplifying the shader.",
                total_size,
                MAX_PRIVATE_MEMORY,
                frag_outputs_size,
                locals_size,
                if uses_frag_depth {
                    "4 bytes"
                } else {
                    "not used"
                }
            )));
        }

        // Warn if using excessive local memory (>8KB is unusual for shaders)
        if locals_size > 8192 {
            tracing::warn!(
                "Shader uses {} bytes of local variables (>8KB). \
                 This may indicate inefficient memory usage.",
                locals_size
            );
        }

        Ok(Self {
            frag_outputs_size,
            locals_start: frag_outputs_size,
            locals_size,
            local_offsets,
            frag_depth_offset,
            total_size: MEMORY_ALIGNMENT.round_up(total_size),
        })
    }

    /// Analyze fragment shader outputs to determine total size and FragDepth usage
    ///
    /// Walks the function result type (if it's a struct) and calculates the actual
    /// memory needed for color outputs.
    fn analyze_fragment_outputs(module: &Module, func: &Function) -> (u32, bool) {
        let mut max_location_end: u32 = 0;
        let mut uses_frag_depth = false;

        // Check if function has a result
        if let Some(result) = &func.result {
            let result_type = &module.types[result.ty];

            // If it's a single value with a binding, handle it
            if let Some(binding) = &result.binding {
                match binding {
                    naga::Binding::BuiltIn(naga::BuiltIn::FragDepth) => {
                        uses_frag_depth = true;
                    }
                    naga::Binding::Location { location, .. } => {
                        // Each location is treated as a 16-byte slot (GPU standard)
                        max_location_end = max_location_end.max((location + 1) * 16);
                    }
                    _ => {}
                }
            }

            // If it's a struct, analyze each member
            if let naga::TypeInner::Struct { members, .. } = &result_type.inner {
                for member in members {
                    if let Some(binding) = &member.binding {
                        match binding {
                            naga::Binding::BuiltIn(naga::BuiltIn::FragDepth) => {
                                uses_frag_depth = true;
                            }
                            naga::Binding::Location { location, .. } => {
                                max_location_end = max_location_end.max((location + 1) * 16);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // If we found no location outputs, assume at least location 0 exists for safety
        if max_location_end == 0 && !uses_frag_depth {
            max_location_end = 16;
        }

        (max_location_end, uses_frag_depth)
    }
}

/// Analyzes fragment outputs and locals for memory mapping.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_layout_simple() {
        // Create a minimal module and function for testing
        let module = Module::default();
        let func = Function::default();

        // Vertex shader should have minimal private memory
        let layout = PrivateMemoryLayout::compute(&module, &func, ShaderStage::Vertex)
            .expect("Layout computation failed");

        assert_eq!(layout.frag_outputs_size, 0);
        assert_eq!(layout.locals_start, 0);
        assert_eq!(layout.locals_size, 0);
        assert_eq!(layout.frag_depth_offset, None);
        assert_eq!(layout.total_size, 0);
    }

    #[test]
    fn test_memory_layout_fragment_no_locals() {
        let module = Module::default();
        let func = Function::default();

        // Fragment shader with no locals and no outputs (minimal case)
        let layout = PrivateMemoryLayout::compute(&module, &func, ShaderStage::Fragment)
            .expect("Layout computation failed");

        // Should still allocate space for at least location 0
        assert_eq!(layout.frag_outputs_size, 16); // 1 location * 16 bytes, aligned
        assert_eq!(layout.locals_start, 16);
        assert_eq!(layout.locals_size, 0);
    }

    #[test]
    fn test_excessive_memory_rejected() {
        let mut module = Module::default();

        // Create a function with way too many locals
        let mut func = Function::default();

        // Add a huge array type to exceed limits
        let huge_array_ty = naga::Type {
            name: None,
            inner: naga::TypeInner::Array {
                base: module.types.insert(
                    naga::Type {
                        name: None,
                        inner: naga::TypeInner::Scalar(naga::Scalar::F32),
                    },
                    naga::Span::UNDEFINED,
                ),
                size: naga::ArraySize::Constant(
                    std::num::NonZeroU32::new(20000).unwrap(), // 20000 * 4 = 80KB
                ),
                stride: 4,
            },
        };
        let huge_array_handle = module.types.insert(huge_array_ty, naga::Span::UNDEFINED);

        func.local_variables.append(
            naga::LocalVariable {
                name: Some("huge_array".to_string()),
                ty: huge_array_handle,
                init: None,
            },
            naga::Span::UNDEFINED,
        );

        // Should fail with memory limit exceeded
        let result = PrivateMemoryLayout::compute(&module, &func, ShaderStage::Fragment);
        assert!(result.is_err());

        if let Err(BackendError::UnsupportedFeature(msg)) = result {
            assert!(msg.contains("exceeds 64KB limit"));
        } else {
            panic!("Expected UnsupportedFeature error");
        }
    }

    #[test]
    fn test_memory_layout_locals_packing() {
        let mut module = Module::default();
        let mut func = Function::default();

        let f32_ty = module.types.insert(
            naga::Type {
                name: None,
                inner: naga::TypeInner::Scalar(naga::Scalar::F32),
            },
            naga::Span::UNDEFINED,
        );

        let vec2_ty = module.types.insert(
            naga::Type {
                name: None,
                inner: naga::TypeInner::Vector {
                    size: naga::VectorSize::Bi,
                    scalar: naga::Scalar::F32,
                },
            },
            naga::Span::UNDEFINED,
        );

        // Add locals in an order that might need padding if not careful
        // 1. f32 (4 bytes)
        // 2. vec2 (8 bytes, needs 8-byte alignment)
        // 3. f32 (4 bytes)

        let l1 = func.local_variables.append(
            naga::LocalVariable {
                name: Some("f1".to_string()),
                ty: f32_ty,
                init: None,
            },
            naga::Span::UNDEFINED,
        );
        let l2 = func.local_variables.append(
            naga::LocalVariable {
                name: Some("v1".to_string()),
                ty: vec2_ty,
                init: None,
            },
            naga::Span::UNDEFINED,
        );
        let l3 = func.local_variables.append(
            naga::LocalVariable {
                name: Some("f2".to_string()),
                ty: f32_ty,
                init: None,
            },
            naga::Span::UNDEFINED,
        );

        let layout = PrivateMemoryLayout::compute(&module, &func, ShaderStage::Fragment)
            .expect("Layout computation failed");

        // Fragment always starts locals at 16 (after loc0)
        assert_eq!(layout.locals_start, 16);

        let off1 = *layout.local_offsets.get(&l1).unwrap();
        let off2 = *layout.local_offsets.get(&l2).unwrap();
        let off3 = *layout.local_offsets.get(&l3).unwrap();

        // Check alignment and packing.
        // l1 is at 16.
        // l2 (vec2, align 8) should be at 24 (if 16+4=20, next 8-align is 24).
        // l3 (f32, align 4) should be at 32.

        assert_eq!(off1, 16);
        assert_eq!(off2, 24);
        assert_eq!(off3, 32);

        assert_eq!(layout.locals_size, 20); // 32 + 4 - 16 = 20
        assert_eq!(layout.total_size, 48); // 32 + 4 = 36, then aligned to 16 -> 48
    }
}
