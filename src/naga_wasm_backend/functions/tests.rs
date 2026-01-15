//! Tests for the functions module (registry and preparation pass).

#[cfg(test)]
mod internal_tests {
    use crate::naga_wasm_backend::functions::{prep_module, FunctionRegistry};
    use naga::{
        Arena, EntryPoint, Function, FunctionArgument, FunctionResult, LocalVariable, ScalarKind,
        ShaderStage, Span, Type, TypeInner, UniqueArena,
    };

    /// Helper to create a minimal Naga module for testing.
    fn create_test_module() -> naga::Module {
        naga::Module {
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

    /// Helper to add an f32 type to a module.
    fn add_f32_type(module: &mut naga::Module) -> naga::Handle<Type> {
        module.types.insert(
            Type {
                name: Some("f32".to_string()),
                inner: TypeInner::Scalar(naga::Scalar {
                    kind: ScalarKind::Float,
                    width: 4,
                }),
            },
            Span::UNDEFINED,
        )
    }

    /// Helper to add a vec4 type to a module.
    fn add_vec4_type(module: &mut naga::Module) -> naga::Handle<Type> {
        module.types.insert(
            Type {
                name: Some("vec4".to_string()),
                inner: TypeInner::Vector {
                    size: naga::VectorSize::Quad,
                    scalar: naga::Scalar {
                        kind: ScalarKind::Float,
                        width: 4,
                    },
                },
            },
            Span::UNDEFINED,
        )
    }

    /// Helper to validate a module and return ModuleInfo.
    fn validate_module(module: &naga::Module) -> naga::valid::ModuleInfo {
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(module)
        .unwrap()
    }

    #[test]
    fn test_registry_empty() {
        let registry = FunctionRegistry::new();
        assert_eq!(registry.function_count(), 0);
        assert_eq!(registry.entry_point_count(), 0);
    }

    #[test]
    fn test_prep_module_empty() {
        let module = create_test_module();
        let module_info = validate_module(&module);

        let registry = prep_module(&module, &module_info);

        assert_eq!(registry.function_count(), 0);
        assert_eq!(registry.entry_point_count(), 0);
    }

    #[test]
    fn test_prep_module_simple_function() {
        let mut module = create_test_module();
        let f32_type = add_f32_type(&mut module);

        // Create function: fn add(a: f32, b: f32) -> f32
        let mut function = Function::default();
        function.arguments.push(FunctionArgument {
            name: Some("a".to_string()),
            ty: f32_type,
            binding: None,
        });
        function.arguments.push(FunctionArgument {
            name: Some("b".to_string()),
            ty: f32_type,
            binding: None,
        });
        function.result = Some(FunctionResult {
            ty: f32_type,
            binding: None,
        });

        let func_handle = module.functions.append(function, Span::UNDEFINED);
        let module_info = validate_module(&module);

        let registry = prep_module(&module, &module_info);

        assert_eq!(registry.function_count(), 1);

        let manifest = registry.get_function(func_handle).unwrap();
        assert!(
            !manifest.needs_frame_alloc,
            "Simple scalar function should not need frame"
        );
        assert_eq!(manifest.abi.params.len(), 2);
        assert!(manifest.abi.result.is_some());
    }

    #[test]
    fn test_prep_module_vector_function() {
        let mut module = create_test_module();
        let vec4_type = add_vec4_type(&mut module);

        // Create function: fn transform(v: vec4) -> vec4
        let mut function = Function::default();
        function.arguments.push(FunctionArgument {
            name: Some("v".to_string()),
            ty: vec4_type,
            binding: None,
        });
        function.result = Some(FunctionResult {
            ty: vec4_type,
            binding: None,
        });

        let func_handle = module.functions.append(function, Span::UNDEFINED);
        let module_info = validate_module(&module);

        let registry = prep_module(&module, &module_info);

        assert_eq!(registry.function_count(), 1);

        let manifest = registry.get_function(func_handle).unwrap();
        // vec4 is 16 bytes, should be flattened (at threshold)
        assert!(
            !manifest.needs_frame_alloc,
            "vec4 should be flattened (16 bytes)"
        );
        assert_eq!(manifest.linear_frame_size, 0);
    }

    #[test]
    fn test_prep_module_entry_point_no_locals() {
        let mut module = create_test_module();

        let function = Function::default();

        let entry_point = EntryPoint {
            name: "fragment_main".to_string(),
            stage: ShaderStage::Fragment,
            early_depth_test: None,
            workgroup_size: [0, 0, 0],
            function,
            mesh_info: None,
            task_payload: None,
            workgroup_size_overrides: None,
        };

        module.entry_points.push(entry_point);
        let module_info = validate_module(&module);

        let registry = prep_module(&module, &module_info);

        assert_eq!(registry.entry_point_count(), 1);

        let manifest = registry.get_entry_point("fragment_main").unwrap();
        assert_eq!(
            manifest.abi.params.len(),
            0,
            "Entry points use fixed signature"
        );
        assert_eq!(manifest.linear_frame_size, 0, "No locals, no frame");
        assert!(!manifest.needs_frame_alloc);
    }

    #[test]
    fn test_prep_module_entry_point_with_locals() {
        let mut module = create_test_module();
        let vec4_type = add_vec4_type(&mut module);

        let mut function = Function::default();

        // Add local variables
        function.local_variables.append(
            LocalVariable {
                name: Some("temp".to_string()),
                ty: vec4_type,
                init: None,
            },
            Span::UNDEFINED,
        );

        let entry_point = EntryPoint {
            name: "fragment_main".to_string(),
            stage: ShaderStage::Fragment,
            mesh_info: None,
            task_payload: None,
            workgroup_size_overrides: None,
            early_depth_test: None,
            workgroup_size: [0, 0, 0],
            function,
        };

        module.entry_points.push(entry_point);
        let module_info = validate_module(&module);

        let registry = prep_module(&module, &module_info);

        assert_eq!(registry.entry_point_count(), 1);

        let manifest = registry.get_entry_point("fragment_main").unwrap();
        // vec4 = 16 bytes, should be included in frame size
        assert!(
            manifest.linear_frame_size >= 16,
            "Should account for vec4 local"
        );
        assert!(manifest.needs_frame_alloc);
    }

    #[test]
    fn test_prep_module_multiple_functions() {
        let mut module = create_test_module();
        let f32_type = add_f32_type(&mut module);
        let vec4_type = add_vec4_type(&mut module);

        // Function 1: fn foo(x: f32) -> f32
        let mut func1 = Function::default();
        func1.arguments.push(FunctionArgument {
            name: Some("x".to_string()),
            ty: f32_type,
            binding: None,
        });
        func1.result = Some(FunctionResult {
            ty: f32_type,
            binding: None,
        });
        let handle1 = module.functions.append(func1, Span::UNDEFINED);

        // Function 2: fn bar(v: vec4) -> vec4
        let mut func2 = Function::default();
        func2.arguments.push(FunctionArgument {
            name: Some("v".to_string()),
            ty: vec4_type,
            binding: None,
        });
        func2.result = Some(FunctionResult {
            ty: vec4_type,
            binding: None,
        });
        let handle2 = module.functions.append(func2, Span::UNDEFINED);

        let module_info = validate_module(&module);

        let registry = prep_module(&module, &module_info);

        assert_eq!(registry.function_count(), 2);
        assert!(registry.get_function(handle1).is_some());
        assert!(registry.get_function(handle2).is_some());
    }

    #[test]
    fn test_prep_module_frame_size_calculation() {
        let mut module = create_test_module();

        // Create a large array type that requires frame passing
        let f32_type = add_f32_type(&mut module);
        let array_type = module.types.insert(
            Type {
                name: Some("array_20_f32".to_string()),
                inner: TypeInner::Array {
                    base: f32_type,
                    size: naga::ArraySize::Constant(std::num::NonZeroU32::new(20).unwrap()),
                    stride: 4,
                },
            },
            Span::UNDEFINED,
        );

        // Create function with large array parameter
        let mut function = Function::default();
        function.arguments.push(FunctionArgument {
            name: Some("data".to_string()),
            ty: array_type,
            binding: None,
        });

        let func_handle = module.functions.append(function, Span::UNDEFINED);
        let module_info = validate_module(&module);

        let registry = prep_module(&module, &module_info);

        let manifest = registry.get_function(func_handle).unwrap();
        // 20 * 4 = 80 bytes, exceeds 16-byte threshold
        assert!(
            manifest.needs_frame_alloc,
            "Large array should require frame"
        );
        assert!(manifest.abi.uses_frame);
        assert!(manifest.linear_frame_size > 0);
    }
}
