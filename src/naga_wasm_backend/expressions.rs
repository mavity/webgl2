//! Expression translation from Naga IR to WASM
//!
//! Phase 0: Placeholder for future expression handling

use super::{output_layout, BackendError, TranslationContext};

use naga::{
    BinaryOperator, Expression, Literal, MathFunction, RelationalFunction, ScalarKind, TypeInner,
};
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
                naga::TypeInner::Pointer { base, .. } => match &ctx.module.types[*base].inner {
                    naga::TypeInner::Scalar(scalar) => scalar.kind,
                    naga::TypeInner::Vector { scalar, .. } => scalar.kind,
                    _ => naga::ScalarKind::Float,
                },
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

            // Vector-Matrix multiplication (v * M)
            if let (
                naga::TypeInner::Vector { size, .. },
                naga::TypeInner::Matrix {
                    columns: _, rows, ..
                },
            ) = (left_ty, right_ty)
            {
                if *op == BinaryOperator::Multiply {
                    // result[component_idx] = sum_i(vector[i] * matrix[component_idx][i])
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    for i in 0..(*size as u32) {
                        translate_expression_component(*left, i, ctx)?;
                        translate_expression_component(
                            *right,
                            component_idx * (*rows as u32) + i,
                            ctx,
                        )?;
                        ctx.wasm_func.instruction(&Instruction::F32Mul);
                        ctx.wasm_func.instruction(&Instruction::F32Add);
                    }
                    return Ok(());
                }
            }

            // Matrix-Matrix multiplication (M * M)
            if let (
                naga::TypeInner::Matrix {
                    columns: l_cols,
                    rows: l_rows,
                    ..
                },
                naga::TypeInner::Matrix {
                    columns: _,
                    rows: r_rows,
                    ..
                },
            ) = (left_ty, right_ty)
            {
                if *op == BinaryOperator::Multiply {
                    // result[col][row] = sum_j(left[j][row] * right[col][j])
                    let row = component_idx % (*l_rows as u32);
                    let col = component_idx / (*l_rows as u32);

                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    for j in 0..(*l_cols as u32) {
                        // left[j][row]
                        translate_expression_component(*left, j * (*l_rows as u32) + row, ctx)?;
                        // right[col][j]
                        translate_expression_component(*right, col * (*r_rows as u32) + j, ctx)?;
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
                        // If the right operand is integer, convert it to f32 first
                        if let naga::TypeInner::Scalar(s) = right_ty {
                            if s.kind == naga::ScalarKind::Sint {
                                ctx.wasm_func.instruction(&Instruction::F32ConvertI32S);
                            } else if s.kind == naga::ScalarKind::Uint {
                                ctx.wasm_func.instruction(&Instruction::F32ConvertI32U);
                            }
                        }
                        ctx.wasm_func.instruction(&Instruction::F32Add);
                    }
                    naga::ScalarKind::Sint | naga::ScalarKind::Uint => {
                        ctx.wasm_func.instruction(&Instruction::I32Add);
                    }
                    _ => {}
                },
                BinaryOperator::Subtract => match left_scalar_kind {
                    naga::ScalarKind::Float => {
                        // Convert right operand if it's an integer
                        if let naga::TypeInner::Scalar(s) = right_ty {
                            if s.kind == naga::ScalarKind::Sint {
                                ctx.wasm_func.instruction(&Instruction::F32ConvertI32S);
                            } else if s.kind == naga::ScalarKind::Uint {
                                ctx.wasm_func.instruction(&Instruction::F32ConvertI32U);
                            }
                        }
                        ctx.wasm_func.instruction(&Instruction::F32Sub);
                    }
                    naga::ScalarKind::Sint | naga::ScalarKind::Uint => {
                        ctx.wasm_func.instruction(&Instruction::I32Sub);
                    }
                    _ => {}
                },
                BinaryOperator::Multiply => match left_scalar_kind {
                    naga::ScalarKind::Float => {
                        // Convert right operand if needed
                        if let naga::TypeInner::Scalar(s) = right_ty {
                            if s.kind == naga::ScalarKind::Sint {
                                ctx.wasm_func.instruction(&Instruction::F32ConvertI32S);
                            } else if s.kind == naga::ScalarKind::Uint {
                                ctx.wasm_func.instruction(&Instruction::F32ConvertI32U);
                            }
                        }
                        ctx.wasm_func.instruction(&Instruction::F32Mul);
                    }
                    naga::ScalarKind::Sint | naga::ScalarKind::Uint => {
                        ctx.wasm_func.instruction(&Instruction::I32Mul);
                    }
                    _ => {}
                },
                BinaryOperator::Divide => match left_scalar_kind {
                    naga::ScalarKind::Float => {
                        // Convert right operand if needed
                        if let naga::TypeInner::Scalar(s) = right_ty {
                            if s.kind == naga::ScalarKind::Sint {
                                ctx.wasm_func.instruction(&Instruction::F32ConvertI32S);
                            } else if s.kind == naga::ScalarKind::Uint {
                                ctx.wasm_func.instruction(&Instruction::F32ConvertI32U);
                            }
                        }
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
                // Remainder (modulo) operation
                BinaryOperator::Modulo => match left_scalar_kind {
                    naga::ScalarKind::Float => {
                        // Float modulo: a - b * trunc(a/b)
                        // Stack at start: [a, b]
                        // We need: a - b * trunc(a/b)

                        // Use both swap locals for this operation
                        let temp_a = ctx.swap_f32_local;
                        let temp_b = ctx
                            .swap_f32_local_2
                            .expect("Float modulo swap local missing");

                        // Save b to temp_b
                        ctx.wasm_func.instruction(&Instruction::LocalSet(temp_b));
                        // Duplicate a using LocalTee
                        ctx.wasm_func.instruction(&Instruction::LocalTee(temp_a));
                        // Load b for division
                        ctx.wasm_func.instruction(&Instruction::LocalGet(temp_b));
                        // a / b
                        ctx.wasm_func.instruction(&Instruction::F32Div);
                        // trunc(a/b)
                        ctx.wasm_func.instruction(&Instruction::F32Trunc);
                        // Load b again
                        ctx.wasm_func.instruction(&Instruction::LocalGet(temp_b));
                        // trunc(a/b) * b
                        ctx.wasm_func.instruction(&Instruction::F32Mul);
                        // Now we need a - (trunc(a/b) * b)
                        // Stack: [trunc(a/b) * b]
                        // We need to negate and add: a + (-(trunc(a/b) * b))
                        ctx.wasm_func.instruction(&Instruction::F32Neg);
                        // Load a
                        ctx.wasm_func.instruction(&Instruction::LocalGet(temp_a));
                        // a + (-trunc(a/b) * b) = a - trunc(a/b) * b
                        ctx.wasm_func.instruction(&Instruction::F32Add);
                    }
                    naga::ScalarKind::Sint => {
                        ctx.wasm_func.instruction(&Instruction::I32RemS);
                    }
                    naga::ScalarKind::Uint => {
                        ctx.wasm_func.instruction(&Instruction::I32RemU);
                    }
                    _ => {}
                },
                // Bitwise operations (integer only)
                BinaryOperator::And => {
                    // For integers, this is bitwise AND
                    ctx.wasm_func.instruction(&Instruction::I32And);
                }
                BinaryOperator::ExclusiveOr => {
                    ctx.wasm_func.instruction(&Instruction::I32Xor);
                }
                BinaryOperator::InclusiveOr => {
                    ctx.wasm_func.instruction(&Instruction::I32Or);
                }
                // Shift operations
                BinaryOperator::ShiftLeft => {
                    ctx.wasm_func.instruction(&Instruction::I32Shl);
                }
                BinaryOperator::ShiftRight => match left_scalar_kind {
                    naga::ScalarKind::Sint => {
                        ctx.wasm_func.instruction(&Instruction::I32ShrS);
                    }
                    naga::ScalarKind::Uint => {
                        ctx.wasm_func.instruction(&Instruction::I32ShrU);
                    }
                    _ => {
                        // Default to unsigned for other cases
                        ctx.wasm_func.instruction(&Instruction::I32ShrU);
                    }
                },
                BinaryOperator::Less => match left_scalar_kind {
                    naga::ScalarKind::Float | naga::ScalarKind::AbstractFloat => {
                        ctx.wasm_func.instruction(&Instruction::F32Lt);
                    }
                    naga::ScalarKind::Sint | naga::ScalarKind::AbstractInt => {
                        ctx.wasm_func.instruction(&Instruction::I32LtS);
                    }
                    naga::ScalarKind::Uint | naga::ScalarKind::Bool => {
                        ctx.wasm_func.instruction(&Instruction::I32LtU);
                    }
                },
                BinaryOperator::LessEqual => match left_scalar_kind {
                    naga::ScalarKind::Float | naga::ScalarKind::AbstractFloat => {
                        ctx.wasm_func.instruction(&Instruction::F32Le);
                    }
                    naga::ScalarKind::Sint | naga::ScalarKind::AbstractInt => {
                        ctx.wasm_func.instruction(&Instruction::I32LeS);
                    }
                    naga::ScalarKind::Uint | naga::ScalarKind::Bool => {
                        ctx.wasm_func.instruction(&Instruction::I32LeU);
                    }
                },
                BinaryOperator::Greater => match left_scalar_kind {
                    naga::ScalarKind::Float | naga::ScalarKind::AbstractFloat => {
                        ctx.wasm_func.instruction(&Instruction::F32Gt);
                    }
                    naga::ScalarKind::Sint | naga::ScalarKind::AbstractInt => {
                        ctx.wasm_func.instruction(&Instruction::I32GtS);
                    }
                    naga::ScalarKind::Uint | naga::ScalarKind::Bool => {
                        ctx.wasm_func.instruction(&Instruction::I32GtU);
                    }
                },
                BinaryOperator::GreaterEqual => match left_scalar_kind {
                    naga::ScalarKind::Float | naga::ScalarKind::AbstractFloat => {
                        ctx.wasm_func.instruction(&Instruction::F32Ge);
                    }
                    naga::ScalarKind::Sint | naga::ScalarKind::AbstractInt => {
                        ctx.wasm_func.instruction(&Instruction::I32GeS);
                    }
                    naga::ScalarKind::Uint | naga::ScalarKind::Bool => {
                        ctx.wasm_func.instruction(&Instruction::I32GeU);
                    }
                },
            }
        }
        Expression::Unary { op, expr } => {
            translate_expression_component(*expr, component_idx, ctx)?;

            let expr_ty = ctx.typifier.get(*expr, &ctx.module.types);
            let is_int = is_integer_type(expr_ty);

            match op {
                naga::UnaryOperator::Negate => {
                    if is_int {
                        // x * -1
                        ctx.wasm_func.instruction(&Instruction::I32Const(-1));
                        ctx.wasm_func.instruction(&Instruction::I32Mul);
                    } else {
                        ctx.wasm_func.instruction(&Instruction::F32Neg);
                    }
                }
                naga::UnaryOperator::LogicalNot => {
                    ctx.wasm_func.instruction(&Instruction::I32Eqz);
                }
                naga::UnaryOperator::BitwiseNot => {
                    ctx.wasm_func.instruction(&Instruction::I32Const(-1));
                    ctx.wasm_func.instruction(&Instruction::I32Xor);
                }
            }
        }
        Expression::FunctionArgument(idx) => {
            if ctx.is_entry_point {
                let arg = &ctx.func.arguments[*idx as usize];

                // Tier 2: Built-ins that are passed as direct arguments
                if let Some(naga::Binding::BuiltIn(bi)) = arg.binding {
                    match (bi, ctx.stage) {
                        (naga::BuiltIn::VertexIndex, naga::ShaderStage::Vertex) => {
                            // vertex_id is argument 0
                            ctx.wasm_func.instruction(&Instruction::LocalGet(0));
                            return Ok(());
                        }
                        (naga::BuiltIn::InstanceIndex, naga::ShaderStage::Vertex) => {
                            // instance_id is argument 1
                            ctx.wasm_func.instruction(&Instruction::LocalGet(1));
                            return Ok(());
                        }
                        _ => {}
                    }
                }

                // For entry points, regular arguments are loaded from memory
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

                // Allow program-level type maps to override IR-inferred type
                let mut override_is_int = false;
                if ctx.stage == naga::ShaderStage::Vertex {
                    if let Some(name) = &arg.name {
                        if let Some((type_code, _)) = ctx.attribute_types.get(name) {
                            override_is_int = (*type_code == 1) || (*type_code == 2);
                        }
                    } else if let Some(naga::Binding::Location { location, .. }) = arg.binding {
                        for (aname, &loc) in ctx.attribute_locations.iter() {
                            if loc == location {
                                if let Some((type_code, _)) = ctx.attribute_types.get(aname) {
                                    override_is_int = (*type_code == 1) || (*type_code == 2);
                                }
                                break;
                            }
                        }
                    }
                } else if let Some(name) = &arg.name {
                    if let Some((type_code, _)) = ctx.varying_types.get(name) {
                        override_is_int = (*type_code == 1) || (*type_code == 2);
                    }
                } else if let Some(naga::Binding::Location { location, .. }) = arg.binding {
                    for (vname, &loc) in ctx.varying_locations.iter() {
                        if loc == location {
                            if let Some((type_code, _)) = ctx.varying_types.get(vname) {
                                override_is_int = (*type_code == 1) || (*type_code == 2);
                            }
                            break;
                        }
                    }
                }

                if override_is_int || is_integer_type(arg_ty) {
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

                // For Uniforms and Textures, the offset points into the Context Block.
                // We must load the actual base address from there.
                if base_ptr_idx == output_layout::UNIFORM_PTR_GLOBAL
                    || base_ptr_idx == output_layout::TEXTURE_PTR_GLOBAL
                {
                    ctx.wasm_func
                        .instruction(&Instruction::I32Const(offset as i32));
                    ctx.wasm_func.instruction(&Instruction::I32Add);
                    ctx.wasm_func
                        .instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 2,
                            memory_index: 0,
                        }));
                }

                let final_offset = component_idx * 4;
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

            // Allow program-level type info (uniforms/varyings) to override
            // Find underlying GlobalVariable (if any) by walking loads/access chains
            let mut override_is_int = false;
            let mut cur_expr = *pointer;
            let mut found_global = None;
            loop {
                match ctx.func.expressions[cur_expr] {
                    naga::Expression::GlobalVariable(handle) => {
                        found_global = Some(handle);
                        break;
                    }
                    naga::Expression::Load { pointer: p } => {
                        cur_expr = p;
                    }
                    naga::Expression::AccessIndex { base: b, .. } => {
                        cur_expr = b;
                    }
                    naga::Expression::LocalVariable(lh) => {
                        if let Some(&gh) = ctx.local_origins.get(&lh) {
                            found_global = Some(gh);
                        }
                        break;
                    }
                    _ => {
                        break;
                    }
                }
            }
            if let Some(handle) = found_global {
                if let Some(name) = &ctx.module.global_variables[handle].name {
                    match ctx.module.global_variables[handle].space {
                        naga::AddressSpace::Uniform | naga::AddressSpace::Handle => {
                            if let Some((type_code, _)) = ctx.uniform_types.get(name) {
                                override_is_int = (*type_code == 1) || (*type_code == 2);
                            }
                        }
                        _ => {
                            if let Some((type_code, _)) = ctx.varying_types.get(name) {
                                override_is_int = (*type_code == 1) || (*type_code == 2);
                            }
                        }
                    }
                }
            } else {
                // No underlying global found; leave override_is_int as detected from pointer type
            }

            if override_is_int || is_integer_type(load_ty) {
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
                    // Compute byte offset using element size (don't assume 4 bytes per element).
                    // For arrays, use the element stride rather than total array size.
                    let element_inner = &ctx.module.types[*pointed_ty].inner;
                    let element_size = match element_inner {
                        naga::TypeInner::Array { stride, .. } => *stride,
                        _ => super::types::type_size(element_inner).unwrap_or(4),
                    };
                    let offset = (*index * element_size) + (component_idx * 4);
                    if offset > 0 {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Const(offset as i32));
                        ctx.wasm_func.instruction(&Instruction::I32Add);
                    }

                    // Determine the type of the element being accessed
                    let element_ty = &ctx.module.types[*pointed_ty].inner;

                    // Allow program-level type info to override element type
                    // Allow program-level type info to override element type
                    // Find underlying GlobalVariable (if any) by walking loads/access chains
                    let mut override_is_int = false;
                    let mut cur_expr = *base;
                    let mut found_global = None;
                    loop {
                        match ctx.func.expressions[cur_expr] {
                            naga::Expression::GlobalVariable(handle) => {
                                found_global = Some(handle);
                                break;
                            }
                            naga::Expression::Load { pointer: p } => {
                                cur_expr = p;
                            }
                            naga::Expression::AccessIndex { base: b, .. } => {
                                cur_expr = b;
                            }
                            naga::Expression::LocalVariable(lh) => {
                                if let Some(&gh) = ctx.local_origins.get(&lh) {
                                    found_global = Some(gh);
                                }
                                break;
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    if let Some(handle) = found_global {
                        if let Some(name) = &ctx.module.global_variables[handle].name {
                            match ctx.module.global_variables[handle].space {
                                naga::AddressSpace::Uniform | naga::AddressSpace::Handle => {
                                    if let Some((type_code, _)) = ctx.uniform_types.get(name) {
                                        override_is_int = (*type_code == 1) || (*type_code == 2);
                                    }
                                }
                                _ => {
                                    if let Some((type_code, _)) = ctx.varying_types.get(name) {
                                        override_is_int = (*type_code == 1) || (*type_code == 2);
                                    }
                                }
                            }
                        }
                    }

                    if override_is_int || is_integer_type(element_ty) {
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
        Expression::Access { base, index } => {
            let base_ty = ctx.typifier.get(*base, &ctx.module.types);
            match base_ty {
                naga::TypeInner::Pointer {
                    base: pointed_ty, ..
                } => {
                    // Base is a pointer, calculate address
                    translate_expression(*base, ctx)?;

                    // Index is dynamic
                    translate_expression_component(*index, 0, ctx)?;

                    // Element size
                    let element_inner = &ctx.module.types[*pointed_ty].inner;
                    let element_size = match element_inner {
                        naga::TypeInner::Array { stride, .. } => *stride,
                        _ => super::types::type_size(element_inner).unwrap_or(4),
                    };

                    ctx.wasm_func
                        .instruction(&Instruction::I32Const(element_size as i32));
                    ctx.wasm_func.instruction(&Instruction::I32Mul);
                    ctx.wasm_func.instruction(&Instruction::I32Add);

                    // Add component offset
                    if component_idx > 0 {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Const((component_idx * 4) as i32));
                        ctx.wasm_func.instruction(&Instruction::I32Add);
                    }

                    // Determine the type of the element being accessed
                    let element_ty = &ctx.module.types[*pointed_ty].inner;

                    // Allow program-level type info to override element type
                    let mut override_is_int = false;
                    let mut cur_expr = *base;
                    let mut found_global = None;
                    loop {
                        match ctx.func.expressions[cur_expr] {
                            naga::Expression::GlobalVariable(handle) => {
                                found_global = Some(handle);
                                break;
                            }
                            naga::Expression::Load { pointer: p } => {
                                cur_expr = p;
                            }
                            naga::Expression::AccessIndex { base: b, .. } => {
                                cur_expr = b;
                            }
                            naga::Expression::Access { base: b, .. } => {
                                cur_expr = b;
                            }
                            naga::Expression::LocalVariable(lh) => {
                                if let Some(&gh) = ctx.local_origins.get(&lh) {
                                    found_global = Some(gh);
                                }
                                break;
                            }
                            _ => {
                                break;
                            }
                        }
                    }

                    if let Some(handle) = found_global {
                        if let Some(name) = &ctx.module.global_variables[handle].name {
                            match ctx.module.global_variables[handle].space {
                                naga::AddressSpace::Uniform | naga::AddressSpace::Handle => {
                                    if let Some((type_code, _)) = ctx.uniform_types.get(name) {
                                        override_is_int = (*type_code == 1) || (*type_code == 2);
                                    }
                                }
                                _ => {
                                    if let Some((type_code, _)) = ctx.varying_types.get(name) {
                                        override_is_int = (*type_code == 1) || (*type_code == 2);
                                    }
                                }
                            }
                        }
                    }

                    if override_is_int || is_integer_type(element_ty) {
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
                    // Accessing a component of a value (e.g. vector[i])
                    // This is hard to do without memory. For now, we only support constant indexing here.
                    // But wait, we can use a sequence of selects.
                    let base_count = super::types::component_count(base_ty, &ctx.module.types);
                    if base_count == 1 {
                        translate_expression_component(*base, 0, ctx)?;
                    } else if base_count <= 4 {
                        // For small vectors, we can unroll selects
                        // result = (index == 0) ? a.x : ((index == 1) ? a.y : ((index == 2) ? a.z : a.w))

                        // Push components in reverse order for nested selects
                        for i in (0..base_count).rev() {
                            translate_expression_component(*base, i, ctx)?;
                        }

                        // Now we have [base_count elements] on stack
                        // We need to pick one. This is annoying with just select.

                        // Alternate approach: use a local array.
                        // But we don't have a local array per expression.

                        // Let's use the swap locals.
                        let temp_idx = ctx.swap_i32_local;
                        translate_expression_component(*index, 0, ctx)?;
                        ctx.wasm_func.instruction(&Instruction::LocalSet(temp_idx));

                        // Current result (default to last component)
                        translate_expression_component(*base, base_count - 1, ctx)?;

                        for i in (0..base_count - 1).rev() {
                            // stack: [prev_result]
                            translate_expression_component(*base, i, ctx)?;
                            // stack: [prev_result, component_i]
                            ctx.wasm_func.instruction(&Instruction::LocalGet(temp_idx));
                            ctx.wasm_func.instruction(&Instruction::I32Const(i as i32));
                            ctx.wasm_func.instruction(&Instruction::I32Eq);
                            // stack: [prev_result, component_i, is_i]
                            ctx.wasm_func.instruction(&Instruction::Select);
                        }
                    } else {
                        // Larger values not supported for dynamic access yet
                        ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    }
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
                // Bitcast between different WASM types
                match (src_scalar_kind, kind) {
                    (naga::ScalarKind::Float, naga::ScalarKind::Uint)
                    | (naga::ScalarKind::Float, naga::ScalarKind::Sint) => {
                        ctx.wasm_func.instruction(&Instruction::I32ReinterpretF32);
                    }
                    (naga::ScalarKind::Uint, naga::ScalarKind::Float)
                    | (naga::ScalarKind::Sint, naga::ScalarKind::Float) => {
                        ctx.wasm_func.instruction(&Instruction::F32ReinterpretI32);
                    }
                    _ => {
                        // i32 <-> u32 or bool <-> i32: no-op in WASM
                    }
                }
            }
        }
        Expression::CallResult(_handle) => {
            if let Some(&runtime_base) = ctx.call_result_locals.get(&expr_handle) {
                // call_result_locals now stores runtime indices directly
                let target = runtime_base + component_idx;

                // Load the local value
                ctx.wasm_func.instruction(&Instruction::LocalGet(target));

                // No conversion needed: target indexes into an F32 local
            } else {
                ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
            }
        }
        Expression::ImageSample {
            image,
            coordinate,
            sampler,
            ..
        } => {
            let ty_handle = ctx.typifier[*image].handle().unwrap();
            let dim = match ctx.module.types[ty_handle].inner {
                naga::TypeInner::Image { dim, .. } => dim,
                _ => naga::ImageDimension::D2,
            };

            let sampler_idx = if dim == naga::ImageDimension::D3 {
                ctx.webgl_sampler_3d_idx
            } else {
                ctx.webgl_sampler_2d_idx
            };

            if let Some(tex_fetch_idx) = sampler_idx {
                let push_handle_addr =
                    |h_expr: naga::Handle<Expression>, ctx: &mut TranslationContext| {
                        if let Expression::GlobalVariable(h) = ctx.func.expressions[h_expr] {
                            if let Some(&(offset, base_ptr_idx)) = ctx.global_offsets.get(&h) {
                                let var = &ctx.module.global_variables[h];
                                if base_ptr_idx == output_layout::UNIFORM_PTR_GLOBAL
                                    && var.space == naga::AddressSpace::Handle
                                    && !ctx.uniform_locations.is_empty()
                                {
                                    // Index model (WebGL): Load unit index from uniform memory
                                    // Note: WebGL uniforms use indirection (table -> value)
                                    ctx.wasm_func.instruction(&Instruction::GlobalGet(
                                        output_layout::UNIFORM_PTR_GLOBAL,
                                    ));
                                    if offset > 0 {
                                        ctx.wasm_func
                                            .instruction(&Instruction::I32Const(offset as i32));
                                        ctx.wasm_func.instruction(&Instruction::I32Add);
                                    }
                                    ctx.wasm_func.instruction(&Instruction::I32Load(
                                        wasm_encoder::MemArg {
                                            offset: 0,
                                            align: 2,
                                            memory_index: 0,
                                        },
                                    ));

                                    // Now we have unit_index on stack
                                    ctx.wasm_func.instruction(&Instruction::I32Const(6));
                                    ctx.wasm_func.instruction(&Instruction::I32Shl); // * 64
                                    ctx.wasm_func.instruction(&Instruction::GlobalGet(
                                        output_layout::TEXTURE_PTR_GLOBAL,
                                    ));
                                    ctx.wasm_func.instruction(&Instruction::I32Add);
                                } else {
                                    // Handle model (WebGPU) or direct pointer: Push absolute descriptor address
                                    translate_expression_component(h_expr, 0, ctx)?;
                                }
                            } else {
                                translate_expression_component(h_expr, 0, ctx)?;
                            }
                        } else {
                            translate_expression_component(h_expr, 0, ctx)?;
                        }
                        Ok::<(), BackendError>(())
                    };

                // 1. Push texture descriptor address
                push_handle_addr(*image, ctx)?;

                // 2. Push sampler descriptor address
                push_handle_addr(*sampler, ctx)?;

                // 3. Push coordinates
                translate_expression_component(*coordinate, 0, ctx)?;
                translate_expression_component(*coordinate, 1, ctx)?;
                if dim == naga::ImageDimension::D3 {
                    translate_expression_component(*coordinate, 2, ctx)?;
                }

                // 4. Call helper (expects texture_desc, sampler_desc, u, v, [w])
                ctx.wasm_func.instruction(&Instruction::Call(tex_fetch_idx));

                // 5. Store results
                let sample_base = ctx
                    .sample_f32_locals
                    .expect("Sampling locals not allocated for function with ImageSample?");

                ctx.wasm_func
                    .instruction(&Instruction::LocalSet(sample_base + 3)); // a
                ctx.wasm_func
                    .instruction(&Instruction::LocalSet(sample_base + 2)); // b
                ctx.wasm_func
                    .instruction(&Instruction::LocalSet(sample_base + 1)); // g
                ctx.wasm_func
                    .instruction(&Instruction::LocalSet(sample_base)); // r

                // 6. Return requested component
                ctx.wasm_func
                    .instruction(&Instruction::LocalGet(sample_base + component_idx));

                // Reinterpret if it's an integer sampler
                let ty_handle = ctx.typifier[*image].handle().unwrap();
                let is_integer = match ctx.module.types[ty_handle].inner {
                    naga::TypeInner::Image {
                        class: naga::ImageClass::Sampled { kind, .. },
                        ..
                    } => kind != naga::ScalarKind::Float,
                    _ => false,
                };
                if is_integer {
                    ctx.wasm_func.instruction(&Instruction::I32ReinterpretF32);
                }
            } else {
                ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
            }
        }
        Expression::ImageLoad {
            image, coordinate, ..
        } => {
            if let Some(load_idx) = ctx.webgl_image_load_idx {
                // 1. Resolve arguments 0 and 1 (texture_ptr and desc_addr)
                if let Expression::GlobalVariable(h) = ctx.func.expressions[*image] {
                    if let Some(&(offset, base_ptr_idx)) = ctx.global_offsets.get(&h) {
                        ctx.wasm_func.instruction(&Instruction::GlobalGet(
                            output_layout::TEXTURE_PTR_GLOBAL,
                        ));
                        if base_ptr_idx == output_layout::UNIFORM_PTR_GLOBAL
                            && !ctx.uniform_locations.is_empty()
                        {
                            // Index model (WebGL): Load unit index from uniform memory
                            ctx.wasm_func.instruction(&Instruction::GlobalGet(
                                output_layout::UNIFORM_PTR_GLOBAL,
                            ));
                            let final_offset = offset;
                            if final_offset > 0 {
                                ctx.wasm_func
                                    .instruction(&Instruction::I32Const(final_offset as i32));
                                ctx.wasm_func.instruction(&Instruction::I32Add);
                            }
                            ctx.wasm_func.instruction(&Instruction::I32Load(
                                wasm_encoder::MemArg {
                                    offset: 0,
                                    align: 2,
                                    memory_index: 0,
                                },
                            ));
                            ctx.wasm_func.instruction(&Instruction::I32Const(64));
                            ctx.wasm_func.instruction(&Instruction::I32Mul);
                            ctx.wasm_func.instruction(&Instruction::GlobalGet(
                                output_layout::TEXTURE_PTR_GLOBAL,
                            ));
                            ctx.wasm_func.instruction(&Instruction::I32Add);
                        } else {
                            // Handle model (WebGPU)
                            translate_expression_component(*image, 0, ctx)?;
                        }
                    } else {
                        // Fallback
                        ctx.wasm_func.instruction(&Instruction::GlobalGet(
                            output_layout::TEXTURE_PTR_GLOBAL,
                        ));
                        translate_expression_component(*image, 0, ctx)?;
                    }
                } else {
                    // Fallback
                    ctx.wasm_func
                        .instruction(&Instruction::GlobalGet(output_layout::TEXTURE_PTR_GLOBAL));
                    translate_expression_component(*image, 0, ctx)?;
                }

                // 2. Push coordinates (x, y)
                translate_expression_component(*coordinate, 0, ctx)?;
                translate_expression_component(*coordinate, 1, ctx)?;

                // 3. Call helper (expects texture_ptr, desc_addr, x, y)
                ctx.wasm_func.instruction(&Instruction::Call(load_idx));

                // 6. Store all 4 results
                let sample_base = ctx
                    .sample_f32_locals
                    .expect("Sampling locals not allocated for function with ImageLoad?");

                ctx.wasm_func
                    .instruction(&Instruction::LocalSet(sample_base + 3)); // a
                ctx.wasm_func
                    .instruction(&Instruction::LocalSet(sample_base + 2)); // b
                ctx.wasm_func
                    .instruction(&Instruction::LocalSet(sample_base + 1)); // g
                ctx.wasm_func
                    .instruction(&Instruction::LocalSet(sample_base)); // r

                // 7. Load requested component
                ctx.wasm_func
                    .instruction(&Instruction::LocalGet(sample_base + component_idx));

                // Reinterpret if it's an integer image
                let ty_handle = ctx.typifier[*image].handle().unwrap();
                let is_integer = match ctx.module.types[ty_handle].inner {
                    naga::TypeInner::Image {
                        class: naga::ImageClass::Sampled { kind, .. },
                        ..
                    } => kind != naga::ScalarKind::Float,
                    _ => false,
                };
                if is_integer {
                    ctx.wasm_func.instruction(&Instruction::I32ReinterpretF32);
                }
            } else {
                ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
            }
        }
        Expression::ImageQuery { image, query } => {
            match query {
                naga::ImageQuery::Size { .. } => {
                    // 1. Resolve descriptor address
                    if let Expression::GlobalVariable(h) = ctx.func.expressions[*image] {
                        if let Some(&(offset, base_ptr_idx)) = ctx.global_offsets.get(&h) {
                            if base_ptr_idx == output_layout::UNIFORM_PTR_GLOBAL {
                                // Index model (WebGL)
                                ctx.wasm_func.instruction(&Instruction::GlobalGet(
                                    output_layout::UNIFORM_PTR_GLOBAL,
                                ));
                                let final_offset = offset;
                                if final_offset > 0 {
                                    ctx.wasm_func
                                        .instruction(&Instruction::I32Const(final_offset as i32));
                                    ctx.wasm_func.instruction(&Instruction::I32Add);
                                }
                                ctx.wasm_func.instruction(&Instruction::I32Load(
                                    wasm_encoder::MemArg {
                                        offset: 0,
                                        align: 2,
                                        memory_index: 0,
                                    },
                                ));
                                ctx.wasm_func.instruction(&Instruction::I32Const(64));
                                ctx.wasm_func.instruction(&Instruction::I32Mul);
                                ctx.wasm_func.instruction(&Instruction::GlobalGet(
                                    output_layout::TEXTURE_PTR_GLOBAL,
                                ));
                                ctx.wasm_func.instruction(&Instruction::I32Add);
                            } else {
                                // Handle model (WebGPU)
                                translate_expression_component(*image, 0, ctx)?;
                            }
                        } else {
                            translate_expression_component(*image, 0, ctx)?;
                        }
                    } else {
                        translate_expression_component(*image, 0, ctx)?;
                    }

                    // index 0 -> width, index 1 -> height, index 3 -> depth
                    if component_idx == 0 {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                                offset: 0,
                                align: 2,
                                memory_index: 0,
                            }));
                    } else if component_idx == 1 {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                                offset: 4,
                                align: 2,
                                memory_index: 0,
                            }));
                    } else if component_idx == 2 {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                                offset: 12,
                                align: 2,
                                memory_index: 0,
                            }));
                    } else {
                        ctx.wasm_func.instruction(&Instruction::I32Const(0));
                    }
                }
                _ => {
                    ctx.wasm_func.instruction(&Instruction::I32Const(0));
                }
            }
        }
        Expression::Select {
            condition,
            accept,
            reject,
        } => {
            let cond_ty = ctx.typifier.get(*condition, &ctx.module.types);
            let cond_count = super::types::component_count(cond_ty, &ctx.module.types);
            let cond_idx = if cond_count > 1 { component_idx } else { 0 };

            // WASM select: [val1, val2, condition] -> if condition != 0 then val1 else val2
            translate_expression_component(*accept, component_idx, ctx)?;
            translate_expression_component(*reject, component_idx, ctx)?;
            translate_expression_component(*condition, cond_idx, ctx)?;
            ctx.wasm_func.instruction(&Instruction::Select);
        }
        Expression::Math {
            fun,
            arg,
            arg1,
            arg2,
            arg3: _,
        } => {
            match fun {
                MathFunction::Sin
                | MathFunction::Cos
                | MathFunction::Tan
                | MathFunction::Asin
                | MathFunction::Acos
                | MathFunction::Atan
                | MathFunction::Atan2
                | MathFunction::Exp
                | MathFunction::Exp2
                | MathFunction::Log
                | MathFunction::Log2
                | MathFunction::Pow
                | MathFunction::Sinh
                | MathFunction::Cosh
                | MathFunction::Tanh
                | MathFunction::Asinh
                | MathFunction::Acosh
                | MathFunction::Atanh => {
                    translate_expression_component(*arg, component_idx, ctx)?;
                    if let Some(a1) = arg1 {
                        translate_expression_component(*a1, component_idx, ctx)?;
                    }
                    let func_idx = *ctx.math_import_map.get(fun).expect("Math import missing");
                    ctx.wasm_func.instruction(&Instruction::Call(func_idx));
                }
                MathFunction::Abs => {
                    translate_expression_component(*arg, component_idx, ctx)?;
                    let ty = ctx.typifier.get(*arg, &ctx.module.types);
                    if is_integer_type(ty) {
                        // abs(x) = (x ^ (x >> 31)) - (x >> 31)
                        // But simpler for WASM: push 0, sub, then select max
                        ctx.wasm_func
                            .instruction(&Instruction::LocalTee(ctx.swap_i32_local));
                        ctx.wasm_func.instruction(&Instruction::I32Const(0));
                        ctx.wasm_func
                            .instruction(&Instruction::LocalGet(ctx.swap_i32_local));
                        ctx.wasm_func.instruction(&Instruction::I32Sub);
                        ctx.wasm_func
                            .instruction(&Instruction::LocalGet(ctx.swap_i32_local));
                        ctx.wasm_func.instruction(&Instruction::I32Const(0));
                        ctx.wasm_func.instruction(&Instruction::I32LtS);
                        ctx.wasm_func.instruction(&Instruction::Select);
                    } else {
                        ctx.wasm_func.instruction(&Instruction::F32Abs);
                    }
                }
                MathFunction::Min => {
                    translate_expression_component(*arg, component_idx, ctx)?;
                    if let Some(a1) = arg1 {
                        translate_expression_component(*a1, component_idx, ctx)?;
                        let ty = ctx.typifier.get(*arg, &ctx.module.types);
                        if is_integer_type(ty) {
                            // i32.min: select if a < b
                            let temp_a = ctx.swap_i32_local;
                            let temp_b = ctx.swap_f32_local; // repurpose as i32 if needed, but better to use i32 swap
                            ctx.wasm_func.instruction(&Instruction::LocalSet(temp_b)); // actually i32 bits
                            ctx.wasm_func.instruction(&Instruction::LocalTee(temp_a));
                            ctx.wasm_func.instruction(&Instruction::LocalGet(temp_b));
                            ctx.wasm_func.instruction(&Instruction::LocalGet(temp_a));
                            ctx.wasm_func.instruction(&Instruction::LocalGet(temp_b));
                            ctx.wasm_func.instruction(&Instruction::I32LtS);
                            ctx.wasm_func.instruction(&Instruction::Select);
                        } else {
                            ctx.wasm_func.instruction(&Instruction::F32Min);
                        }
                    }
                }
                MathFunction::Max => {
                    translate_expression_component(*arg, component_idx, ctx)?;
                    if let Some(a1) = arg1 {
                        translate_expression_component(*a1, component_idx, ctx)?;
                        let ty = ctx.typifier.get(*arg, &ctx.module.types);
                        if is_integer_type(ty) {
                            let temp_a = ctx.swap_i32_local;
                            let temp_b = ctx.swap_f32_local;
                            ctx.wasm_func.instruction(&Instruction::LocalSet(temp_b));
                            ctx.wasm_func.instruction(&Instruction::LocalTee(temp_a));
                            ctx.wasm_func.instruction(&Instruction::LocalGet(temp_b));
                            ctx.wasm_func.instruction(&Instruction::LocalGet(temp_a));
                            ctx.wasm_func.instruction(&Instruction::LocalGet(temp_b));
                            ctx.wasm_func.instruction(&Instruction::I32GtS);
                            ctx.wasm_func.instruction(&Instruction::Select);
                        } else {
                            ctx.wasm_func.instruction(&Instruction::F32Max);
                        }
                    }
                }
                MathFunction::Clamp => {
                    // clamp(x, min, max) = max(min(x, max), min)
                    let x = *arg;
                    let min_val = arg1.expect("Clamp needs 3 arguments");
                    let max_val = arg2.expect("Clamp needs 3 arguments");

                    let ty = ctx.typifier.get(x, &ctx.module.types);
                    let is_int = is_integer_type(ty);

                    if is_int {
                        // min(x, max)
                        translate_expression_component(x, component_idx, ctx)?;
                        translate_expression_component(max_val, component_idx, ctx)?;
                        let temp_a = ctx.swap_i32_local;
                        let temp_b = ctx.swap_f32_local;
                        ctx.wasm_func.instruction(&Instruction::LocalSet(temp_b));
                        ctx.wasm_func.instruction(&Instruction::LocalTee(temp_a));
                        ctx.wasm_func.instruction(&Instruction::LocalGet(temp_b));
                        ctx.wasm_func.instruction(&Instruction::LocalGet(temp_a));
                        ctx.wasm_func.instruction(&Instruction::LocalGet(temp_b));
                        ctx.wasm_func.instruction(&Instruction::I32LtS);
                        ctx.wasm_func.instruction(&Instruction::Select);

                        // max(result, min)
                        translate_expression_component(min_val, component_idx, ctx)?;
                        // stack: [result_min, min]
                        ctx.wasm_func.instruction(&Instruction::LocalSet(temp_b));
                        ctx.wasm_func.instruction(&Instruction::LocalTee(temp_a));
                        ctx.wasm_func.instruction(&Instruction::LocalGet(temp_b));
                        ctx.wasm_func.instruction(&Instruction::LocalGet(temp_a));
                        ctx.wasm_func.instruction(&Instruction::LocalGet(temp_b));
                        ctx.wasm_func.instruction(&Instruction::I32GtS);
                        ctx.wasm_func.instruction(&Instruction::Select);
                    } else {
                        translate_expression_component(x, component_idx, ctx)?;
                        translate_expression_component(max_val, component_idx, ctx)?;
                        ctx.wasm_func.instruction(&Instruction::F32Min);
                        translate_expression_component(min_val, component_idx, ctx)?;
                        ctx.wasm_func.instruction(&Instruction::F32Max);
                    }
                }
                MathFunction::Saturate => {
                    // saturate(x) = clamp(x, 0.0, 1.0)
                    translate_expression_component(*arg, component_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Const(1.0));
                    ctx.wasm_func.instruction(&Instruction::F32Min);
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    ctx.wasm_func.instruction(&Instruction::F32Max);
                }
                MathFunction::Sqrt => {
                    translate_expression_component(*arg, component_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Sqrt);
                }
                MathFunction::Floor => {
                    translate_expression_component(*arg, component_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Floor);
                }
                MathFunction::Ceil => {
                    translate_expression_component(*arg, component_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Ceil);
                }
                MathFunction::Trunc => {
                    translate_expression_component(*arg, component_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Trunc);
                }
                MathFunction::Round => {
                    translate_expression_component(*arg, component_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Nearest);
                }
                MathFunction::Fract => {
                    // fract(x) = x - floor(x)
                    translate_expression_component(*arg, component_idx, ctx)?;
                    ctx.wasm_func
                        .instruction(&Instruction::LocalTee(ctx.swap_f32_local));
                    ctx.wasm_func.instruction(&Instruction::F32Floor);
                    ctx.wasm_func.instruction(&Instruction::F32Neg);
                    ctx.wasm_func
                        .instruction(&Instruction::LocalGet(ctx.swap_f32_local));
                    ctx.wasm_func.instruction(&Instruction::F32Add);
                }
                MathFunction::Transpose => {
                    let arg_ty = ctx.typifier.get(*arg, &ctx.module.types);
                    if let naga::TypeInner::Matrix { columns, rows, .. } = arg_ty {
                        // result is rows x columns (input was columns x rows)
                        let res_rows = *columns as u32;
                        let row = component_idx % res_rows;
                        let col = component_idx / res_rows;
                        // result[col][row] = input[row][col]
                        translate_expression_component(*arg, row * (*rows as u32) + col, ctx)?;
                    } else {
                        ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    }
                }
                MathFunction::Outer => {
                    let left = arg;
                    let right = arg1.as_ref().unwrap();
                    let left_ty = ctx.typifier.get(*left, &ctx.module.types);
                    let right_ty = ctx.typifier.get(*right, &ctx.module.types);
                    if let (
                        naga::TypeInner::Vector { size: rows, .. },
                        naga::TypeInner::Vector { size: _cols, .. },
                    ) = (left_ty, right_ty)
                    {
                        // result[col][row] = left[row] * right[col]
                        let row = component_idx % (*rows as u32);
                        let col = component_idx / (*rows as u32);
                        translate_expression_component(*left, row, ctx)?;
                        translate_expression_component(*right, col, ctx)?;
                        ctx.wasm_func.instruction(&Instruction::F32Mul);
                    } else {
                        ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    }
                }
                MathFunction::Step => {
                    // step(edge, x) = x < edge ? 0.0 : 1.0
                    translate_expression_component(*arg1.as_ref().unwrap(), component_idx, ctx)?; // x
                    translate_expression_component(*arg, component_idx, ctx)?; // edge
                    ctx.wasm_func.instruction(&Instruction::F32Lt);
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    ctx.wasm_func.instruction(&Instruction::F32Const(1.0));
                    ctx.wasm_func.instruction(&Instruction::Select);
                }
                MathFunction::Dot => {
                    if component_idx == 0 {
                        let arg_ty = ctx.typifier.get(*arg, &ctx.module.types);
                        let count = super::types::component_count(arg_ty, &ctx.module.types);

                        ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                        for j in 0..count {
                            translate_expression_component(*arg, j, ctx)?;
                            translate_expression_component(*arg1.as_ref().unwrap(), j, ctx)?;
                            ctx.wasm_func.instruction(&Instruction::F32Mul);
                            ctx.wasm_func.instruction(&Instruction::F32Add);
                        }
                    } else {
                        // Dot product result is a scalar at component 0
                        ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    }
                }
                MathFunction::Length => {
                    if component_idx == 0 {
                        let arg_ty = ctx.typifier.get(*arg, &ctx.module.types);
                        let count = super::types::component_count(arg_ty, &ctx.module.types);

                        ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                        for j in 0..count {
                            translate_expression_component(*arg, j, ctx)?;
                            translate_expression_component(*arg, j, ctx)?;
                            ctx.wasm_func.instruction(&Instruction::F32Mul);
                            ctx.wasm_func.instruction(&Instruction::F32Add);
                        }
                        ctx.wasm_func.instruction(&Instruction::F32Sqrt);
                    } else {
                        ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    }
                }
                MathFunction::Normalize => {
                    // normalize(v) = v / length(v)
                    // We need to calculate length(v) and store it in a temp local
                    let arg_ty = ctx.typifier.get(*arg, &ctx.module.types);
                    let count = super::types::component_count(arg_ty, &ctx.module.types);

                    // We need a scratch F32 local. Use swap_f32_local_2 if available.
                    let temp_len = ctx
                        .swap_f32_local_2
                        .expect("Normalize needs secondary swap local");

                    // First compute length in a separate pass if we haven't yet for this expression
                    // For efficiency in scalarized emission, we might be recalculating this every component.
                    // But for now, simple and correct:
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    for j in 0..count {
                        translate_expression_component(*arg, j, ctx)?;
                        translate_expression_component(*arg, j, ctx)?;
                        ctx.wasm_func.instruction(&Instruction::F32Mul);
                        ctx.wasm_func.instruction(&Instruction::F32Add);
                    }
                    ctx.wasm_func.instruction(&Instruction::F32Sqrt);
                    ctx.wasm_func.instruction(&Instruction::LocalSet(temp_len));

                    // v[component_idx] / length
                    translate_expression_component(*arg, component_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::LocalGet(temp_len));
                    ctx.wasm_func.instruction(&Instruction::F32Div);
                }
                MathFunction::InverseSqrt => {
                    ctx.wasm_func.instruction(&Instruction::F32Const(1.0));
                    translate_expression_component(*arg, component_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Sqrt);
                    ctx.wasm_func.instruction(&Instruction::F32Div);
                }
                MathFunction::Sign => {
                    translate_expression_component(*arg, component_idx, ctx)?;
                    let ty = ctx.typifier.get(*arg, &ctx.module.types);
                    if is_integer_type(ty) {
                        ctx.wasm_func
                            .instruction(&Instruction::LocalTee(ctx.swap_i32_local));
                        ctx.wasm_func.instruction(&Instruction::I32Const(0));
                        ctx.wasm_func.instruction(&Instruction::I32GtS);
                        ctx.wasm_func.instruction(&Instruction::I32Const(1));

                        ctx.wasm_func
                            .instruction(&Instruction::LocalGet(ctx.swap_i32_local));
                        ctx.wasm_func.instruction(&Instruction::I32Const(0));
                        ctx.wasm_func.instruction(&Instruction::I32LtS);
                        ctx.wasm_func.instruction(&Instruction::I32Const(-1));
                        ctx.wasm_func.instruction(&Instruction::I32Const(0));
                        ctx.wasm_func.instruction(&Instruction::Select);

                        ctx.wasm_func.instruction(&Instruction::Select);
                    } else {
                        ctx.wasm_func
                            .instruction(&Instruction::LocalTee(ctx.swap_f32_local));
                        ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                        ctx.wasm_func.instruction(&Instruction::F32Gt);
                        ctx.wasm_func.instruction(&Instruction::F32Const(1.0));

                        ctx.wasm_func
                            .instruction(&Instruction::LocalGet(ctx.swap_f32_local));
                        ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                        ctx.wasm_func.instruction(&Instruction::F32Lt);
                        ctx.wasm_func.instruction(&Instruction::F32Const(-1.0));
                        ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                        ctx.wasm_func.instruction(&Instruction::Select);

                        ctx.wasm_func.instruction(&Instruction::Select);
                    }
                }
                MathFunction::Cross => {
                    let a = *arg;
                    let b = *arg1.as_ref().unwrap();
                    match component_idx {
                        0 => {
                            translate_expression_component(a, 1, ctx)?;
                            translate_expression_component(b, 2, ctx)?;
                            ctx.wasm_func.instruction(&Instruction::F32Mul);
                            translate_expression_component(a, 2, ctx)?;
                            translate_expression_component(b, 1, ctx)?;
                            ctx.wasm_func.instruction(&Instruction::F32Mul);
                            ctx.wasm_func.instruction(&Instruction::F32Sub);
                        }
                        1 => {
                            translate_expression_component(a, 2, ctx)?;
                            translate_expression_component(b, 0, ctx)?;
                            ctx.wasm_func.instruction(&Instruction::F32Mul);
                            translate_expression_component(a, 0, ctx)?;
                            translate_expression_component(b, 2, ctx)?;
                            ctx.wasm_func.instruction(&Instruction::F32Mul);
                            ctx.wasm_func.instruction(&Instruction::F32Sub);
                        }
                        2 => {
                            translate_expression_component(a, 0, ctx)?;
                            translate_expression_component(b, 1, ctx)?;
                            ctx.wasm_func.instruction(&Instruction::F32Mul);
                            translate_expression_component(a, 1, ctx)?;
                            translate_expression_component(b, 0, ctx)?;
                            ctx.wasm_func.instruction(&Instruction::F32Mul);
                            ctx.wasm_func.instruction(&Instruction::F32Sub);
                        }
                        _ => {
                            ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                        }
                    }
                }
                MathFunction::Radians => {
                    translate_expression_component(*arg, component_idx, ctx)?;
                    ctx.wasm_func
                        .instruction(&Instruction::F32Const(std::f32::consts::PI / 180.0));
                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                }
                MathFunction::Degrees => {
                    translate_expression_component(*arg, component_idx, ctx)?;
                    ctx.wasm_func
                        .instruction(&Instruction::F32Const(180.0 / std::f32::consts::PI));
                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                }
                MathFunction::Mix => {
                    let a = *arg;
                    let b = *arg1.as_ref().unwrap();
                    let t = *arg2.as_ref().unwrap();

                    let ty_t = ctx.typifier.get(t, &ctx.module.types);
                    let t_count = super::types::component_count(ty_t, &ctx.module.types);
                    let t_idx = if t_count > 1 { component_idx } else { 0 };

                    translate_expression_component(a, component_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Const(1.0));
                    translate_expression_component(t, t_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Sub);
                    ctx.wasm_func.instruction(&Instruction::F32Mul);

                    translate_expression_component(b, component_idx, ctx)?;
                    translate_expression_component(t, t_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Mul);

                    ctx.wasm_func.instruction(&Instruction::F32Add);
                }
                _ => {
                    // Default to 0.0 for unimplemented math functions
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                }
            }
        }
        Expression::Derivative { .. } => {
            // Derivatives are currently unimplemented in soft emulator
            ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
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
            RelationalFunction::IsNan => {
                // NaN check: x != x (standard NaN test)
                translate_expression_component(*argument, component_idx, ctx)?;
                // Duplicate the value on stack
                translate_expression_component(*argument, component_idx, ctx)?;
                // Compare: x != x returns true only for NaN
                ctx.wasm_func.instruction(&Instruction::F32Ne);
            }
            RelationalFunction::IsInf => {
                // Infinity check: abs(x) == Infinity
                translate_expression_component(*argument, component_idx, ctx)?;
                // Get absolute value
                ctx.wasm_func.instruction(&Instruction::F32Abs);
                // Push infinity constant
                ctx.wasm_func
                    .instruction(&Instruction::F32Const(f32::INFINITY));
                // Compare: abs(x) == Infinity
                ctx.wasm_func.instruction(&Instruction::F32Eq);
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
                let var = &ctx.module.global_variables[*handle];
                if let Some(&(offset, base_ptr_idx)) = ctx.global_offsets.get(handle) {
                    ctx.wasm_func
                        .instruction(&Instruction::GlobalGet(base_ptr_idx));

                    if base_ptr_idx == output_layout::UNIFORM_PTR_GLOBAL
                        || base_ptr_idx == output_layout::TEXTURE_PTR_GLOBAL
                    {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Const(offset as i32));
                        ctx.wasm_func.instruction(&Instruction::I32Add);
                        ctx.wasm_func
                            .instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                                offset: 0,
                                align: 2,
                                memory_index: 0,
                            }));
                    } else if offset > 0 {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Const(offset as i32));
                        ctx.wasm_func.instruction(&Instruction::I32Add);
                    }
                } else {
                    // Fallback for unknown globals
                    match var.name.as_deref() {
                        Some("gl_Position") | Some("gl_Position_1") => {
                            ctx.wasm_func.instruction(&Instruction::GlobalGet(
                                output_layout::VARYING_PTR_GLOBAL,
                            ));
                        }
                        Some(name)
                            if name.starts_with("output")
                                || name == "outColor"
                                || name == "fragColor"
                                || name == "gl_FragColor" =>
                        {
                            ctx.wasm_func.instruction(&Instruction::GlobalGet(
                                output_layout::PRIVATE_PTR_GLOBAL,
                            ));
                        }
                        _ => {
                            if var.space == naga::AddressSpace::Uniform
                                || var.space == naga::AddressSpace::Handle
                            {
                                ctx.wasm_func.instruction(&Instruction::GlobalGet(
                                    output_layout::UNIFORM_PTR_GLOBAL,
                                ));
                            } else {
                                ctx.wasm_func.instruction(&Instruction::GlobalGet(
                                    output_layout::PRIVATE_PTR_GLOBAL,
                                ));
                            }
                        }
                    }
                }
            }
            Expression::AccessIndex { base, index } => {
                translate_expression(*base, ctx)?;
                // Use element size for indexing instead of assuming 4 bytes
                let base_ty = ctx.typifier.get(*base, &ctx.module.types);
                if let TypeInner::Pointer {
                    base: pointed_ty, ..
                } = base_ty
                {
                    let element_inner = &ctx.module.types[*pointed_ty].inner;
                    let element_size = match element_inner {
                        naga::TypeInner::Array { stride, .. } => *stride,
                        _ => super::types::type_size(element_inner).unwrap_or(4),
                    };
                    if *index > 0 {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Const((*index * element_size) as i32));
                        ctx.wasm_func.instruction(&Instruction::I32Add);
                    }
                } else {
                    // Fallback to 4-byte stride if not a pointer
                    if *index > 0 {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Const((*index * 4) as i32));
                        ctx.wasm_func.instruction(&Instruction::I32Add);
                    }
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
