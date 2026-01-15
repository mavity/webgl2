//! Preparation pass: analyze Naga module and compute function manifests.

use super::registry::{FunctionManifest, FunctionRegistry};
use crate::naga_wasm_backend::function_abi::FunctionABI;
use crate::naga_wasm_backend::types;
use naga::{valid::ModuleInfo, Module};

/// Run the preparation pass over a Naga module.
///
/// This function analyzes all functions and entry points, computing their ABIs
/// and frame requirements. The resulting registry is used during code emission
/// to determine calling conventions and frame allocation.
///
/// # Arguments
/// - `module`: The Naga module to analyze
/// - `_module_info`: Validated module info (currently unused, reserved for future use)
///
/// # Returns
/// A `FunctionRegistry` containing manifests for all functions and entry points.
pub fn prep_module(module: &Module, _module_info: &ModuleInfo) -> FunctionRegistry {
    let mut registry = FunctionRegistry::new();

    // Process all regular functions
    for (handle, function) in module.functions.iter() {
        // Collect parameter types
        let param_types: Vec<_> = function.arguments.iter().map(|arg| arg.ty).collect();

        // Get result type if present
        let result_type = function.result.as_ref().map(|r| r.ty);

        // Compute ABI
        let abi = match FunctionABI::compute(module, &param_types, result_type) {
            Ok(abi) => abi,
            Err(e) => {
                // If ABI computation fails, log and skip this function
                tracing::warn!("Failed to compute ABI for function: {:?}", e);
                continue;
            }
        };

        // Calculate linear frame size (same as ABI frame_size for now)
        let linear_frame_size = abi.frame_size;

        // Determine if frame allocation is needed
        let needs_frame_alloc = abi.uses_frame;

        let manifest = FunctionManifest {
            abi,
            linear_frame_size,
            needs_frame_alloc,
        };

        registry.insert_function(handle, manifest);
    }

    // Process entry points
    for entry_point in &module.entry_points {
        // Entry points use a fixed ABI with 6 pointer parameters
        // (type, attr_ptr, uniform_ptr, varying_ptr, private_ptr, texture_ptr) -> ()

        // Calculate frame size based on local variables
        let mut frame_size = 0u32;

        for (_handle, local_var) in entry_point.function.local_variables.iter() {
            if let Ok(size) = types::type_size(&module.types[local_var.ty].inner) {
                // Align to 4 bytes (minimum alignment)
                frame_size = align_up(frame_size, 4);
                frame_size += size;
            }
        }

        // Entry points always use a fixed flattened ABI (no frame-based parameters)
        // Create a minimal ABI representing the 6-pointer signature
        let abi = FunctionABI {
            params: vec![],      // Entry point params are not represented in ABI (fixed signature)
            result: None,        // Entry points return void
            uses_frame: false,   // Entry points don't use frame for parameters
            frame_size: 0,       // No frame needed for parameters
            frame_alignment: 16, // Standard alignment
            implicit_returns: vec![], // Entry points have no InOut parameters
        };

        let manifest = FunctionManifest {
            abi,
            linear_frame_size: frame_size,
            needs_frame_alloc: frame_size > 0,
        };

        registry.insert_entry_point(entry_point.name.clone(), manifest);
    }

    registry
}

/// Align a value up to the nearest multiple of alignment.
fn align_up(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use naga::{
        Arena, EntryPoint, Function, FunctionArgument, FunctionResult, ScalarKind, ShaderStage,
        Span, Type, TypeInner, UniqueArena,
    };

    /// Helper to create a minimal Naga module for testing.
    fn create_test_module() -> Module {
        Module {
            types: UniqueArena::new(),
            constants: Arena::new(),
            global_variables: Arena::new(),
            functions: Arena::new(),
            entry_points: Vec::new(),
            global_expressions: Arena::new(),
            special_types: naga::SpecialTypes::default(),
            overrides: Arena::new(),
            diagnostic_filters: Arena::new(),
            diagnostic_filter_leaf: None,
            doc_comments: Default::default(),
        }
    }

    #[test]
    fn test_prep_module_empty() {
        let module = create_test_module();
        let module_info = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();

        let registry = prep_module(&module, &module_info);

        assert_eq!(registry.function_count(), 0);
        assert_eq!(registry.entry_point_count(), 0);
    }

    #[test]
    fn test_prep_module_basic_function() {
        let mut module = create_test_module();

        // Add a simple scalar type (f32)
        let f32_type = module.types.insert(
            Type {
                name: Some("f32".to_string()),
                inner: TypeInner::Scalar(naga::Scalar {
                    kind: ScalarKind::Float,
                    width: 4,
                }),
            },
            Span::UNDEFINED,
        );

        // Create a function with one f32 parameter and f32 result
        let mut function = Function::default();
        function.arguments.push(FunctionArgument {
            name: Some("x".to_string()),
            ty: f32_type,
            binding: None,
        });
        function.result = Some(FunctionResult {
            ty: f32_type,
            binding: None,
        });

        let func_handle = module.functions.append(function, Span::UNDEFINED);

        let module_info = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();

        let registry = prep_module(&module, &module_info);

        assert_eq!(registry.function_count(), 1);

        let manifest = registry.get_function(func_handle).unwrap();
        assert!(
            !manifest.needs_frame_alloc,
            "Simple scalar function should not need frame"
        );
        assert_eq!(manifest.abi.params.len(), 1);
    }

    #[test]
    fn test_prep_module_entry_point() {
        let mut module = create_test_module();

        // Create a simple entry point
        let function = Function::default();

        let entry_point = EntryPoint {
            name: "main".to_string(),
            stage: ShaderStage::Fragment,
            early_depth_test: None,
            workgroup_size: [0, 0, 0],
            function,
            mesh_info: None,
            task_payload: None,
            workgroup_size_overrides: None,
        };

        module.entry_points.push(entry_point);

        let module_info = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .expect("Validation failed");

        let registry = prep_module(&module, &module_info);

        assert_eq!(registry.entry_point_count(), 1);

        let manifest = registry.get_entry_point("main").unwrap();
        assert_eq!(
            manifest.abi.params.len(),
            0,
            "Entry points use fixed signature"
        );
    }

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(0, 4), 0);
        assert_eq!(align_up(1, 4), 4);
        assert_eq!(align_up(4, 4), 4);
        assert_eq!(align_up(5, 4), 8);
        assert_eq!(align_up(8, 4), 8);
        assert_eq!(align_up(10, 16), 16);
        assert_eq!(align_up(16, 16), 16);
        assert_eq!(align_up(17, 16), 32);
    }
}
