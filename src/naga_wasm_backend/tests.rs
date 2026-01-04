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
                attribute_locations: &HashMap::new(),
                uniform_locations: &HashMap::new(),
                varying_locations: &HashMap::new(),
            },
            Some("test"),
        );

        assert!(res.is_err());
        let e_str = format!("{:?}", res.err().unwrap());
        assert!(e_str.contains("Varying 'vtest' has no assigned location"));
    }
}
