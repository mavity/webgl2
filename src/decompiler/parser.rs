//! WASM Parser: Parses WASM bytecode and extracts function bodies.
//!
//! This module uses wasmparser to iterate through WASM sections and
//! extract the information needed for decompilation.

use super::ast::{Function, ScalarType};
use super::lifter::{valtype_to_scalar, Lifter};
use super::module::DecompiledModule;
use anyhow::Result;
use wasmparser::{CompositeInnerType, FunctionBody, Parser, Payload, TypeRef};

/// Stored function type info for decompilation.
#[derive(Clone)]
struct FuncTypeInfo {
    params: Vec<ScalarType>,
    results: Vec<ScalarType>,
}

/// Parse a WASM module and decompile all functions.
pub fn parse_wasm(data: &[u8]) -> Result<DecompiledModule> {
    let mut module = DecompiledModule::new();
    let parser = Parser::new(0);

    // Collect type section info
    let mut types: Vec<FuncTypeInfo> = Vec::new();
    // Collect function section info (type indices)
    let mut func_type_indices: Vec<u32> = Vec::new();
    // Track import count
    let mut import_func_count: u32 = 0;

    // First pass: collect metadata
    for payload in parser.parse_all(data) {
        let payload = payload?;
        match payload {
            Payload::TypeSection(reader) => {
                for rec_group in reader {
                    let rec_group = rec_group?;
                    // Iterate through types in the rec group
                    for sub_type in rec_group.types() {
                        let composite = &sub_type.composite_type;
                        // Check if it's a function type using pattern matching
                        if let CompositeInnerType::Func(func_type) = &composite.inner {
                            let params: Vec<ScalarType> = func_type
                                .params()
                                .iter()
                                .map(|ty| valtype_to_scalar(*ty))
                                .collect();
                            let results: Vec<ScalarType> = func_type
                                .results()
                                .iter()
                                .map(|ty| valtype_to_scalar(*ty))
                                .collect();
                            types.push(FuncTypeInfo { params, results });
                        }
                    }
                }
            }
            Payload::ImportSection(reader) => {
                for import in reader {
                    let import = import?;
                    if matches!(import.ty, TypeRef::Func(_)) {
                        import_func_count += 1;
                    }
                }
            }
            Payload::FunctionSection(reader) => {
                for func in reader {
                    let type_idx = func?;
                    func_type_indices.push(type_idx);
                }
            }
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export?;
                    if let wasmparser::ExternalKind::Func = export.kind {
                        module.set_function_name(export.index, export.name.to_string());
                    }
                }
            }
            Payload::CustomSection(reader) => {
                if reader.name() == "name" {
                    use wasmparser::{BinaryReader, Name, NameSectionReader};
                    let binary_reader = BinaryReader::new(reader.data(), reader.data_offset());
                    let name_section = NameSectionReader::new(binary_reader);
                    for name in name_section {
                        if let Name::Function(names) = name? {
                            for naming in names {
                                let naming = naming?;
                                module.set_function_name(naming.index, naming.name.to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Second pass: decompile function bodies
    let parser = Parser::new(0);
    let mut code_idx = 0u32;

    for payload in parser.parse_all(data) {
        let payload = payload?;
        if let Payload::CodeSectionEntry(body) = payload {
            let func_idx = import_func_count + code_idx;
            let type_idx = func_type_indices
                .get(code_idx as usize)
                .copied()
                .unwrap_or(0);
            let func_type_info = types.get(type_idx as usize);

            if let Some(type_info) = func_type_info {
                let func = decompile_function(func_idx, type_info, body)?;
                module.add_function(func);
            }
            code_idx += 1;
        }
    }

    module.import_count = import_func_count;
    Ok(module)
}

/// Decompile a single function body.
fn decompile_function(
    func_idx: u32,
    type_info: &FuncTypeInfo,
    body: FunctionBody,
) -> Result<Function> {
    let param_types = type_info.params.clone();
    let param_count = param_types.len() as u32;
    let return_type = type_info.results.first().copied();

    // Get locals
    let mut local_types: Vec<ScalarType> = param_types.clone();
    for local in body.get_locals_reader()? {
        let (count, ty) = local?;
        for _ in 0..count {
            local_types.push(valtype_to_scalar(ty));
        }
    }

    // Create lifter and process operators
    let mut lifter = Lifter::new(param_count, local_types.clone());
    let operators_reader = body.get_operators_reader()?;

    for op_result in operators_reader {
        let op = op_result?;
        lifter.process_operator(&op);
    }

    // Finish and get the body
    let body_stmts = lifter.finish(return_type);

    Ok(Function {
        func_idx,
        param_count,
        param_types,
        return_type,
        local_types: local_types[param_count as usize..].to_vec(),
        body: body_stmts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal valid WASM module with one function that returns 42
    const MINIMAL_WASM: &[u8] = &[
        0x00, 0x61, 0x73, 0x6D, // magic
        0x01, 0x00, 0x00, 0x00, // version
        // Type section
        0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7F, // () -> i32
        // Function section
        0x03, 0x02, 0x01, 0x00, // function 0 uses type 0
        // Export section
        0x07, 0x08, 0x01, 0x04, 0x6D, 0x61, 0x69, 0x6E, 0x00, 0x00, // export "main" = func 0
        // Code section
        0x0A, 0x06, 0x01, 0x04, 0x00, 0x41, 0x2A, 0x0B, // func: i32.const 42, end
    ];

    #[test]
    fn test_parse_minimal_wasm() {
        let result = parse_wasm(MINIMAL_WASM);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        let module = result.unwrap();
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.get_function_name(0), "main");
    }
}
