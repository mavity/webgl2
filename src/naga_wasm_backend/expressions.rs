//! Expression translation from Naga IR to WASM
//!
//! Phase 0: Placeholder for future expression handling

use super::{BackendError, TranslationContext};

use naga::{BinaryOperator, Expression, Literal};
use wasm_encoder::Instruction;

/// Translate a Naga expression component to WASM instructions
pub fn translate_expression_component(
    expr_handle: naga::Handle<Expression>,
    component_idx: u32,
    ctx: &mut TranslationContext,
) -> Result<(), BackendError> {
    let expr = &ctx.func.expressions[expr_handle];
    match expr {
        Expression::Literal(literal) => {
            if component_idx == 0 {
                match literal {
                    Literal::F32(f) => {
                        ctx.wasm_func.instruction(&Instruction::F32Const(*f));
                    }
                    Literal::I32(i) => {
                        ctx.wasm_func.instruction(&Instruction::I32Const(*i));
                        ctx.wasm_func.instruction(&Instruction::F32ReinterpretI32);
                    }
                    Literal::U32(u) => {
                        ctx.wasm_func.instruction(&Instruction::I32Const(*u as i32));
                        ctx.wasm_func.instruction(&Instruction::F32ReinterpretI32);
                    }
                    Literal::Bool(b) => {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Const(if *b { 1 } else { 0 }));
                        ctx.wasm_func.instruction(&Instruction::F32ReinterpretI32);
                    }
                    _ => {
                        return Err(BackendError::UnsupportedFeature(format!(
                            "Unsupported literal: {:?}",
                            literal
                        )));
                    }
                }
            } else {
                ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
            }
        }
        Expression::Constant(c_handle) => {
            let c = &ctx.module.constants[*c_handle];
            // let init_expr = &module.global_expressions[c.init];
            // For now, just handle scalar constants
            if component_idx == 0 {
                translate_expression(c.init, ctx)?;
            } else {
                ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
            }
        }
        Expression::Compose { components, .. } => {
            let mut current_component_idx = component_idx;
            let mut found = false;
            for &comp_handle in components {
                let comp_ty = ctx.typifier.get(comp_handle, &ctx.module.types);
                let comp_count = super::types::component_count(comp_ty);
                if current_component_idx < comp_count {
                    translate_expression_component(comp_handle, current_component_idx, ctx)?;
                    found = true;
                    break;
                }
                current_component_idx -= comp_count;
            }
            if !found {
                ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
            }
        }
        Expression::Binary { op, left, right } => {
            let left_ty = ctx.typifier.get(*left, &ctx.module.types);
            let right_ty = ctx.typifier.get(*right, &ctx.module.types);

            let left_scalar_kind = match left_ty {
                naga::TypeInner::Scalar(scalar) => scalar.kind,
                naga::TypeInner::Vector { scalar, .. } => scalar.kind,
                _ => naga::ScalarKind::Float,
            };

            // Handle Matrix * Vector
            if let (naga::TypeInner::Matrix { columns, rows, .. }, naga::TypeInner::Vector { .. }) =
                (left_ty, right_ty)
            {
                if *op == BinaryOperator::Multiply {
                    // result[component_idx] = sum_j(matrix[j][component_idx] * vector[j])
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    for j in 0..(*columns as u32) {
                        // matrix[j][component_idx]
                        translate_expression_component(
                            *left,
                            j * (*rows as u32) + component_idx,
                            ctx,
                        )?;
                        // vector[j]
                        translate_expression_component(*right, j, ctx)?;
                        ctx.wasm_func.instruction(&Instruction::F32Mul);
                        ctx.wasm_func.instruction(&Instruction::F32Add);
                    }
                    return Ok(());
                }
            }

            let left_count = super::types::component_count(left_ty);
            let right_count = super::types::component_count(right_ty);

            let left_idx = if left_count > 1 { component_idx } else { 0 };
            let right_idx = if right_count > 1 { component_idx } else { 0 };

            translate_expression_component(*left, left_idx, ctx)?;
            if matches!(
                left_scalar_kind,
                naga::ScalarKind::Sint | naga::ScalarKind::Uint | naga::ScalarKind::Bool
            ) {
                ctx.wasm_func.instruction(&Instruction::I32ReinterpretF32);
            }

            translate_expression_component(*right, right_idx, ctx)?;
            if matches!(
                left_scalar_kind,
                naga::ScalarKind::Sint | naga::ScalarKind::Uint | naga::ScalarKind::Bool
            ) {
                ctx.wasm_func.instruction(&Instruction::I32ReinterpretF32);
            }

            match op {
                BinaryOperator::Add => match left_scalar_kind {
                    naga::ScalarKind::Float => {
                        ctx.wasm_func.instruction(&Instruction::F32Add);
                    }
                    naga::ScalarKind::Sint | naga::ScalarKind::Uint => {
                        ctx.wasm_func.instruction(&Instruction::I32Add);
                        ctx.wasm_func.instruction(&Instruction::F32ReinterpretI32);
                    }
                    _ => {}
                },
                BinaryOperator::Subtract => match left_scalar_kind {
                    naga::ScalarKind::Float => {
                        ctx.wasm_func.instruction(&Instruction::F32Sub);
                    }
                    naga::ScalarKind::Sint | naga::ScalarKind::Uint => {
                        ctx.wasm_func.instruction(&Instruction::I32Sub);
                        ctx.wasm_func.instruction(&Instruction::F32ReinterpretI32);
                    }
                    _ => {}
                },
                BinaryOperator::Multiply => match left_scalar_kind {
                    naga::ScalarKind::Float => {
                        ctx.wasm_func.instruction(&Instruction::F32Mul);
                    }
                    naga::ScalarKind::Sint | naga::ScalarKind::Uint => {
                        ctx.wasm_func.instruction(&Instruction::I32Mul);
                        ctx.wasm_func.instruction(&Instruction::F32ReinterpretI32);
                    }
                    _ => {}
                },
                BinaryOperator::Divide => match left_scalar_kind {
                    naga::ScalarKind::Float => {
                        ctx.wasm_func.instruction(&Instruction::F32Div);
                    }
                    naga::ScalarKind::Sint => {
                        ctx.wasm_func.instruction(&Instruction::I32DivS);
                        ctx.wasm_func.instruction(&Instruction::F32ReinterpretI32);
                    }
                    naga::ScalarKind::Uint => {
                        ctx.wasm_func.instruction(&Instruction::I32DivU);
                        ctx.wasm_func.instruction(&Instruction::F32ReinterpretI32);
                    }
                    _ => {}
                },
                BinaryOperator::Equal => {
                    match left_scalar_kind {
                        naga::ScalarKind::Float => {
                            ctx.wasm_func.instruction(&Instruction::F32Eq);
                        }
                        naga::ScalarKind::Sint
                        | naga::ScalarKind::Uint
                        | naga::ScalarKind::Bool => {
                            ctx.wasm_func.instruction(&Instruction::I32Eq);
                        }
                        _ => {}
                    }
                    ctx.wasm_func.instruction(&Instruction::F32ReinterpretI32);
                }
                BinaryOperator::NotEqual => {
                    match left_scalar_kind {
                        naga::ScalarKind::Float => {
                            ctx.wasm_func.instruction(&Instruction::F32Ne);
                        }
                        naga::ScalarKind::Sint
                        | naga::ScalarKind::Uint
                        | naga::ScalarKind::Bool => {
                            ctx.wasm_func.instruction(&Instruction::I32Ne);
                        }
                        _ => {}
                    }
                    ctx.wasm_func.instruction(&Instruction::F32ReinterpretI32);
                }
                _ => {
                    return Err(BackendError::UnsupportedFeature(format!(
                        "Unsupported binary operator: {:?}",
                        op
                    )));
                }
            }
        }
        Expression::Unary { op, expr } => {
            translate_expression_component(*expr, component_idx, ctx)?;
            match op {
                naga::UnaryOperator::Negate => {
                    ctx.wasm_func.instruction(&Instruction::F32Neg);
                }
                _ => {
                    return Err(BackendError::UnsupportedFeature(format!(
                        "Unsupported unary operator: {:?}",
                        op
                    )));
                }
            }
        }
        Expression::FunctionArgument(idx) => {
            if ctx.is_entry_point {
                // For entry points, arguments are loaded from memory
                // VS: from attr_ptr (Global 0)
                // FS: from varying_ptr (Global 2)
                if ctx.stage == naga::ShaderStage::Vertex {
                    ctx.wasm_func.instruction(&Instruction::GlobalGet(0)); // attr_ptr
                } else {
                    ctx.wasm_func.instruction(&Instruction::GlobalGet(2)); // varying_ptr
                }

                // Calculate offset: sum of sizes of previous arguments
                let mut offset = 0;
                let arg = &ctx.func.arguments[*idx as usize];
                let mut found_location = false;

                if ctx.stage == naga::ShaderStage::Vertex {
                    if let Some(name) = &arg.name {
                        if let Some(&location) = ctx.attribute_locations.get(name) {
                            offset = location * 64;
                            found_location = true;
                        }
                    }
                }

                if !found_location {
                    if let Some(naga::Binding::Location { location, .. }) = arg.binding {
                        // Use location-based offset
                        if ctx.stage == naga::ShaderStage::Vertex {
                            // VS: attribute location L is at offset L * 64 (to match uniform alignment)
                            offset = location * 64;
                        } else {
                            // FS: varying location L is at offset (L + 1) * 16 (skipping gl_Position)
                            offset = (location + 1) * 16;
                        }
                    } else {
                        for i in 0..(*idx as usize) {
                            let prev_arg = &ctx.func.arguments[i];
                            let prev_arg_ty = &ctx.module.types[prev_arg.ty].inner;
                            let prev_size = super::types::type_size(prev_arg_ty).unwrap_or(16);
                            offset += (prev_size + 3) & !3; // 4-byte alignment
                        }
                    }
                }

                ctx.wasm_func
                    .instruction(&Instruction::I32Const((offset + component_idx * 4) as i32));
                ctx.wasm_func.instruction(&Instruction::I32Add);
                ctx.wasm_func
                    .instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                        offset: 0,
                        align: 2,
                        memory_index: 0,
                    }));
            } else {
                // If it's an internal function, we use LocalGet
                let base_idx = ctx.argument_local_offsets.get(idx).cloned().unwrap_or(*idx);
                ctx.wasm_func
                    .instruction(&Instruction::LocalGet(base_idx + component_idx));
            }
        }
        Expression::GlobalVariable(handle) => {
            if let Some(&(offset, base_ptr_idx)) = ctx.global_offsets.get(handle) {
                ctx.wasm_func
                    .instruction(&Instruction::GlobalGet(base_ptr_idx));
                let final_offset = offset + component_idx * 4;
                if final_offset > 0 {
                    ctx.wasm_func
                        .instruction(&Instruction::I32Const(final_offset as i32));
                    ctx.wasm_func.instruction(&Instruction::I32Add);
                }

                let ty = &ctx.module.global_variables[*handle].ty;
                let inner = &ctx.module.types[*ty].inner;
                match inner {
                    naga::TypeInner::Image { .. } | naga::TypeInner::Sampler { .. } => {
                        ctx.wasm_func
                            .instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                                offset: 0,
                                align: 2,
                                memory_index: 0,
                            }));
                    }
                    _ => {
                        ctx.wasm_func
                            .instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                                offset: 0,
                                align: 2,
                                memory_index: 0,
                            }));
                    }
                }
            } else {
                ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
            }
        }
        Expression::Load { pointer } => {
            translate_expression(*pointer, ctx)?;
            if component_idx > 0 {
                ctx.wasm_func
                    .instruction(&Instruction::I32Const((component_idx * 4) as i32));
                ctx.wasm_func.instruction(&Instruction::I32Add);
            }
            ctx.wasm_func
                .instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));
        }
        Expression::AccessIndex { base, index } => {
            let base_ty = ctx.typifier.get(*base, &ctx.module.types);
            match base_ty {
                naga::TypeInner::Pointer { .. } => {
                    translate_expression(*base, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::I32Const(
                        (*index * 4 + component_idx * 4) as i32,
                    ));
                    ctx.wasm_func.instruction(&Instruction::I32Add);
                    ctx.wasm_func
                        .instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 2,
                            memory_index: 0,
                        }));
                }
                _ => {
                    // Accessing a component of a value
                    translate_expression_component(*base, *index + component_idx, ctx)?;
                }
            }
        }
        Expression::Swizzle {
            size: _,
            vector,
            pattern,
        } => {
            let component = pattern[component_idx as usize];
            translate_expression_component(*vector, component.index(), ctx)?;
        }
        Expression::Splat { size: _, value } => {
            translate_expression_component(*value, 0, ctx)?;
        }
        Expression::As {
            expr,
            kind,
            convert,
        } => {
            // Compile the inner expression component first
            translate_expression_component(*expr, component_idx, ctx)?;

            // Determine source scalar kind
            let src_ty = ctx.typifier.get(*expr, &ctx.module.types);
            let src_scalar_kind = match src_ty {
                naga::TypeInner::Scalar(scalar) => scalar.kind,
                naga::TypeInner::Vector { scalar, .. } => scalar.kind,
                _ => naga::ScalarKind::Float,
            };

            if let Some(_width) = convert {
                // Conversion with potential change of interpretation
                match (src_scalar_kind, kind) {
                    (naga::ScalarKind::Sint, naga::ScalarKind::Float) => {
                        // i32 -> f32
                        ctx.wasm_func.instruction(&Instruction::I32ReinterpretF32);
                        ctx.wasm_func.instruction(&Instruction::F32ConvertI32S);
                    }
                    (naga::ScalarKind::Uint, naga::ScalarKind::Float) => {
                        // u32 -> f32
                        ctx.wasm_func.instruction(&Instruction::I32ReinterpretF32);
                        ctx.wasm_func.instruction(&Instruction::F32ConvertI32U);
                    }
                    (naga::ScalarKind::Float, naga::ScalarKind::Sint) => {
                        // f32 -> i32
                        ctx.wasm_func.instruction(&Instruction::I32TruncF32S);
                        ctx.wasm_func.instruction(&Instruction::F32ReinterpretI32);
                    }
                    (naga::ScalarKind::Float, naga::ScalarKind::Uint) => {
                        // f32 -> u32
                        ctx.wasm_func.instruction(&Instruction::I32TruncF32U);
                        ctx.wasm_func.instruction(&Instruction::F32ReinterpretI32);
                    }
                    (naga::ScalarKind::Bool, naga::ScalarKind::Float) => {
                        // bool -> f32 (0 or 1)
                        ctx.wasm_func.instruction(&Instruction::I32ReinterpretF32);
                        ctx.wasm_func.instruction(&Instruction::F32ConvertI32S);
                    }
                    (naga::ScalarKind::Bool, naga::ScalarKind::Sint) => {
                        // bool -> i32: already represented as i32 (0 or 1) -- no-op
                    }
                    (naga::ScalarKind::Bool, naga::ScalarKind::Uint) => {
                        // bool -> u32: no-op
                    }
                    _ => {
                        // Other conversions not handled explicitly; treat as no-op for now
                    }
                }
            } else {
                // Bitcast (no-op) since values are stored as f32 bits
            }
        }
        Expression::CallResult(_handle) => {
            if let Some(&local_idx) = ctx.call_result_locals.get(&expr_handle) {
                ctx.wasm_func
                    .instruction(&Instruction::LocalGet(local_idx + component_idx));
            } else {
                ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
            }
        }
        Expression::ImageSample {
            image, coordinate, ..
        } => {
            // Basic 2D texture sampling
            // 1. Get texture unit index (from uniform)
            translate_expression_component(*image, 0, ctx)?;
            ctx.wasm_func.instruction(&Instruction::I32TruncF32S);

            // 2. Calculate descriptor address: texture_ptr + unit_idx * 32
            ctx.wasm_func.instruction(&Instruction::I32Const(32));
            ctx.wasm_func.instruction(&Instruction::I32Mul);
            ctx.wasm_func.instruction(&Instruction::GlobalGet(4)); // texture_ptr
            ctx.wasm_func.instruction(&Instruction::I32Add);
            ctx.wasm_func
                .instruction(&Instruction::LocalSet(ctx.scratch_base)); // scratch_base is desc_addr

            // 3. Load width and height
            ctx.wasm_func
                .instruction(&Instruction::LocalGet(ctx.scratch_base));
            ctx.wasm_func
                .instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));
            ctx.wasm_func
                .instruction(&Instruction::LocalSet(ctx.scratch_base + 1)); // scratch_base + 1 is width

            ctx.wasm_func
                .instruction(&Instruction::LocalGet(ctx.scratch_base));
            ctx.wasm_func
                .instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                    offset: 4,
                    align: 2,
                    memory_index: 0,
                }));
            ctx.wasm_func
                .instruction(&Instruction::LocalSet(ctx.scratch_base + 2)); // scratch_base + 2 is height

            // 4. Load data_ptr
            ctx.wasm_func
                .instruction(&Instruction::LocalGet(ctx.scratch_base));
            ctx.wasm_func
                .instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                    offset: 8,
                    align: 2,
                    memory_index: 0,
                }));
            ctx.wasm_func
                .instruction(&Instruction::LocalSet(ctx.scratch_base + 3)); // scratch_base + 3 is data_ptr

            // 5. Calculate texel coordinates
            // u = coordinate.x, v = coordinate.y
            translate_expression_component(*coordinate, 0, ctx)?;
            // Clamp u to [0, 0.9999] to avoid out of bounds at 1.0
            ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
            ctx.wasm_func.instruction(&Instruction::F32Max);
            ctx.wasm_func.instruction(&Instruction::F32Const(0.9999));
            ctx.wasm_func.instruction(&Instruction::F32Min);
            ctx.wasm_func
                .instruction(&Instruction::LocalGet(ctx.scratch_base + 1)); // width
            ctx.wasm_func.instruction(&Instruction::F32ConvertI32S);
            ctx.wasm_func.instruction(&Instruction::F32Mul);
            ctx.wasm_func.instruction(&Instruction::I32TruncF32S);
            ctx.wasm_func
                .instruction(&Instruction::LocalSet(ctx.scratch_base + 4)); // scratch_base + 4 is texel_x

            translate_expression_component(*coordinate, 1, ctx)?;
            // Clamp v to [0, 0.9999]
            ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
            ctx.wasm_func.instruction(&Instruction::F32Max);
            ctx.wasm_func.instruction(&Instruction::F32Const(0.9999));
            ctx.wasm_func.instruction(&Instruction::F32Min);
            ctx.wasm_func
                .instruction(&Instruction::LocalGet(ctx.scratch_base + 2)); // height
            ctx.wasm_func.instruction(&Instruction::F32ConvertI32S);
            ctx.wasm_func.instruction(&Instruction::F32Mul);
            ctx.wasm_func.instruction(&Instruction::I32TruncF32S);
            ctx.wasm_func
                .instruction(&Instruction::LocalSet(ctx.scratch_base + 5)); // scratch_base + 5 is texel_y

            // 6. Calculate pixel address: data_ptr + (texel_y * width + texel_x) * 4
            ctx.wasm_func
                .instruction(&Instruction::LocalGet(ctx.scratch_base + 5)); // texel_y
            ctx.wasm_func
                .instruction(&Instruction::LocalGet(ctx.scratch_base + 1)); // width
            ctx.wasm_func.instruction(&Instruction::I32Mul);
            ctx.wasm_func
                .instruction(&Instruction::LocalGet(ctx.scratch_base + 4)); // texel_x
            ctx.wasm_func.instruction(&Instruction::I32Add);
            ctx.wasm_func.instruction(&Instruction::I32Const(4));
            ctx.wasm_func.instruction(&Instruction::I32Mul);
            ctx.wasm_func
                .instruction(&Instruction::LocalGet(ctx.scratch_base + 3)); // data_ptr
            ctx.wasm_func.instruction(&Instruction::I32Add);
            // Stack: [pixel_addr]

            // 7. Load component
            // Pixel is RGBA u8. We need to convert to f32 [0, 1].
            ctx.wasm_func
                .instruction(&Instruction::I32Load8U(wasm_encoder::MemArg {
                    offset: component_idx as u64,
                    align: 0,
                    memory_index: 0,
                }));
            ctx.wasm_func.instruction(&Instruction::F32ConvertI32U);
            ctx.wasm_func.instruction(&Instruction::F32Const(255.0));
            ctx.wasm_func.instruction(&Instruction::F32Div);
        }
        _ => {
            ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
        }
    }
    Ok(())
}

