//! Control flow translation from Naga IR to WASM
//!
//! This module handles translation of Naga control flow statements (if, return, etc.)
//! into WebAssembly instructions, with special handling for shader output values.

use super::{call_lowering::emit_abi_call, output_layout, BackendError, TranslationContext};

use wasm_encoder::Instruction;

// Global indices for shader memory pointers
#[allow(dead_code)]
const ATTR_PTR_GLOBAL: u32 = 0;
#[allow(dead_code)]
const UNIFORM_PTR_GLOBAL: u32 = 1;
const VARYING_PTR_GLOBAL: u32 = output_layout::VARYING_PTR_GLOBAL;
const PRIVATE_PTR_GLOBAL: u32 = output_layout::PRIVATE_PTR_GLOBAL;
#[allow(dead_code)]
const TEXTURE_PTR_GLOBAL: u32 = 4;

/// Helper function to determine output destination and offset for a binding.
///
/// This is a thin wrapper around the centralized output_layout module,
/// maintaining compatibility with the existing control flow code.
///
/// Returns a tuple `(offset, base_ptr_index)` where:
/// - `offset`: Byte offset within the memory region
/// - `base_ptr_index`: Global index of the memory pointer (2=varying_ptr, 3=private_ptr)
fn get_output_destination(binding: &naga::Binding, ctx: &TranslationContext) -> (u32, u32) {
    output_layout::compute_output_destination(binding, ctx.stage)
}

/// Helper function to store components to memory.
///
/// # Parameters
/// - `offset`: Byte offset within the memory region pointed to by base_ptr
/// - `base_ptr`: Global index of memory pointer (2=varying_ptr, 3=private_ptr)
/// - `num_components`: Number of components (floats or ints) to store
/// - `is_int`: Whether components are integers (true) or floats (false)
/// - `ctx`: Translation context containing WASM function builder
///
/// This function pops values from the WASM stack (in reverse order) and stores them
/// to the specified memory location using either I32Store or F32Store instructions.
fn store_components_to_memory(
    offset: u32,
    base_ptr: u32,
    num_components: u32,
    is_int: bool,
    ctx: &mut TranslationContext,
) {
    // Use explicit swap locals from context (allocated at END of locals array)
    let swap_local = if is_int {
        ctx.swap_i32_local
    } else {
        ctx.swap_f32_local
    };

    if base_ptr == VARYING_PTR_GLOBAL || base_ptr == PRIVATE_PTR_GLOBAL {
        // Handle varyings and fragment outputs
        // Store components in reverse order
        for i in (0..num_components).rev() {
            // Pop value to swap local
            ctx.wasm_func
                .instruction(&Instruction::LocalSet(swap_local));

            // Calculate byte offset for this component
            let comp_offset = offset + (i * 4);

            // Load base pointer from global
            ctx.wasm_func.instruction(&Instruction::GlobalGet(base_ptr));

            // Push value from swap local and store
            ctx.wasm_func
                .instruction(&Instruction::LocalGet(swap_local));

            if is_int {
                ctx.wasm_func
                    .instruction(&Instruction::I32Store(wasm_encoder::MemArg {
                        offset: comp_offset as u64,
                        align: 2,
                        memory_index: 0,
                    }));
            } else {
                ctx.wasm_func
                    .instruction(&Instruction::F32Store(wasm_encoder::MemArg {
                        offset: comp_offset as u64,
                        align: 2,
                        memory_index: 0,
                    }));
            }
        }
    } else {
        // Drop values if not handled
        for _ in 0..num_components {
            ctx.wasm_func.instruction(&Instruction::Drop);
        }
    }
}

