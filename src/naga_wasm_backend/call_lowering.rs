//! Call lowering with FunctionABI support.
//!
//! This module implements ABI-aware function call lowering, handling both
//! flattened scalar parameters and frame-based parameter passing for large types.

use super::{frame_allocator, function_abi, BackendError, TranslationContext};
use wasm_encoder::{Instruction, ValType};

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
    // Allocate frame
    // We need two locals: old_sp and aligned_frame_base
    // Compute the i32 scratch base by scanning the local types so we don't
    // rely on declaration ordering assumptions of the encoder.
    let i32_base_pos = ctx
        .local_types
        .iter()
        .position(|t| *t == ValType::I32)
        .unwrap_or(0) as u32;
    let i32_base = ctx.param_count + i32_base_pos;
    let old_sp_local = i32_base; // first i32 scratch slot
    let aligned_local = i32_base + 1; // second i32 scratch slot

    frame_allocator::emit_alloc_frame(
        ctx.wasm_func,
        abi.frame_size,
        abi.frame_alignment,
        old_sp_local,
        aligned_local,
    );

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
            function_abi::ParameterABI::Frame { offset, .. } => {
                // For Frame parameters, we pass a pointer to the frame location
                // TODO: Implement proper copy_in by storing struct/array data into frame
                // For now, pass the frame pointer directly - the callee will access
                // the data from the original location via this pointer

                // Push frame pointer as argument
                ctx.wasm_func
                    .instruction(&Instruction::LocalGet(aligned_local));
                if *offset > 0 {
                    ctx.wasm_func
                        .instruction(&Instruction::I32Const(*offset as i32));
                    ctx.wasm_func.instruction(&Instruction::I32Add);
                }
            }
        }
    }

    // Emit the call
    ctx.wasm_func.instruction(&Instruction::Call(wasm_idx));

    // TODO: Handle copy_out for Frame parameters (for out/inout semantics)
    // This would require tracking the source expression location and
    // copying data back from the frame to the original location

    // Free the frame
    frame_allocator::emit_free_frame(ctx.wasm_func, old_sp_local);

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
        if let Some(&_local_idx) = ctx.call_result_locals.get(res_handle) {
            // Store result into locals
            match result_abi {
                Some(function_abi::ResultABI::Flattened { valtypes, .. }) => {
                    // Store each component in reverse order
                    // Find the first F32 local index to use as base so we don't assume
                    // a particular declaration ordering in the encoder output.
                    // Compute actual F32 base by counting preceding locals that will
                    // appear before F32 locals in the final encoding (I32 locals are emitted first).
                    let num_i32_locals = ctx
                        .local_types
                        .iter()
                        .filter(|t| *t == &ValType::I32)
                        .count() as u32;
                    let f32_base = ctx.param_count + num_i32_locals;
                    eprintln!("[debug] handle_call_result: param_count={} num_i32_locals={} f32_base={} local_types_len={} valtypes_len={}", ctx.param_count, num_i32_locals, f32_base, ctx.local_types.len(), valtypes.len());
                    for i in (0..valtypes.len()).rev() {
                        ctx.wasm_func
                            .instruction(&Instruction::LocalSet(f32_base + i as u32));
                    }
                }
                Some(function_abi::ResultABI::Frame { .. }) => {
                    // Frame results would be written to memory, not returned on stack
                    // No values to pop
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
                    // Frame results don't appear on stack
                }
                None => {
                    // Void result, nothing to drop
                }
            }
        }
    }

    Ok(())
}
