//! Control flow translation from Naga IR to WASM
//!
//! Phase 0: Placeholder for future implementation

use super::BackendError;

use naga::front::Typifier;
use std::collections::HashMap;
use wasm_encoder::{Function, Instruction};

/// Translate a Naga statement to WASM instructions
pub fn translate_statement(
    stmt: &naga::Statement,
    func: &naga::Function,
    module: &naga::Module,
    wasm_func: &mut Function,
    global_offsets: &HashMap<naga::Handle<naga::GlobalVariable>, (u32, u32)>,
    local_offsets: &HashMap<naga::Handle<naga::LocalVariable>, u32>,
    call_result_locals: &HashMap<naga::Handle<naga::Expression>, u32>,
    stage: naga::ShaderStage,
    typifier: &Typifier,
    naga_function_map: &HashMap<naga::Handle<naga::Function>, u32>,
    argument_local_offsets: &HashMap<u32, u32>,
    is_entry_point: bool,
    scratch_base: u32,
) -> Result<(), BackendError> {
    // crate::js_print(&format!("DEBUG: Statement: {:?}", stmt));
    match stmt {
        naga::Statement::Block(block) => {
            for s in block {
                translate_statement(
                    s,
                    func,
                    module,
                    wasm_func,
                    global_offsets,
                    local_offsets,
                    call_result_locals,
                    stage,
                    typifier,
                    naga_function_map,
                    argument_local_offsets,
                    is_entry_point,
                    scratch_base,
                )?;
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

            for i in 0..num_components {
                // Evaluate pointer (address)
                super::expressions::translate_expression(
                    *pointer,
                    func,
                    module,
                    wasm_func,
                    global_offsets,
                    local_offsets,
                    call_result_locals,
                    stage,
                    typifier,
                    naga_function_map,
                    argument_local_offsets,
                    is_entry_point,
                    scratch_base,
                )?;
                // Evaluate value component i
                super::expressions::translate_expression_component(
                    *value,
                    i,
                    func,
                    module,
                    wasm_func,
                    global_offsets,
                    local_offsets,
                    call_result_locals,
                    stage,
                    typifier,
                    naga_function_map,
                    argument_local_offsets,
                    is_entry_point,
                    scratch_base,
                )?;
                // Store
                wasm_func.instruction(&Instruction::F32Store(wasm_encoder::MemArg {
                    offset: (i * 4) as u64,
                    align: 2,
                    memory_index: 0,
                }));
            }
        }
        naga::Statement::Call {
            function,
            arguments,
            result,
        } => {
            // Push arguments
            for arg in arguments {
                super::expressions::translate_expression(
                    *arg,
                    func,
                    module,
                    wasm_func,
                    global_offsets,
                    local_offsets,
                    call_result_locals,
                    stage,
                    typifier,
                    naga_function_map,
                    argument_local_offsets,
                    is_entry_point,
                    scratch_base,
                )?;
            }
            // Call
            if let Some(&wasm_idx) = naga_function_map.get(function) {
                wasm_func.instruction(&Instruction::Call(wasm_idx));
            } else {
                return Err(BackendError::UnsupportedFeature(format!(
                    "Call to unknown function: {:?}",
                    function
                )));
            }
            // Handle result if any
            if let Some(res_handle) = result {
                if let Some(&local_idx) = call_result_locals.get(res_handle) {
                    // Store all components of the result into locals
                    let called_func = &module.functions[*function];
                    if let Some(ret) = &called_func.result {
                        let types = super::types::naga_to_wasm_types(&module.types[ret.ty].inner)?;
                        // WASM returns values in order, so we need to store them in reverse order if we use LocalSet?
                        // Actually, LocalSet pops the top value. If the function returns (f32, f32), the stack is [..., v1, v2].
                        // So we should do LocalSet(local_idx + 1), then LocalSet(local_idx).
                        for i in (0..types.len()).rev() {
                            wasm_func.instruction(&Instruction::LocalSet(local_idx + i as u32));
                        }
                    }
                } else {
                    // If it's not used, we should pop it.
                    let called_func = &module.functions[*function];
                    if let Some(ret) = &called_func.result {
                        let types = super::types::naga_to_wasm_types(&module.types[ret.ty].inner)?;
                        for _ in 0..types.len() {
                            wasm_func.instruction(&Instruction::Drop);
                        }
                    }
                }
            }
        }
        naga::Statement::Return { value } => {
            if let Some(expr_handle) = value {
                if !is_entry_point {
                    super::expressions::translate_expression(
                        *expr_handle,
                        func,
                        module,
                        wasm_func,
                        global_offsets,
                        local_offsets,
                        call_result_locals,
                        stage,
                        typifier,
                        naga_function_map,
                        argument_local_offsets,
                        is_entry_point,
                        scratch_base,
                    )?;
                }
            }
            wasm_func.instruction(&Instruction::Return);
        }
        _ => {}
    }
    Ok(())
}
