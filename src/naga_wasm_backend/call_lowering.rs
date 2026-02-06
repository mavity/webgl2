//! Call lowering with FunctionABI support.
//!
//! This module implements ABI-aware function call lowering, handling both
//! flattened scalar parameters and frame-based parameter passing for large types.

use super::{function_abi, output_layout, types, BackendError, TranslationContext};
use wasm_encoder::{Instruction, MemArg, ValType};

/// Emit a function call using FunctionABI for proper parameter/result handling.
///
/// This function inspects the FunctionABI to determine how to pass parameters:
/// - Flattened parameters: pushed directly onto the WASM stack
/// - Frame parameters: allocated on the frame stack, data copied to frame
///
/// # Arguments
/// - `function`: Handle to the Naga function being called
/// - `arguments`: Expression handles for the call arguments
/// - `result`: Optional expression handle for storing the result
/// - `abi`: The computed FunctionABI for this function
/// - `wasm_idx`: WASM function index to call
/// - `ctx`: Translation context
pub fn emit_abi_call(
    function: &naga::Handle<naga::Function>,
    arguments: &[naga::Handle<naga::Expression>],
    result: &Option<naga::Handle<naga::Expression>>,
    abi: &function_abi::FunctionABI,
    wasm_idx: u32,
    ctx: &mut TranslationContext,
) -> Result<(), BackendError> {
    // Check if frame allocation is needed
    if abi.uses_frame {
        emit_frame_based_call(function, arguments, result, abi, wasm_idx, ctx)
    } else {
        emit_flattened_call(function, arguments, result, abi, wasm_idx, ctx)
    }
}

/// Emit a call where all parameters are flattened (the common case).
fn emit_flattened_call(
    _function: &naga::Handle<naga::Function>,
    arguments: &[naga::Handle<naga::Expression>],
    result: &Option<naga::Handle<naga::Expression>>,
    abi: &function_abi::FunctionABI,
    wasm_idx: u32,
    ctx: &mut TranslationContext,
) -> Result<(), BackendError> {
    // Push all arguments (they're already flattened by translate_expression)
    for arg in arguments {
        super::expressions::translate_expression(*arg, ctx)?;
    }

    // Emit the call
    ctx.wasm_func.instruction(&Instruction::Call(wasm_idx));

    // Handle result
    handle_call_result(result, &abi.result, ctx)
}

