//! Control flow translation from Naga IR to WASM
//!
//! Phase 0: Placeholder for future implementation

use super::{BackendError, TranslationContext};

use wasm_encoder::Instruction;

/// Translate a Naga statement to WASM instructions
pub fn translate_statement(
    stmt: &naga::Statement,
    span: &naga::Span,
    ctx: &mut TranslationContext,
) -> Result<(), BackendError> {
    // Calculate line number for debug
    let line = if let Some(_debug_step_idx) = ctx.debug_step_idx {
        span.location(ctx.source).line_number as i32
    } else {
        0
    };

    let is_call = matches!(stmt, naga::Statement::Call { .. });

    // Emit debug step for non-call statements
    if !is_call {
        if let Some(debug_step_idx) = ctx.debug_step_idx {
            // Heuristic: If line is 1 and source starts with #version, ignore it.
            // Also ignore if line is 0 (invalid).
            let skip = if line <= 1 {
                ctx.source.trim_start().starts_with("#version")
            } else {
                false
            };

            if !skip {
                // debug_step(line, -1, 0)
                ctx.wasm_func.instruction(&Instruction::I32Const(line));
                ctx.wasm_func.instruction(&Instruction::I32Const(-1));
                ctx.wasm_func.instruction(&Instruction::I32Const(0));
                ctx.wasm_func
                    .instruction(&Instruction::Call(debug_step_idx));
            }
        }
    }

    match stmt {
        naga::Statement::Block(block) => {
            for (s, s_span) in block.span_iter() {
                translate_statement(s, s_span, ctx)?;
            }
        }
        naga::Statement::Store { pointer, value } => {
            // Determine how many components to store
            let value_ty = ctx.typifier.get(*value, &ctx.module.types);
            let num_components = super::types::component_count(value_ty);

            // Use helper to determine if we should use I32Store or F32Store
            let use_i32_store = super::expressions::is_integer_type(value_ty);

            for i in 0..num_components {
                // Evaluate pointer (address)
                super::expressions::translate_expression(*pointer, ctx)?;
                // Evaluate value component i
                super::expressions::translate_expression_component(*value, i, ctx)?;
                // Store with appropriate instruction
                if use_i32_store {
                    ctx.wasm_func
                        .instruction(&Instruction::I32Store(wasm_encoder::MemArg {
                            offset: (i * 4) as u64,
                            align: 2,
                            memory_index: 0,
                        }));
                } else {
                    ctx.wasm_func
                        .instruction(&Instruction::F32Store(wasm_encoder::MemArg {
                            offset: (i * 4) as u64,
                            align: 2,
                            memory_index: 0,
                        }));
                }
            }
        }
        naga::Statement::Call {
            function,
            arguments,
            result,
        } => {
            let skip_debug = if ctx.debug_step_idx.is_some() {
                line <= 1 && ctx.source.trim_start().starts_with("#version")
            } else {
                false
            };

            // Handle Call with debug trampolining
            if let Some(debug_step_idx) = ctx.debug_step_idx.filter(|_| !skip_debug) {
                // Trampoline mode
                if let Some(&wasm_idx) = ctx.naga_function_map.get(function) {
                    // 1. Store arguments to memory (if any)
                    // For simplicity, we assume arguments are passed via stack to the trampoline?
                    // No, debug_step doesn't take args.
                    // We need to store args to a known location.
                    // Let's use 0x10000 (64KB) as base for args.
                    // 2. Call debug_step(line, func_idx, result_ptr)
                    ctx.wasm_func.instruction(&Instruction::I32Const(line));
                    ctx.wasm_func
                        .instruction(&Instruction::I32Const(wasm_idx as i32));
                    ctx.wasm_func.instruction(&Instruction::I32Const(0));
                    ctx.wasm_func
                        .instruction(&Instruction::Call(debug_step_idx));

                    // Restore actual call for now (Trampolining requires complex arg marshalling)
                    for arg in arguments {
                        super::expressions::translate_expression(*arg, ctx)?;
                    }
                    ctx.wasm_func.instruction(&Instruction::Call(wasm_idx));

                    // 3. Handle result (if any) exactly as normal call path
                    if let Some(res_handle) = result {
                        if let Some(&local_idx) = ctx.call_result_locals.get(res_handle) {
                            let called_func = &ctx.module.functions[*function];
                            if let Some(ret) = &called_func.result {
                                let types = super::types::naga_to_wasm_types(
                                    &ctx.module.types[ret.ty].inner,
                                )?;
                                for i in (0..types.len()).rev() {
                                    ctx.wasm_func
                                        .instruction(&Instruction::LocalSet(local_idx + i as u32));
                                }
                            }
                        } else {
                            let called_func = &ctx.module.functions[*function];
                            if let Some(ret) = &called_func.result {
                                let types = super::types::naga_to_wasm_types(
                                    &ctx.module.types[ret.ty].inner,
                                )?;
                                for _ in 0..types.len() {
                                    ctx.wasm_func.instruction(&Instruction::Drop);
                                }
                            }
                        }
                    }
                } else {
                    return Err(BackendError::UnsupportedFeature(format!(
                        "Call to unknown function: {:?}",
                        function
                    )));
                }
            } else {
                // Normal Call
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
            // Evaluate condition - it's already I32 (bool), no reinterpret needed
            super::expressions::translate_expression(*condition, ctx)?;

            ctx.wasm_func
                .instruction(&Instruction::If(wasm_encoder::BlockType::Empty));
            for (s, s_span) in accept.span_iter() {
                translate_statement(s, s_span, ctx)?;
            }
            if !reject.is_empty() {
                ctx.wasm_func.instruction(&Instruction::Else);
                for (s, s_span) in reject.span_iter() {
                    translate_statement(s, s_span, ctx)?;
                }
            }
            ctx.wasm_func.instruction(&Instruction::End);
        }
        _ => {}
    }
    Ok(())
}