/// Translate a Naga expression to WASM instructions
pub fn translate_expression(
    expr_handle: naga::Handle<Expression>,
    ctx: &mut TranslationContext,
) -> Result<(), BackendError> {
    let expr = &ctx.func.expressions[expr_handle];
    let ty = ctx.typifier.get(expr_handle, &ctx.module.types);

    // If it's a pointer type, we want the address
    if let naga::TypeInner::Pointer { .. } = ty {
        match expr {
            Expression::LocalVariable(handle) => {
                ctx.wasm_func.instruction(&Instruction::GlobalGet(3)); // private_ptr
                if let Some(&offset) = ctx.local_offsets.get(handle) {
                    if offset > 0 {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Const(offset as i32));
                        ctx.wasm_func.instruction(&Instruction::I32Add);
                    }
                }
            }
            Expression::GlobalVariable(handle) => {
                if let Some(&(offset, base_ptr_idx)) = ctx.global_offsets.get(handle) {
                    ctx.wasm_func
                        .instruction(&Instruction::GlobalGet(base_ptr_idx));
                    if offset > 0 {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Const(offset as i32));
                        ctx.wasm_func.instruction(&Instruction::I32Add);
                    }
                } else {
                    // Fallback for unknown globals
                    let var = &ctx.module.global_variables[*handle];
                    if var.name.as_deref() == Some("gl_Position")
                        || var.name.as_deref() == Some("gl_Position_1")
                    {
                        ctx.wasm_func.instruction(&Instruction::GlobalGet(2)); // varying_ptr
                    } else {
                        ctx.wasm_func.instruction(&Instruction::GlobalGet(3)); // private_ptr
                    }
                }
            }
            Expression::AccessIndex { base, index } => {
                translate_expression(*base, ctx)?;
                // Assume each index is 4 bytes (float)
                if *index > 0 {
                    ctx.wasm_func
                        .instruction(&Instruction::I32Const((*index * 4) as i32));
                    ctx.wasm_func.instruction(&Instruction::I32Add);
                }
            }
            _ => {
                return Err(BackendError::UnsupportedFeature(format!(
                    "Unsupported pointer expression: {:?}",
                    expr
                )));
            }
        }
        return Ok(());
    }

    // Otherwise, it's a value. Loop over components.
    let count = super::types::component_count(ty);
    for i in 0..count {
        translate_expression_component(expr_handle, i, ctx)?;
    }
    Ok(())
}