/// Store a single scalar/vector function result into the appropriate output
/// destination using existing layout helpers.
///
/// This helper centralizes the logic that was previously duplicated at
/// several call sites. It expects `binding` to be present (WGSL Case B)
/// and will call `get_output_destination` and `store_components_to_memory`.
fn store_single_value_to_output(
    ty: &naga::Type,
    binding: &naga::Binding,
    ctx: &mut TranslationContext,
) {
    let num_components = super::types::component_count(&ty.inner, &ctx.module.types);
    let is_int = super::expressions::is_integer_type(&ty.inner);
    let (offset, base_ptr) = get_output_destination(binding, ctx);
    store_components_to_memory(offset, base_ptr, num_components, is_int, ctx);
}

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
            // Debug: Check if storing to global
            if let naga::Expression::GlobalVariable(handle) = ctx.func.expressions[*pointer] {
                if let Some((_offset, _base_ptr)) = ctx.global_offsets.get(&handle) {
                    let _name = ctx.module.global_variables[handle]
                        .name
                        .as_deref()
                        .unwrap_or("?");
                }
            }

            // Determine how many components to store
            let value_ty = ctx.typifier.get(*value, &ctx.module.types);
            let num_components = super::types::component_count(value_ty, &ctx.module.types);

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
                        if let Some(&runtime_base) = ctx.call_result_locals.get(res_handle) {
                            let called_func = &ctx.module.functions[*function];
                            if let Some(ret) = &called_func.result {
                                let types = super::types::naga_to_wasm_types(
                                    &ctx.module.types[ret.ty].inner,
                                )?;
                                // call_result_locals now stores runtime indices directly
                                for i in (0..types.len()).rev() {
                                    ctx.wasm_func.instruction(&Instruction::LocalSet(
                                        runtime_base + i as u32,
                                    ));
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
                // Normal Call - use FunctionABI for proper lowering
                if let Some(&wasm_idx) = ctx.naga_function_map.get(function) {
                    // Check if we have ABI info for this function
                    if let Some(abi) = ctx.function_abis.get(function) {
                        // ABI-aware call lowering
                        emit_abi_call(function, arguments, result, abi, wasm_idx, ctx)?;
                    } else {
                        // Fallback for functions without ABI (shouldn't happen for internal functions)
                        // Push arguments
                        for arg in arguments {
                            super::expressions::translate_expression(*arg, ctx)?;
                        }
                        // Call
                        ctx.wasm_func.instruction(&Instruction::Call(wasm_idx));
                        // Handle result if any
                        if let Some(res_handle) = result {
                            if let Some(&runtime_base) = ctx.call_result_locals.get(res_handle) {
                                let called_func = &ctx.module.functions[*function];
                                if let Some(ret) = &called_func.result {
                                    let types = super::types::naga_to_wasm_types(
                                        &ctx.module.types[ret.ty].inner,
                                    )?;
                                    // call_result_locals now stores runtime indices directly
                                    for i in (0..types.len()).rev() {
                                        ctx.wasm_func.instruction(&Instruction::LocalSet(
                                            runtime_base + i as u32,
                                        ));
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
                    }
                } else {
                    return Err(BackendError::UnsupportedFeature(format!(
                        "Call to unknown function: {:?}",
                        function
                    )));
                }
            }
        }
        naga::Statement::Return { value } => {
            if let Some(expr_handle) = value {
                if ctx.is_entry_point {
                    // Translate expression to push results to stack
                    super::expressions::translate_expression(*expr_handle, ctx)?;

                    // Store results to memory
                    if let Some(result) = &ctx.func.result {
                        let ty = &ctx.module.types[result.ty];
                        match &ty.inner {
                            naga::TypeInner::Struct { members, .. } => {
                                // Iterate in reverse order because stack is LIFO
                                for member in members.iter().rev() {
                                    let member_ty = &ctx.module.types[member.ty];
                                    let num_components = super::types::component_count(
                                        &member_ty.inner,
                                        &ctx.module.types,
                                    );
                                    let is_int =
                                        super::expressions::is_integer_type(&member_ty.inner);

                                    // Determine offset and base pointer based on binding and shader stage
                                    let (offset, base_ptr) = if let Some(binding) = &member.binding
                                    {
                                        get_output_destination(binding, ctx)
                                    } else {
                                        (0, 0)
                                    };

                                    store_components_to_memory(
                                        offset,
                                        base_ptr,
                                        num_components,
                                        is_int,
                                        ctx,
                                    );
                                }
                            }
                            _ => {
                                // Handle single return value with binding (WGSL Case B)
                                if let Some(binding) = &result.binding {
                                    let num_components =
                                        super::types::component_count(&ty.inner, &ctx.module.types);
                                    let is_int = super::expressions::is_integer_type(&ty.inner);

                                    let (offset, base_ptr) = get_output_destination(binding, ctx);
                                    store_components_to_memory(
                                        offset,
                                        base_ptr,
                                        num_components,
                                        is_int,
                                        ctx,
                                    );
                                } else {
                                    // No binding, drop the values
                                    let num_components =
                                        super::types::component_count(&ty.inner, &ctx.module.types);
                                    if let Some(binding) =
                                        &ctx.func.result.as_ref().and_then(|r| r.binding.clone())
                                    {
                                        store_single_value_to_output(ty, binding, ctx);
                                    } else {
                                        for _ in 0..num_components {
                                            ctx.wasm_func.instruction(&Instruction::Drop);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
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
