//! Expression translation from Naga IR to WASM
//!
//! Phase 0: Placeholder for future expression handling

use super::{output_layout, BackendError, TranslationContext};

use naga::{BinaryOperator, Expression, Literal, RelationalFunction, ScalarKind, TypeInner};
use wasm_encoder::Instruction;

/// Helper function to determine if a type should use I32 operations
pub fn is_integer_type(type_inner: &TypeInner) -> bool {
    match type_inner {
        TypeInner::Scalar(s) => matches!(
            s.kind,
            ScalarKind::Sint | ScalarKind::Uint | ScalarKind::Bool
        ),
        TypeInner::Vector { scalar, .. } => matches!(
            scalar.kind,
            ScalarKind::Sint | ScalarKind::Uint | ScalarKind::Bool
        ),
        _ => false,
    }
}

/// Translate a Naga constant expression component to WASM instructions
fn translate_const_expression_component(
    expr_handle: naga::Handle<Expression>,
    component_idx: u32,
    ctx: &mut TranslationContext,
) -> Result<(), BackendError> {
    let expr = &ctx.module.global_expressions[expr_handle];
    match expr {
        Expression::Literal(literal) => {
            if component_idx == 0 {
                match literal {
                    Literal::F32(f) => {
                        ctx.wasm_func.instruction(&Instruction::F32Const(*f));
                    }
                    Literal::I32(i) => {
                        ctx.wasm_func.instruction(&Instruction::I32Const(*i));
                    }
                    Literal::U32(u) => {
                        ctx.wasm_func.instruction(&Instruction::I32Const(*u as i32));
                    }
                    Literal::Bool(b) => {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Const(if *b { 1 } else { 0 }));
                    }
                    _ => {
                        return Err(BackendError::UnsupportedFeature(format!(
                            "Unsupported literal in constant: {:?}",
                            literal
                        )));
                    }
                }
            } else {
                match literal {
                    Literal::F32(_) => ctx.wasm_func.instruction(&Instruction::F32Const(0.0)),
                    _ => ctx.wasm_func.instruction(&Instruction::I32Const(0)),
                };
            }
        }
        Expression::Compose { ty, components } => {
            let mut current_component_idx = component_idx;
            let mut found = false;
            for comp_handle in components {
                let comp_expr = &ctx.module.global_expressions[*comp_handle];
                let comp_count = match comp_expr {
                    Expression::Literal(_) => 1,
                    Expression::Compose { ty, .. } => {
                        let inner = &ctx.module.types[*ty].inner;
                        super::types::component_count(inner, &ctx.module.types)
                    }
                    Expression::ZeroValue(ty) => {
                        let inner = &ctx.module.types[*ty].inner;
                        super::types::component_count(inner, &ctx.module.types)
                    }
                    _ => 1,
                };

                if current_component_idx < comp_count {
                    translate_const_expression_component(*comp_handle, current_component_idx, ctx)?;
                    found = true;
                    break;
                }
                current_component_idx -= comp_count;
            }
            if !found {
                let inner = &ctx.module.types[*ty].inner;
                if is_integer_type(inner) {
                    ctx.wasm_func.instruction(&Instruction::I32Const(0));
                } else {
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                }
            }
        }
        Expression::ZeroValue(ty) => {
            let inner = &ctx.module.types[*ty].inner;
            if is_integer_type(inner) {
                ctx.wasm_func.instruction(&Instruction::I32Const(0));
            } else {
                ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
            }
        }
        _ => {
            return Err(BackendError::UnsupportedFeature(format!(
                "Unsupported constant expression: {:?}",
                expr
            )));
        }
    }
    Ok(())
}

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
                        // No F32ReinterpretI32 - leave as I32
                    }
                    Literal::U32(u) => {
                        ctx.wasm_func.instruction(&Instruction::I32Const(*u as i32));
                        // No F32ReinterpretI32 - leave as I32
                    }
                    Literal::Bool(b) => {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Const(if *b { 1 } else { 0 }));
                        // No F32ReinterpretI32 - leave as I32
                    }
                    _ => {
                        return Err(BackendError::UnsupportedFeature(format!(
                            "Unsupported literal: {:?}",
                            literal
                        )));
                    }
                }
            } else {
                // For padding components, use appropriate type
                match literal {
                    Literal::F32(_) => ctx.wasm_func.instruction(&Instruction::F32Const(0.0)),
                    _ => ctx.wasm_func.instruction(&Instruction::I32Const(0)),
                };
            }
        }
        Expression::Constant(c_handle) => {
            let c = &ctx.module.constants[*c_handle];
            translate_const_expression_component(c.init, component_idx, ctx)?;
        }
        Expression::Compose { ty, components } => {
            let mut current_component_idx = component_idx;
            let mut found = false;
            for &comp_handle in components {
                let comp_ty = ctx.typifier.get(comp_handle, &ctx.module.types);
                let comp_count = super::types::component_count(comp_ty, &ctx.module.types);
                if current_component_idx < comp_count {
                    translate_expression_component(comp_handle, current_component_idx, ctx)?;
                    found = true;
                    break;
                }
                current_component_idx -= comp_count;
            }
            if !found {
                let inner = &ctx.module.types[*ty].inner;
                if is_integer_type(inner) {
                    ctx.wasm_func.instruction(&Instruction::I32Const(0));
                } else {
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                }
            }
        }
        Expression::ZeroValue(ty) => {
            let inner = &ctx.module.types[*ty].inner;
            if is_integer_type(inner) {
                ctx.wasm_func.instruction(&Instruction::I32Const(0));
            } else {
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

            let left_count = super::types::component_count(left_ty, &ctx.module.types);
            let right_count = super::types::component_count(right_ty, &ctx.module.types);

            let left_idx = if left_count > 1 { component_idx } else { 0 };
            let right_idx = if right_count > 1 { component_idx } else { 0 };

            // Translate operands - they're already in their natural types (I32 or F32)
            translate_expression_component(*left, left_idx, ctx)?;
            translate_expression_component(*right, right_idx, ctx)?;

            match op {
                BinaryOperator::Add => match left_scalar_kind {
                    naga::ScalarKind::Float => {
                        ctx.wasm_func.instruction(&Instruction::F32Add);
                    }
                    naga::ScalarKind::Sint | naga::ScalarKind::Uint => {
                        ctx.wasm_func.instruction(&Instruction::I32Add);
                    }
                    _ => {}
                },
                BinaryOperator::Subtract => match left_scalar_kind {
                    naga::ScalarKind::Float => {
                        ctx.wasm_func.instruction(&Instruction::F32Sub);
                    }
                    naga::ScalarKind::Sint | naga::ScalarKind::Uint => {
                        ctx.wasm_func.instruction(&Instruction::I32Sub);
                    }
                    _ => {}
                },
                BinaryOperator::Multiply => match left_scalar_kind {
                    naga::ScalarKind::Float => {
                        ctx.wasm_func.instruction(&Instruction::F32Mul);
                    }
                    naga::ScalarKind::Sint | naga::ScalarKind::Uint => {
                        ctx.wasm_func.instruction(&Instruction::I32Mul);
                    }
                    _ => {}
                },
                BinaryOperator::Divide => match left_scalar_kind {
                    naga::ScalarKind::Float => {
                        ctx.wasm_func.instruction(&Instruction::F32Div);
                    }
                    naga::ScalarKind::Sint => {
                        ctx.wasm_func.instruction(&Instruction::I32DivS);
                    }
                    naga::ScalarKind::Uint => {
                        ctx.wasm_func.instruction(&Instruction::I32DivU);
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
                    // Result is already I32 (bool), no reinterpret needed
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
                    // Result is already I32 (bool), no reinterpret needed
                }
                BinaryOperator::LogicalAnd => {
                    ctx.wasm_func.instruction(&Instruction::I32And);
                }
                BinaryOperator::LogicalOr => {
                    ctx.wasm_func.instruction(&Instruction::I32Or);
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

            let expr_ty = ctx.typifier.get(*expr, &ctx.module.types);
            let is_int = is_integer_type(expr_ty);

            match op {
                naga::UnaryOperator::Negate => {
                    if is_int {
                        // 0 - x
                        ctx.wasm_func.instruction(&Instruction::I32Const(0));
                        ctx.wasm_func.instruction(&Instruction::I32Sub);
                        // Wait, stack is [x], we need [0, x] -> sub -> -x
                        // But we already pushed x.
                        // Correct sequence:
                        // 1. Push 0
                        // 2. Push x (already done by translate_expression_component)
                        // 3. Sub
                        // Ah, we can't inject 0 before x easily here without re-translating.
                        // Better: x * -1
                        ctx.wasm_func.instruction(&Instruction::I32Const(-1));
                        ctx.wasm_func.instruction(&Instruction::I32Mul);
                    } else {
                        ctx.wasm_func.instruction(&Instruction::F32Neg);
                    }
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
                    ctx.wasm_func
                        .instruction(&Instruction::GlobalGet(output_layout::ATTR_PTR_GLOBAL));
                // attr_ptr
                } else {
                    ctx.wasm_func
                        .instruction(&Instruction::GlobalGet(output_layout::VARYING_PTR_GLOBAL));
                    // varying_ptr
                }

                // Calculate offset: sum of sizes of previous arguments
                let mut offset = 0;
                let arg = &ctx.func.arguments[*idx as usize];
                let mut found_location = false;

                if let Some(&location) = match ctx.stage {
                    naga::ShaderStage::Vertex => {
                        if let Some(name) = &arg.name {
                            ctx.attribute_locations.get(name)
                        } else {
                            None
                        }
                    }
                    naga::ShaderStage::Fragment => {
                        if let Some(name) = &arg.name {
                            ctx.varying_locations.get(name)
                        } else {
                            None
                        }
                    }
                    _ => None,
                } {
                    (offset, _) = output_layout::compute_input_offset(location, ctx.stage);
                    found_location = true;
                }

                if !found_location {
                    if let Some(naga::Binding::Location { location, .. }) = arg.binding {
                        (offset, _) = output_layout::compute_input_offset(location, ctx.stage);
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

                // Use I32Load for integer types, F32Load for float types
                let arg_ty = &ctx.module.types[arg.ty].inner;

                if is_integer_type(arg_ty) {
                    ctx.wasm_func
                        .instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 2,
                            memory_index: 0,
                        }));
                } else {
                    ctx.wasm_func
                        .instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 2,
                            memory_index: 0,
                        }));
                }
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
            } else {
                ctx.wasm_func.instruction(&Instruction::I32Const(0));
            }
        }
        Expression::Load { pointer } => {
            translate_expression(*pointer, ctx)?;
            if component_idx > 0 {
                ctx.wasm_func
                    .instruction(&Instruction::I32Const((component_idx * 4) as i32));
                ctx.wasm_func.instruction(&Instruction::I32Add);
            }

            // Determine the type being loaded from the pointer
            let pointer_ty = ctx.typifier.get(*pointer, &ctx.module.types);
            let load_ty = match pointer_ty {
                TypeInner::Pointer { base, .. } => &ctx.module.types[*base].inner,
                _ => pointer_ty,
            };

            // Use I32Load for integer types, F32Load for float types
            if is_integer_type(load_ty) {
                ctx.wasm_func
                    .instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                        offset: 0,
                        align: 2,
                        memory_index: 0,
                    }));
            } else {
                ctx.wasm_func
                    .instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                        offset: 0,
                        align: 2,
                        memory_index: 0,
                    }));
            }
        }
        Expression::AccessIndex { base, index } => {
            let base_ty = ctx.typifier.get(*base, &ctx.module.types);
            match base_ty {
                naga::TypeInner::Pointer {
                    base: pointed_ty, ..
                } => {
                    translate_expression(*base, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::I32Const(
                        (*index * 4 + component_idx * 4) as i32,
                    ));
                    ctx.wasm_func.instruction(&Instruction::I32Add);

                    // Determine the type of the element being accessed
                    let element_ty = &ctx.module.types[*pointed_ty].inner;

                    // Use I32Load for integer types, F32Load for float types
                    if is_integer_type(element_ty) {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                                offset: 0,
                                align: 2,
                                memory_index: 0,
                            }));
                    } else {
                        ctx.wasm_func
                            .instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                                offset: 0,
                                align: 2,
                                memory_index: 0,
                            }));
                    }
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
                        // i32 -> f32: value is already I32 on stack
                        ctx.wasm_func.instruction(&Instruction::F32ConvertI32S);
                    }
                    (naga::ScalarKind::Uint, naga::ScalarKind::Float) => {
                        // u32 -> f32: value is already I32 on stack
                        ctx.wasm_func.instruction(&Instruction::F32ConvertI32U);
                    }
                    (naga::ScalarKind::Float, naga::ScalarKind::Sint) => {
                        // f32 -> i32: value is already F32 on stack
                        ctx.wasm_func.instruction(&Instruction::I32TruncSatF32S);
                    }
                    (naga::ScalarKind::Float, naga::ScalarKind::Uint) => {
                        // f32 -> u32: value is already F32 on stack
                        ctx.wasm_func.instruction(&Instruction::I32TruncSatF32U);
                    }
                    (naga::ScalarKind::Bool, naga::ScalarKind::Float) => {
                        // bool -> f32 (0 or 1): value is already I32 on stack
                        ctx.wasm_func.instruction(&Instruction::F32ConvertI32S);
                    }
                    (naga::ScalarKind::Bool, naga::ScalarKind::Sint) => {
                        // bool -> i32: already represented as i32 (0 or 1) -- no-op
                    }
                    (naga::ScalarKind::Bool, naga::ScalarKind::Uint) => {
                        // bool -> u32: no-op
                    }
                    (naga::ScalarKind::Sint, naga::ScalarKind::Uint)
                    | (naga::ScalarKind::Uint, naga::ScalarKind::Sint) => {
                        // i32 <-> u32: no-op (same bit representation)
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

            // If the image expression is a GlobalVariable, we have its address on the stack.
            // We need to load the value (texture unit index, which is i32).
            if let Expression::GlobalVariable(_) = ctx.func.expressions[*image] {
                ctx.wasm_func
                    .instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                        offset: 0,
                        align: 2,
                        memory_index: 0,
                    }));
            }

            // 2. Calculate descriptor address: texture_ptr + unit_idx * 32
            ctx.wasm_func.instruction(&Instruction::I32Const(32));
            ctx.wasm_func.instruction(&Instruction::I32Mul);
            ctx.wasm_func
                .instruction(&Instruction::GlobalGet(output_layout::TEXTURE_PTR_GLOBAL)); // texture_ptr
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
            // Scale by width: u * width
            ctx.wasm_func
                .instruction(&Instruction::LocalGet(ctx.scratch_base + 1)); // width
            ctx.wasm_func.instruction(&Instruction::F32ConvertI32S);
            ctx.wasm_func.instruction(&Instruction::F32Mul);
            // Floor
            ctx.wasm_func.instruction(&Instruction::F32Floor);
            // Clamp to [0.0, width - 1.0]
            ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
            ctx.wasm_func.instruction(&Instruction::F32Max);

            ctx.wasm_func
                .instruction(&Instruction::LocalGet(ctx.scratch_base + 1)); // width
            ctx.wasm_func.instruction(&Instruction::F32ConvertI32S);
            ctx.wasm_func.instruction(&Instruction::F32Const(1.0));
            ctx.wasm_func.instruction(&Instruction::F32Sub);
            ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
            ctx.wasm_func.instruction(&Instruction::F32Max);
            ctx.wasm_func.instruction(&Instruction::F32Min);

            ctx.wasm_func.instruction(&Instruction::I32TruncSatF32S);
            ctx.wasm_func
                .instruction(&Instruction::LocalSet(ctx.scratch_base + 4)); // scratch_base + 4 is texel_x

            translate_expression_component(*coordinate, 1, ctx)?;
            // Scale by height: v * height
            ctx.wasm_func
                .instruction(&Instruction::LocalGet(ctx.scratch_base + 2)); // height
            ctx.wasm_func.instruction(&Instruction::F32ConvertI32S);
            ctx.wasm_func.instruction(&Instruction::F32Mul);
            // Floor
            ctx.wasm_func.instruction(&Instruction::F32Floor);
            // Clamp to [0.0, height - 1.0]
            ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
            ctx.wasm_func.instruction(&Instruction::F32Max);

            ctx.wasm_func
                .instruction(&Instruction::LocalGet(ctx.scratch_base + 2)); // height
            ctx.wasm_func.instruction(&Instruction::F32ConvertI32S);
            ctx.wasm_func.instruction(&Instruction::F32Const(1.0));
            ctx.wasm_func.instruction(&Instruction::F32Sub);
            ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
            ctx.wasm_func.instruction(&Instruction::F32Max);
            ctx.wasm_func.instruction(&Instruction::F32Min);

            ctx.wasm_func.instruction(&Instruction::I32TruncSatF32S);
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
        Expression::Relational { fun, argument } => match fun {
            RelationalFunction::All => {
                if component_idx == 0 {
                    translate_expression(*argument, ctx)?;
                    let arg_ty = ctx.typifier.get(*argument, &ctx.module.types);
                    let count = super::types::component_count(arg_ty, &ctx.module.types);
                    for _ in 1..count {
                        ctx.wasm_func.instruction(&Instruction::I32And);
                    }
                } else {
                    ctx.wasm_func.instruction(&Instruction::I32Const(0));
                }
            }
            RelationalFunction::Any => {
                if component_idx == 0 {
                    translate_expression(*argument, ctx)?;
                    let arg_ty = ctx.typifier.get(*argument, &ctx.module.types);
                    let count = super::types::component_count(arg_ty, &ctx.module.types);
                    for _ in 1..count {
                        ctx.wasm_func.instruction(&Instruction::I32Or);
                    }
                } else {
                    ctx.wasm_func.instruction(&Instruction::I32Const(0));
                }
            }
            _ => {
                ctx.wasm_func.instruction(&Instruction::I32Const(0));
            }
        },
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
                ctx.wasm_func
                    .instruction(&Instruction::GlobalGet(output_layout::PRIVATE_PTR_GLOBAL)); // private_ptr
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
                        ctx.wasm_func.instruction(&Instruction::GlobalGet(
                            output_layout::VARYING_PTR_GLOBAL,
                        )); // varying_ptr
                    } else {
                        ctx.wasm_func.instruction(&Instruction::GlobalGet(
                            output_layout::PRIVATE_PTR_GLOBAL,
                        )); // private_ptr
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
    let count = super::types::component_count(ty, &ctx.module.types);
    for i in 0..count {
        translate_expression_component(expr_handle, i, ctx)?;
    }
    Ok(())
}
