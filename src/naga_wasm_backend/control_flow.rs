//! Control flow translation from Naga IR to WASM
//!
//! Phase 0: Placeholder for future implementation

use super::{BackendError, TranslationContext};

use wasm_encoder::Instruction;

/// Translate a Naga statement to WASM instructions
pub fn translate_statement(
    stmt: &naga::Statement,
    ctx: &mut TranslationContext,
) -> Result<(), BackendError> {
    // crate::js_print(&format!("DEBUG: Statement: {:?}", stmt));
    match stmt {
        naga::Statement::Block(block) => {
            for s in block {
                translate_statement(s, ctx)?;
            }
        }
        naga::Statement::Store { pointer, value } => {
            // Determine how many components to store
            let value_ty = ctx.typifier.get(*value, &ctx.module.types);
            let num_components = super::types::component_count(value_ty);

            for i in 0..num_components {
                // Evaluate pointer (address)
                super::expressions::translate_expression(*pointer, ctx)?;
                // Evaluate value component i
                super::expressions::translate_expression_component(*value, i, ctx)?;
                // Store
                ctx.wasm_func
                    .instruction(&Instruction::F32Store(wasm_encoder::MemArg {
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
                super::expressions::translate_expression(*arg, ctx)?;
            }
            // Call
            if let Some(&wasm_idx) = ctx.naga_function_map.get(function) {
                ctx.wasm_func.instruction(&Instruction::Call(wasm_idx));
            } else {
                return Err(BackendError::UnsupportedFeature(format!(
                    "Call to unknown function: {:?}",
                    function
                )));
            }
            // Handle result if any
            if let Some(res_handle) = result {
                if let Some(&local_idx) = ctx.call_result_locals.get(res_handle) {
                    // Store all components of the result into locals
                    let called_func = &ctx.module.functions[*function];
                    if let Some(ret) = &called_func.result {
                        let types =
                            super::types::naga_to_wasm_types(&ctx.module.types[ret.ty].inner)?;
                        // WASM returns values in order, so we need to store them in reverse order if we use LocalSet?
                        // Actually, LocalSet pops the top value. If the function returns (f32, f32), the stack is [..., v1, v2].
                        // So we should do LocalSet(local_idx + 1), then LocalSet(local_idx).
                        for i in (0..types.len()).rev() {
                            ctx.wasm_func
                                .instruction(&Instruction::LocalSet(local_idx + i as u32));
                        }
                    }
                } else {
                    // If it's not used, we should pop it.
                    let called_func = &ctx.module.functions[*function];
                    if let Some(ret) = &called_func.result {
                        let types =
                            super::types::naga_to_wasm_types(&ctx.module.types[ret.ty].inner)?;
                        for _ in 0..types.len() {
                            ctx.wasm_func.instruction(&Instruction::Drop);
                        }
                    }
                }
            }
        }
        naga::Statement::Return { value } => {
            if let Some(expr_handle) = value {
                if !ctx.is_entry_point {
                    super::expressions::translate_expression(*expr_handle, ctx)?;
                }
            }
            ctx.wasm_func.instruction(&Instruction::Return);
        }
        naga::Statement::If {
            condition,
            accept,
            reject,
        } => {
            // Evaluate condition
            // We assume translate_expression puts an F32 (bits of I32 bool) on the stack
            super::expressions::translate_expression(*condition, ctx)?;
            ctx.wasm_func.instruction(&Instruction::I32ReinterpretF32);

            ctx.wasm_func
                .instruction(&Instruction::If(wasm_encoder::BlockType::Empty));
            for s in accept {
                translate_statement(s, ctx)?;
            }
            if !reject.is_empty() {
                ctx.wasm_func.instruction(&Instruction::Else);
                for s in reject {
                    translate_statement(s, ctx)?;
                }
            }
            ctx.wasm_func.instruction(&Instruction::End);
        }
        _ => {}
    }
    Ok(())
}
