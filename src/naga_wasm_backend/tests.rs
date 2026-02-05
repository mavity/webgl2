#[cfg(test)]
mod tests {
    use super::*;
    use naga::{AddressSpace, Module, ShaderStage};
    use naga::TypeInner;
    use naga::VectorSize;
    use naga::ScalarKind;
    use naga::valid::ModuleInfo;
    use std::collections::HashMap;

    #[test]
    fn unmapped_private_global_errors() {
        // Create a minimal module with a Private global variable and no mappings
        let mut module = Module::default();
        let ty = module.types.append(naga::Type {
            name: None,
            inner: TypeInner::Vector {
                size: VectorSize::Quad,
                scalar: naga::Scalar { kind: ScalarKind::Uint, width: 4 },
            },
        });

        module.global_variables.append(naga::GlobalVariable {
            name: Some("vtest".to_string()),
            space: AddressSpace::Private,
            binding: None,
            ty,
            init: None,
        });

        // ModuleInfo is not strictly needed for this path; a default is fine
        let info = ModuleInfo::default();

        let backend = WasmBackend::new(WasmBackendConfig::default());
        let res = backend.compile(
            CompileConfig {
                module: &module,
                info: &info,
                source: "",
                stage: ShaderStage::Fragment,
                is_webgpu: false,
                entry_point: None,
                attribute_locations: &HashMap::new(),
                uniform_locations: &HashMap::new(),
                varying_locations: &HashMap::new(),
                varying_types: &HashMap::new(),
                uniform_types: &HashMap::new(),
                attribute_types: &HashMap::new(),
            },
            Some("test"),
        );

        assert!(res.is_err());
        let e_str = format!("{:?}", res.err().unwrap());
        assert!(e_str.contains("Varying 'vtest' has no assigned location"));
    }

    #[test]
    fn compile_fma_wgsl() {
        // WGSL shader using fma
        let src = r#"
            @group(0) @binding(0) var<uniform> u_a : f32;
            @group(0) @binding(1) var<uniform> u_b : f32;
            @group(0) @binding(2) var<uniform> u_c : f32;

            @stage(fragment)
            fn main() -> @location(0) f32 {
                return fma(u_a, u_b, u_c);
            }
        "#;

        let module: Module = naga::front::wgsl::parse_str(src).expect("WGSL parse");
        let info = ModuleInfo::default();

        let backend = WasmBackend::new(WasmBackendConfig::default());
        let res = backend.compile(
            CompileConfig {
                module: &module,
                info: &info,
                source: src,
                stage: ShaderStage::Fragment,
                entry_point: Some("main"),
                attribute_locations: &HashMap::new(),
                uniform_locations: &HashMap::new(),
                varying_locations: &HashMap::new(),
                varying_types: &HashMap::new(),
                uniform_types: &HashMap::new(),
                attribute_types: &HashMap::new(),
            },
            Some("fma_test"),
        );

        assert!(res.is_ok());
    }

    #[test]
    fn compile_smoothstep_wgsl() {
        // WGSL shader using smoothstep
        let src = r#"
            @group(0) @binding(0) var<uniform> u_e0 : f32;
            @group(0) @binding(1) var<uniform> u_e1 : f32;
            @group(0) @binding(2) var<uniform> u_x : f32;

            @stage(fragment)
            fn main() -> @location(0) f32 {
                return smoothstep(u_e0, u_e1, u_x);
            }
        "#;

        let module: Module = naga::front::wgsl::parse_str(src).expect("WGSL parse");
        let info = ModuleInfo::default();

        let backend = WasmBackend::new(WasmBackendConfig::default());
        let res = backend.compile(
            CompileConfig {
                module: &module,
                info: &info,
                source: src,
                stage: ShaderStage::Fragment,
                entry_point: Some("main"),
                attribute_locations: &HashMap::new(),
                uniform_locations: &HashMap::new(),
                varying_locations: &HashMap::new(),
                varying_types: &HashMap::new(),
                uniform_types: &HashMap::new(),
                attribute_types: &HashMap::new(),
            },
            Some("smoothstep_test"),
        );

        assert!(res.is_ok());
    }
}