/// Emit a call that requires frame allocation for large parameters.
fn emit_frame_based_call(
    _function: &naga::Handle<naga::Function>,
    arguments: &[naga::Handle<naga::Expression>],
    result: &Option<naga::Handle<naga::Expression>>,
    abi: &function_abi::FunctionABI,
    wasm_idx: u32,
    ctx: &mut TranslationContext,
) -> Result<(), BackendError> {
    // --- Inline frame allocation (no locals needed) ---
    // 1. Load current FRAME_SP
    ctx.wasm_func
        .instruction(&Instruction::GlobalGet(output_layout::FRAME_SP_GLOBAL));

    // 2. Align: (sp + align - 1) & ~(align - 1)
    ctx.wasm_func
        .instruction(&Instruction::I32Const((abi.frame_alignment - 1) as i32));
    ctx.wasm_func.instruction(&Instruction::I32Add);
    ctx.wasm_func
        .instruction(&Instruction::I32Const(!(abi.frame_alignment - 1) as i32));
    ctx.wasm_func.instruction(&Instruction::I32And);
    // Stack: [aligned_base]

    // 3. Advance FRAME_SP: set to aligned_base + frame_size
    // Use local.tee to keep aligned_base on stack while also storing in a temp
    // We use an explicit temp I32 local allocated for this purpose
    let aligned_temp = ctx.frame_temp_idx.expect("frame_temp_idx not allocated");

    ctx.wasm_func
        .instruction(&Instruction::LocalTee(aligned_temp));
    ctx.wasm_func
        .instruction(&Instruction::I32Const(abi.frame_size as i32));
    ctx.wasm_func.instruction(&Instruction::I32Add);
    ctx.wasm_func
        .instruction(&Instruction::GlobalSet(output_layout::FRAME_SP_GLOBAL));
    // Stack: [] (aligned_base now in aligned_temp)

    // Process each argument according to its ABI
    for (arg_idx, arg_expr) in arguments.iter().enumerate() {
        if arg_idx >= abi.params.len() {
            return Err(BackendError::UnsupportedFeature(format!(
                "Argument count mismatch: {} args but {} params in ABI",
                arguments.len(),
                abi.params.len()
            )));
        }

        match &abi.params[arg_idx] {
            function_abi::ParameterABI::Flattened { .. } => {
                // Push flattened argument onto stack
                super::expressions::translate_expression(*arg_expr, ctx)?;
            }
            function_abi::ParameterABI::Frame {
                offset,
                copy_in,
                semantic,
                ..
            } => {
                // Copy data into frame if needed
                if *copy_in {
                    copy_value_to_frame(*arg_expr, aligned_temp, *offset, ctx)?;
                }

                // Push frame pointer as argument
                ctx.wasm_func
                    .instruction(&Instruction::LocalGet(aligned_temp));
                if *offset > 0 {
                    ctx.wasm_func
                        .instruction(&Instruction::I32Const(*offset as i32));
                    ctx.wasm_func.instruction(&Instruction::I32Add);
                }

                // For Out parameters, we may need to handle the original expression
                // location for copy_out later
                if matches!(semantic, function_abi::ParamSemantic::Out)
                    || matches!(semantic, function_abi::ParamSemantic::InOut)
                {
                    // Store metadata for copy_out (not implemented yet)
                    // This would require tracking expression -> memory location mapping
                }
            }
        }
    }

    // Emit the call
    ctx.wasm_func.instruction(&Instruction::Call(wasm_idx));

    // Handle copy_out for Frame parameters with Out/InOut semantics
    for (arg_idx, arg_expr) in arguments.iter().enumerate() {
        if arg_idx >= abi.params.len() {
            continue;
        }

        if let function_abi::ParameterABI::Frame {
            offset,
            copy_out,
            semantic,
            ..
        } = &abi.params[arg_idx]
        {
            if *copy_out
                && (matches!(semantic, function_abi::ParamSemantic::Out)
                    || matches!(semantic, function_abi::ParamSemantic::InOut))
            {
                copy_value_from_frame(*arg_expr, aligned_temp, *offset, ctx)?;
            }
        }
    }

    // --- Inline frame deallocation (no locals needed) ---
    // Restore FRAME_SP by subtracting frame_size (we know it at compile time)
    ctx.wasm_func
        .instruction(&Instruction::GlobalGet(output_layout::FRAME_SP_GLOBAL));
    ctx.wasm_func
        .instruction(&Instruction::I32Const(abi.frame_size as i32));
    ctx.wasm_func.instruction(&Instruction::I32Sub);
    ctx.wasm_func
        .instruction(&Instruction::GlobalSet(output_layout::FRAME_SP_GLOBAL));

    // Handle result
    handle_call_result(result, &abi.result, ctx)
}

