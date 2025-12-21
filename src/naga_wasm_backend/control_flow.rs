//! Control flow translation from Naga IR to WASM
//!
//! Phase 0: Placeholder for future implementation

use super::BackendError;

use wasm_encoder::{Function, Instruction};
use std::collections::HashMap;
use naga::front::Typifier;

/// Translate a Naga statement to WASM instructions
pub fn translate_statement(
    stmt: &naga::Statement,
    func: &naga::Function,
    module: &naga::Module,
    wasm_func: &mut Function,
    global_offsets: &HashMap<naga::Handle<naga::GlobalVariable>, u32>,
    stage: naga::ShaderStage,
    typifier: &Typifier,
    naga_function_map: &HashMap<naga::Handle<naga::Function>, u32>,
    is_entry_point: bool,
) -> Result<(), BackendError> {
    // crate::js_print(&format!("DEBUG: Statement: {:?}", stmt));
    match stmt {
        naga::Statement::Block(block) => {
            for s in block {
                translate_statement(s, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
            }
        }
        naga::Statement::Store { pointer, value } => {
            // Determine how many components to store
            let value_ty = typifier.get(*value, &module.types);
            let num_components = match value_ty {
                naga::TypeInner::Vector { size, .. } => match size {
                    naga::VectorSize::Bi => 2,
                    naga::VectorSize::Tri => 3,
                    naga::VectorSize::Quad => 4,
                },
                naga::TypeInner::Matrix { columns, rows, .. } => {
                    let cols = match columns {
                        naga::VectorSize::Bi => 2,
                        naga::VectorSize::Tri => 3,
                        naga::VectorSize::Quad => 4,
                    };
                    let rws = match rows {
                        naga::VectorSize::Bi => 2,
                        naga::VectorSize::Tri => 3,
                        naga::VectorSize::Quad => 4,
                    };
                    cols * rws
                }
                _ => 1,
            };

            // crate::js_print(&format!("DEBUG: Store num_components={}", num_components));

            for i in 0..num_components {
                // Evaluate pointer (address)
                super::expressions::translate_expression(*pointer, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
                // Add offset for this component
                if i > 0 {
                    wasm_func.instruction(&Instruction::I32Const((i * 4) as i32));
                    wasm_func.instruction(&Instruction::I32Add);
                }
                // Evaluate value component i
                super::expressions::translate_expression_component(*value, i, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
                // Store
                wasm_func.instruction(&Instruction::F32Store(wasm_encoder::MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));
            }
        }
        naga::Statement::Call { function, arguments, result } => {
            // Push arguments
            for arg in arguments {
                super::expressions::translate_expression(*arg, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
            }
            // Call
            if let Some(&wasm_idx) = naga_function_map.get(function) {
                wasm_func.instruction(&Instruction::Call(wasm_idx));
            } else {
                return Err(BackendError::UnsupportedFeature(format!("Call to unknown function: {:?}", function)));
            }
            // Handle result if any
            if let Some(_res) = result {
                // For now, we don't support storing the result of a call in an expression
                // because Naga's IR uses handles for expressions, and we'd need to map them to locals.
                // But if the function returns something, it will be on the WASM stack.
                // If it's not used, we should pop it.
                let called_func = &module.functions[*function];
                if called_func.result.is_some() {
                    wasm_func.instruction(&Instruction::Drop);
                }
            }
        }
        naga::Statement::Return { value } => {
            if let Some(expr_handle) = value {
                super::expressions::translate_expression(*expr_handle, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
            }
            wasm_func.instruction(&Instruction::Return);
        }
        _ => {}
    }
    Ok(())
}
