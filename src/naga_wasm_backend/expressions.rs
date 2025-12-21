//! Expression translation from Naga IR to WASM
//!
//! Phase 0: Placeholder for future expression handling

use super::BackendError;

use wasm_encoder::{Instruction, Function};
use naga::{Expression, BinaryOperator, ScalarKind, Literal};
use std::collections::HashMap;
use naga::front::Typifier;

/// Translate a Naga expression component to WASM instructions
pub fn translate_expression_component(
    expr_handle: naga::Handle<Expression>,
    component_idx: u32,
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
    let expr = &func.expressions[expr_handle];
    match expr {
        Expression::Literal(literal) => {
            if component_idx == 0 {
                match literal {
                    Literal::F32(f) => {
                        wasm_func.instruction(&Instruction::F32Const(*f));
                    }
                    Literal::I32(i) => {
                        wasm_func.instruction(&Instruction::I32Const(*i));
                    }
                    Literal::U32(u) => {
                        wasm_func.instruction(&Instruction::I32Const(*u as i32));
                    }
                    Literal::Bool(b) => {
                        wasm_func.instruction(&Instruction::I32Const(if *b { 1 } else { 0 }));
                    }
                    _ => {
                        return Err(BackendError::UnsupportedFeature(format!("Unsupported literal: {:?}", literal)));
                    }
                }
            } else {
                wasm_func.instruction(&Instruction::F32Const(0.0));
            }
        }
        Expression::Constant(c_handle) => {
            let c = &module.constants[*c_handle];
            // let init_expr = &module.global_expressions[c.init];
            // For now, just handle scalar constants
            if component_idx == 0 {
                translate_expression(c.init, func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
            } else {
                wasm_func.instruction(&Instruction::F32Const(0.0));
            }
        }
        Expression::Compose { components, .. } => {
            let mut current_component_idx = component_idx;
            let mut found = false;
            for &comp_handle in components {
                let comp_ty = typifier.get(comp_handle, &module.types);
                let comp_count = super::types::component_count(comp_ty);
                if current_component_idx < comp_count {
                    translate_expression_component(comp_handle, current_component_idx, func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
                    found = true;
                    break;
                }
                current_component_idx -= comp_count;
            }
            if !found {
                wasm_func.instruction(&Instruction::F32Const(0.0));
            }
        }
        Expression::Binary { op, left, right } => {
            let left_ty = typifier.get(*left, &module.types);
            let right_ty = typifier.get(*right, &module.types);
            
            // Handle Matrix * Vector
            if let (naga::TypeInner::Matrix { columns, rows, .. }, naga::TypeInner::Vector { .. }) = (left_ty, right_ty) {
                if *op == BinaryOperator::Multiply {
                    // result[component_idx] = sum_j(matrix[j][component_idx] * vector[j])
                    wasm_func.instruction(&Instruction::F32Const(0.0));
                    for j in 0..(*columns as u32) {
                        // matrix[j][component_idx]
                        translate_expression_component(*left, j * (*rows as u32) + component_idx, func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
                        // vector[j]
                        translate_expression_component(*right, j, func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
                        wasm_func.instruction(&Instruction::F32Mul);
                        wasm_func.instruction(&Instruction::F32Add);
                    }
                    return Ok(());
                }
            }

            let left_count = super::types::component_count(left_ty);
            let right_count = super::types::component_count(right_ty);
            
            let left_idx = if left_count > 1 { component_idx } else { 0 };
            let right_idx = if right_count > 1 { component_idx } else { 0 };

            translate_expression_component(*left, left_idx, func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
            translate_expression_component(*right, right_idx, func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
            
            match op {
                BinaryOperator::Add => {
                    wasm_func.instruction(&Instruction::F32Add);
                }
                BinaryOperator::Subtract => {
                    wasm_func.instruction(&Instruction::F32Sub);
                }
                BinaryOperator::Multiply => {
                    wasm_func.instruction(&Instruction::F32Mul);
                }
                BinaryOperator::Divide => {
                    wasm_func.instruction(&Instruction::F32Div);
                }
                _ => {
                    return Err(BackendError::UnsupportedFeature(format!("Unsupported binary operator: {:?}", op)));
                }
            }
        }
        Expression::Unary { op, expr } => {
            translate_expression_component(*expr, component_idx, func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
            match op {
                naga::UnaryOperator::Negate => {
                    wasm_func.instruction(&Instruction::F32Neg);
                }
                _ => {
                    return Err(BackendError::UnsupportedFeature(format!("Unsupported unary operator: {:?}", op)));
                }
            }
        }
        Expression::FunctionArgument(idx) => {
            if is_entry_point {
                // For entry points, arguments are loaded from memory
                // VS: from attr_ptr (Global 0)
                // FS: from varying_ptr (Global 2)
                if stage == naga::ShaderStage::Vertex {
                    wasm_func.instruction(&Instruction::GlobalGet(0)); // attr_ptr
                } else {
                    wasm_func.instruction(&Instruction::GlobalGet(2)); // varying_ptr
                }
                
                // Calculate offset: sum of sizes of previous arguments
                let mut offset = 0;
                let arg = &func.arguments[*idx as usize];
                if let Some(naga::Binding::Location { location, .. }) = arg.binding {
                    // Use location-based offset
                    if stage == naga::ShaderStage::Vertex {
                        // VS: attribute location L is at offset L * 64 (to match uniform alignment)
                        offset = location * 64;
                    } else {
                        // FS: varying location L is at offset (L + 1) * 16 (skipping gl_Position)
                        offset = (location + 1) * 16;
                    }
                } else {
                    for i in 0..(*idx as usize) {
                        let prev_arg = &func.arguments[i];
                        let prev_arg_ty = &module.types[prev_arg.ty].inner;
                        let prev_size = super::types::type_size(prev_arg_ty).unwrap_or(16);
                        offset += (prev_size + 3) & !3; // 4-byte alignment
                    }
                }
                
                wasm_func.instruction(&Instruction::I32Const((offset + component_idx * 4) as i32));
                wasm_func.instruction(&Instruction::I32Add);
                wasm_func.instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));
            } else {
                // If it's an internal function, we use LocalGet
                let base_idx = argument_local_offsets.get(idx).cloned().unwrap_or(*idx);
                wasm_func.instruction(&Instruction::LocalGet(base_idx + component_idx));
            }
        }
        Expression::GlobalVariable(handle) => {
            if let Some(&(offset, base_ptr_idx)) = global_offsets.get(handle) {
                wasm_func.instruction(&Instruction::GlobalGet(base_ptr_idx));
                let final_offset = offset + component_idx * 4;
                if final_offset > 0 {
                    wasm_func.instruction(&Instruction::I32Const(final_offset as i32));
                    wasm_func.instruction(&Instruction::I32Add);
                }
                
                let ty = &module.global_variables[*handle].ty;
                let inner = &module.types[*ty].inner;
                match inner {
                    naga::TypeInner::Image { .. } | naga::TypeInner::Sampler { .. } => {
                        wasm_func.instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 2,
                            memory_index: 0,
                        }));
                    }
                    _ => {
                        wasm_func.instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 2,
                            memory_index: 0,
                        }));
                    }
                }
            } else {
                wasm_func.instruction(&Instruction::F32Const(0.0));
            }
        }
        Expression::Load { pointer } => {
            translate_expression(*pointer, func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
            if component_idx > 0 {
                wasm_func.instruction(&Instruction::I32Const((component_idx * 4) as i32));
                wasm_func.instruction(&Instruction::I32Add);
            }
            wasm_func.instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                offset: 0,
                align: 2,
                memory_index: 0,
            }));
        }
        Expression::AccessIndex { base, index } => {
            let base_ty = typifier.get(*base, &module.types);
            match base_ty {
                naga::TypeInner::Pointer { .. } => {
                    translate_expression(*base, func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
                    wasm_func.instruction(&Instruction::I32Const((*index * 4 + component_idx * 4) as i32));
                    wasm_func.instruction(&Instruction::I32Add);
                    wasm_func.instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                        offset: 0,
                        align: 2,
                        memory_index: 0,
                    }));
                }
                _ => {
                    // Accessing a component of a value
                    translate_expression_component(*base, *index + component_idx, func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
                }
            }
        }
        Expression::Swizzle { size: _, vector, pattern } => {
            let component = pattern[component_idx as usize];
            translate_expression_component(*vector, component.index(), func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
        }
        Expression::Splat { size: _, value } => {
            translate_expression_component(*value, 0, func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
        }
        Expression::CallResult(handle) => {
            if let Some(&local_idx) = call_result_locals.get(&expr_handle) {
                wasm_func.instruction(&Instruction::LocalGet(local_idx + component_idx));
            } else {
                wasm_func.instruction(&Instruction::F32Const(0.0));
            }
        }
        Expression::ImageSample { image, coordinate, .. } => {
            // Basic 2D texture sampling
            // 1. Get texture unit index (from uniform)
            translate_expression_component(*image, 0, func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
            wasm_func.instruction(&Instruction::I32TruncF32S);
            
            // 2. Calculate descriptor address: texture_ptr + unit_idx * 32
            wasm_func.instruction(&Instruction::I32Const(32));
            wasm_func.instruction(&Instruction::I32Mul);
            wasm_func.instruction(&Instruction::GlobalGet(4)); // texture_ptr
            wasm_func.instruction(&Instruction::I32Add);
            wasm_func.instruction(&Instruction::LocalSet(scratch_base)); // scratch_base is desc_addr
            
            // 3. Load width and height
            wasm_func.instruction(&Instruction::LocalGet(scratch_base));
            wasm_func.instruction(&Instruction::I32Load(wasm_encoder::MemArg { offset: 0, align: 2, memory_index: 0 }));
            wasm_func.instruction(&Instruction::LocalSet(scratch_base + 1)); // scratch_base + 1 is width
            
            wasm_func.instruction(&Instruction::LocalGet(scratch_base));
            wasm_func.instruction(&Instruction::I32Load(wasm_encoder::MemArg { offset: 4, align: 2, memory_index: 0 }));
            wasm_func.instruction(&Instruction::LocalSet(scratch_base + 2)); // scratch_base + 2 is height
            
            // 4. Load data_ptr
            wasm_func.instruction(&Instruction::LocalGet(scratch_base));
            wasm_func.instruction(&Instruction::I32Load(wasm_encoder::MemArg { offset: 8, align: 2, memory_index: 0 }));
            wasm_func.instruction(&Instruction::LocalSet(scratch_base + 3)); // scratch_base + 3 is data_ptr
            
            // 5. Calculate texel coordinates
            // u = coordinate.x, v = coordinate.y
            translate_expression_component(*coordinate, 0, func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
            // Clamp u to [0, 0.9999] to avoid out of bounds at 1.0
            wasm_func.instruction(&Instruction::F32Const(0.0));
            wasm_func.instruction(&Instruction::F32Max);
            wasm_func.instruction(&Instruction::F32Const(0.9999));
            wasm_func.instruction(&Instruction::F32Min);
            wasm_func.instruction(&Instruction::LocalGet(scratch_base + 1)); // width
            wasm_func.instruction(&Instruction::F32ConvertI32S);
            wasm_func.instruction(&Instruction::F32Mul);
            wasm_func.instruction(&Instruction::I32TruncF32S);
            wasm_func.instruction(&Instruction::LocalSet(scratch_base + 4)); // scratch_base + 4 is texel_x
            
            translate_expression_component(*coordinate, 1, func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
            // Clamp v to [0, 0.9999]
            wasm_func.instruction(&Instruction::F32Const(0.0));
            wasm_func.instruction(&Instruction::F32Max);
            wasm_func.instruction(&Instruction::F32Const(0.9999));
            wasm_func.instruction(&Instruction::F32Min);
            wasm_func.instruction(&Instruction::LocalGet(scratch_base + 2)); // height
            wasm_func.instruction(&Instruction::F32ConvertI32S);
            wasm_func.instruction(&Instruction::F32Mul);
            wasm_func.instruction(&Instruction::I32TruncF32S);
            wasm_func.instruction(&Instruction::LocalSet(scratch_base + 5)); // scratch_base + 5 is texel_y
            
            // 6. Calculate pixel address: data_ptr + (texel_y * width + texel_x) * 4
            wasm_func.instruction(&Instruction::LocalGet(scratch_base + 5)); // texel_y
            wasm_func.instruction(&Instruction::LocalGet(scratch_base + 1)); // width
            wasm_func.instruction(&Instruction::I32Mul);
            wasm_func.instruction(&Instruction::LocalGet(scratch_base + 4)); // texel_x
            wasm_func.instruction(&Instruction::I32Add);
            wasm_func.instruction(&Instruction::I32Const(4));
            wasm_func.instruction(&Instruction::I32Mul);
            wasm_func.instruction(&Instruction::LocalGet(scratch_base + 3)); // data_ptr
            wasm_func.instruction(&Instruction::I32Add);
            // Stack: [pixel_addr]
            
            // 7. Load component
            // Pixel is RGBA u8. We need to convert to f32 [0, 1].
            wasm_func.instruction(&Instruction::I32Load8U(wasm_encoder::MemArg { offset: component_idx as u64, align: 0, memory_index: 0 }));
            wasm_func.instruction(&Instruction::F32ConvertI32U);
            wasm_func.instruction(&Instruction::F32Const(255.0));
            wasm_func.instruction(&Instruction::F32Div);
        }
        _ => {
            wasm_func.instruction(&Instruction::F32Const(0.0));
        }
    }
    Ok(())
}

/// Translate a Naga expression to WASM instructions
pub fn translate_expression(
    expr_handle: naga::Handle<Expression>,
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
    let expr = &func.expressions[expr_handle];
    let ty = typifier.get(expr_handle, &module.types);

    // If it's a pointer type, we want the address
    if let naga::TypeInner::Pointer { .. } = ty {
        match expr {
            Expression::LocalVariable(handle) => {
                wasm_func.instruction(&Instruction::GlobalGet(3)); // private_ptr
                if let Some(&offset) = local_offsets.get(handle) {
                    if offset > 0 {
                        wasm_func.instruction(&Instruction::I32Const(offset as i32));
                        wasm_func.instruction(&Instruction::I32Add);
                    }
                }
            }
            Expression::GlobalVariable(handle) => {
                if let Some(&(offset, base_ptr_idx)) = global_offsets.get(handle) {
                    wasm_func.instruction(&Instruction::GlobalGet(base_ptr_idx));
                    if offset > 0 {
                        wasm_func.instruction(&Instruction::I32Const(offset as i32));
                        wasm_func.instruction(&Instruction::I32Add);
                    }
                } else {
                    // Fallback for unknown globals
                    let var = &module.global_variables[*handle];
                    if var.name.as_deref() == Some("gl_Position") || var.name.as_deref() == Some("gl_Position_1") {
                        wasm_func.instruction(&Instruction::GlobalGet(2)); // varying_ptr
                    } else {
                        wasm_func.instruction(&Instruction::GlobalGet(3)); // private_ptr
                    }
                }
            }
            Expression::AccessIndex { base, index } => {
                translate_expression(*base, func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
                // Assume each index is 4 bytes (float)
                if *index > 0 {
                    wasm_func.instruction(&Instruction::I32Const((*index * 4) as i32));
                    wasm_func.instruction(&Instruction::I32Add);
                }
            }
            _ => {
                return Err(BackendError::UnsupportedFeature(format!("Unsupported pointer expression: {:?}", expr)));
            }
        }
        return Ok(());
    }

    // Otherwise, it's a value. Loop over components.
    let count = super::types::component_count(ty);
    for i in 0..count {
        translate_expression_component(expr_handle, i, func, module, wasm_func, global_offsets, local_offsets, call_result_locals, stage, typifier, naga_function_map, argument_local_offsets, is_entry_point, scratch_base)?;
    }
    Ok(())
}