/// Handle storing or dropping the call result based on ABI.
fn handle_call_result(
    result: &Option<naga::Handle<naga::Expression>>,
    result_abi: &Option<function_abi::ResultABI>,
    ctx: &mut TranslationContext,
) -> Result<(), BackendError> {
    if let Some(res_handle) = result {
        if let Some(&runtime_base) = ctx.call_result_locals.get(res_handle) {
            // Store result into locals
            // call_result_locals now stores runtime indices directly
            match result_abi {
                Some(function_abi::ResultABI::Flattened { valtypes, .. }) => {
                    // Store each component in reverse order
                    for i in (0..valtypes.len()).rev() {
                        ctx.wasm_func
                            .instruction(&Instruction::LocalSet(runtime_base + i as u32));
                    }
                }
                Some(function_abi::ResultABI::Frame { size, align }) => {
                    // Frame results are written to memory by the callee.
                    // We need to load them from the frame and store to result locals.
                    // The frame pointer should be passed to the callee, and the callee
                    // writes the result there.
                    //
                    // For now, this is a placeholder - full implementation requires
                    // the frame pointer to be accessible here (either passed as hidden param
                    // or stored in a known location).
                    //
                    // TODO: Load from frame memory and store to runtime_base locals
                    let _ = (size, align, runtime_base);
                }
                None => {
                    // Void result, nothing to store
                }
            }
        } else {
            // Result not used, drop it
            match result_abi {
                Some(function_abi::ResultABI::Flattened { valtypes, .. }) => {
                    for _ in 0..valtypes.len() {
                        ctx.wasm_func.instruction(&Instruction::Drop);
                    }
                }
                Some(function_abi::ResultABI::Frame { .. }) => {
                    // Frame results don't appear on stack, nothing to drop
                }
                None => {
                    // Void result, nothing to drop
                }
            }
        }
    }

    Ok(())
}
/// Copy a value from the WASM stack to the frame at the specified offset.
///
/// This function evaluates the expression (pushing its flattened scalar components
/// onto the stack), then stores each component to the frame in memory.
fn copy_value_to_frame(
    expr: naga::Handle<naga::Expression>,
    frame_base_local: u32,
    frame_offset: u32,
    ctx: &mut TranslationContext,
) -> Result<(), BackendError> {
    // Get the type of the expression
    let type_inner = ctx.typifier.get(expr, &ctx.module.types);

    // For types that can be resolved to a handle, get the component count
    // Otherwise, compute directly from the inner type
    let count = types::component_count(type_inner, &ctx.module.types);

    // We need a type handle to use get_flat_component_type
    // Try to find it in the module types by matching the inner type
    let type_handle = ctx
        .module
        .types
        .iter()
        .find(|(_, ty)| &ty.inner == type_inner)
        .map(|(handle, _)| handle);

    for i in 0..count {
        // Determine the value type for this component
        let val_type = if let Some(handle) = type_handle {
            types::get_flat_component_type(handle, i, &ctx.module.types)?
        } else {
            // Fallback: assume F32 for unknown types (common case)
            ValType::F32
        };

        // 1. Push address: frame_base + offset + i*4
        let offset = frame_offset + (i as u32 * 4);
        ctx.wasm_func
            .instruction(&Instruction::LocalGet(frame_base_local));
        if offset > 0 {
            ctx.wasm_func
                .instruction(&Instruction::I32Const(offset as i32));
            ctx.wasm_func.instruction(&Instruction::I32Add);
        }

        // 2. Push value (one component)
        super::expressions::translate_expression_component(expr, i, ctx)?;

        // 3. Store to memory
        let memarg = MemArg {
            offset: 0,
            align: 2, // 4-byte alignment (2^2 = 4)
            memory_index: 0,
        };
        match val_type {
            ValType::F32 => {
                ctx.wasm_func.instruction(&Instruction::F32Store(memarg));
            }
            ValType::I32 => {
                ctx.wasm_func.instruction(&Instruction::I32Store(memarg));
            }
            _ => unreachable!(),
        };
    }

    Ok(())
}

/// Copy a value from the frame back to its original location (for Out/InOut parameters).
///
/// This is a placeholder - full implementation requires tracking the destination
/// address or local variable for each argument expression.
fn copy_value_from_frame(
    _expr: naga::Handle<naga::Expression>,
    _frame_base_local: u32,
    _frame_offset: u32,
    _ctx: &mut TranslationContext,
) -> Result<(), BackendError> {
    // TODO: Implement copy_out logic
    // This requires:
    // 1. Determining the destination address (if expr is a pointer dereference)
    // 2. Loading values from frame memory
    // 3. Storing values to the destination
    //
    // For now, Out/InOut parameters are rarely used in typical GLSL shaders,
    // so this is deferred to a future implementation.
    Ok(())
}
