//! Expression translation from Naga IR to WASM
//!
//! Phase 0: Placeholder for future expression handling

use super::{output_layout, BackendError, TranslationContext};

use naga::{
    BinaryOperator, Expression, Literal, MathFunction, RelationalFunction, ScalarKind, TypeInner,
};
use wasm_encoder::{Instruction, ValType};

/// Helper function to determine if a type should use I32 operations
/// Helper function to determine if a type should use I32 operations
pub fn is_integer_type(type_inner: &TypeInner, types: &naga::UniqueArena<naga::Type>) -> bool {
    match type_inner {
        TypeInner::Scalar(s) => matches!(
            s.kind,
            ScalarKind::Sint | ScalarKind::Uint | ScalarKind::Bool
        ),
        TypeInner::Vector { scalar, .. } => matches!(
            scalar.kind,
            ScalarKind::Sint | ScalarKind::Uint | ScalarKind::Bool
        ),
        TypeInner::Image { .. } | TypeInner::Sampler { .. } => true,
        TypeInner::Atomic(s) => matches!(
            s.kind,
            ScalarKind::Sint | ScalarKind::Uint | ScalarKind::Bool
        ),
        TypeInner::Array { base, .. } => is_integer_type(&types[*base].inner, types),
        TypeInner::Struct { members, .. } => {
            // Structs are considered "integer" if they contain only integers?
            // No, that's not right. Structs should be handled member by member.
            // But for simple "use_i32_store" check, we just check first member?
            // Fallback to false for now.
            if let Some(m) = members.first() {
                is_integer_type(&types[m.ty].inner, types)
            } else {
                false
            }
        }
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
                if is_integer_type(inner, &ctx.module.types) {
                    ctx.wasm_func.instruction(&Instruction::I32Const(0));
                } else {
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                }
            }
        }
        Expression::ZeroValue(ty) => {
            let inner = &ctx.module.types[*ty].inner;
            if is_integer_type(inner, &ctx.module.types) {
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
                if is_integer_type(inner, &ctx.module.types) {
                    ctx.wasm_func.instruction(&Instruction::I32Const(0));
                } else {
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                }
            }
        }
        Expression::ZeroValue(ty) => {
            let inner = &ctx.module.types[*ty].inner;
            if is_integer_type(inner, &ctx.module.types) {
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
            let is_int = is_integer_type(expr_ty, &ctx.module.types);

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
            let arg = &ctx.func.arguments[*idx as usize];
            let mut override_is_int = false;

            if ctx.is_entry_point {
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

                // Use I32Load for integer types, F32Load for float types
                let is_int = is_integer_type(arg_ty, &ctx.module.types) || override_is_int;

                if is_int {
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
                // If it's an internal function, we use LocalGet or Load from pointer
                let base_idx = ctx.argument_local_offsets.get(idx).cloned().unwrap_or(*idx);
                let arg = &ctx.func.arguments[*idx as usize];
                let arg_ty = &ctx.module.types[arg.ty].inner;

                // Check if this argument was passed by Frame using the actual ABI
                // or if it's already a pointer in Naga IR
                let is_ptr_like = if let Some(abi) = ctx.abi {
                    if (*idx as usize) < abi.params.len() {
                        matches!(
                            abi.params[*idx as usize],
                            super::function_abi::ParameterABI::Frame { .. }
                        )
                    } else {
                        matches!(
                            arg_ty,
                            TypeInner::Pointer { .. } | TypeInner::ValuePointer { .. }
                        )
                    }
                } else {
                    matches!(
                        arg_ty,
                        TypeInner::Pointer { .. } | TypeInner::ValuePointer { .. }
                    ) || super::types::type_size(arg_ty).unwrap_or(0) > 16
                };

                if is_ptr_like {
                    // Pointer-like argument (Frame-passed or real Pointer)
                    // Load the component from that pointer.
                    ctx.wasm_func.instruction(&Instruction::LocalGet(base_idx));

                    if component_idx > 0 {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Const((component_idx * 4) as i32));
                        ctx.wasm_func.instruction(&Instruction::I32Add);
                    }

                    // Determine if the component is an integer
                    let is_int = match arg_ty {
                        TypeInner::Array { base, .. } => {
                            is_integer_type(&ctx.module.types[*base].inner, &ctx.module.types)
                        }
                        TypeInner::Struct { members, .. } => {
                            let mut current_comp = 0;
                            let mut found_int = false;
                            for member in members {
                                let member_ty = &ctx.module.types[member.ty].inner;
                                let count =
                                    super::types::component_count(member_ty, &ctx.module.types);
                                if component_idx < current_comp + count {
                                    found_int = is_integer_type(member_ty, &ctx.module.types);
                                    break;
                                }
                                current_comp += count;
                            }
                            found_int
                        }
                        TypeInner::Pointer { base, .. } => {
                            // Elements of the pointed type
                            let pointed_inner = &ctx.module.types[*base].inner;
                            match pointed_inner {
                                TypeInner::Array {
                                    base: elem_base, ..
                                } => is_integer_type(
                                    &ctx.module.types[*elem_base].inner,
                                    &ctx.module.types,
                                ),
                                _ => is_integer_type(pointed_inner, &ctx.module.types),
                            }
                        }
                        _ => is_integer_type(arg_ty, &ctx.module.types),
                    } || override_is_int;

                    if is_int {
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
                    ctx.wasm_func
                        .instruction(&Instruction::LocalGet(base_idx + component_idx));
                }
            }
        }
        Expression::LocalVariable(handle) => {
            if let Some(&ptr_local) = ctx.local_offsets.get(handle) {
                // Return the VALUE of the component by loading from the local's memory
                ctx.wasm_func
                    .instruction(&Instruction::GlobalGet(output_layout::PRIVATE_PTR_GLOBAL));
                ctx.wasm_func.instruction(&Instruction::I32Const(
                    (ptr_local + component_idx * 4) as i32,
                ));
                ctx.wasm_func.instruction(&Instruction::I32Add);

                let ty = &ctx.module.types[ctx.func.local_variables[*handle].ty].inner;
                if is_integer_type(ty, &ctx.module.types) {
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
                ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
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

                // If this is a WebGPU uniform, follow the context block indirection
                let final_offset = component_idx * 4;
                if final_offset > 0 {
                    ctx.wasm_func
                        .instruction(&Instruction::I32Const(final_offset as i32));
                    ctx.wasm_func.instruction(&Instruction::I32Add);
                }

                // If this is a WebGPU uniform, follow the context block indirection
                if ctx.uniform_locations.is_empty()
                    && base_ptr_idx == output_layout::UNIFORM_PTR_GLOBAL
                {
                    // Choose the correct load type based on the global variable's base type
                    let var = &ctx.module.global_variables[*handle];
                    let var_ty = &ctx.module.types[var.ty].inner;
                    let uses_int = is_integer_type(var_ty, &ctx.module.types);

                    if uses_int {
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

            if override_is_int || is_integer_type(load_ty, &ctx.module.types) {
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
                }
                | naga::TypeInner::Array {
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

                    if override_is_int || is_integer_type(element_ty, &ctx.module.types) {
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
                    // Accessing a component of a value (e.g. array of vectors or vector)
                    let elem_count = match base_ty {
                        naga::TypeInner::Array { base, .. } => super::types::component_count(
                            &ctx.module.types[*base].inner,
                            &ctx.module.types,
                        ),
                        naga::TypeInner::Vector { .. } => 1,
                        naga::TypeInner::Matrix { rows, .. } => *rows as u32,
                        naga::TypeInner::Struct { members, .. } => {
                            // Sum of components of previous members
                            let mut sum = 0;
                            for i in 0..*index {
                                sum += super::types::component_count(
                                    &ctx.module.types[members[i as usize].ty].inner,
                                    &ctx.module.types,
                                );
                            }
                            sum
                        }
                        _ => 1,
                    };

                    let final_idx = if matches!(base_ty, naga::TypeInner::Struct { .. }) {
                        elem_count + component_idx
                    } else {
                        (*index * elem_count) + component_idx
                    };

                    translate_expression_component(*base, final_idx, ctx)?;
                }
            }
        }
        Expression::Access { base, index } => {
            let base_ty = ctx.typifier.get(*base, &ctx.module.types);
            match base_ty {
                naga::TypeInner::Pointer {
                    base: pointed_ty, ..
                }
                | naga::TypeInner::Array {
                    base: pointed_ty, ..
                } => {
                    // Base is a pointer or array in memory, calculate address
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

                    if override_is_int || is_integer_type(element_ty, &ctx.module.types) {
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
                        if !ctx.uniform_locations.is_empty() {
                            // WebGL model (with indirection or direct texture ptr)
                            if let Expression::GlobalVariable(h) = ctx.func.expressions[h_expr] {
                                if let Some(&(offset, base_ptr_idx)) = ctx.global_offsets.get(&h) {
                                    let var = &ctx.module.global_variables[h];
                                    if base_ptr_idx == output_layout::UNIFORM_PTR_GLOBAL
                                        && var.space == naga::AddressSpace::Handle
                                    {
                                        // Index model (WebGL): Load unit index from uniform memory
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
                                        return Ok::<(), BackendError>(());
                                    }
                                }
                            }
                            // Fallback for WebGL (direct handle or other)
                            translate_expression_component(h_expr, 0, ctx)?;
                        } else {
                            // Handle model (WebGPU): Load descriptor pointer from uniform memory
                            // translate_expression_component now follows the indirection automatically.
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
                // 1. Resolve descriptor address
                if !ctx.uniform_locations.is_empty() {
                    if let Expression::GlobalVariable(h) = ctx.func.expressions[*image] {
                        if let Some(&(offset, base_ptr_idx)) = ctx.global_offsets.get(&h) {
                            if base_ptr_idx == output_layout::UNIFORM_PTR_GLOBAL {
                                // Index model (WebGL): Load unit index from uniform memory
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
                                ctx.wasm_func.instruction(&Instruction::I32Const(6));
                                ctx.wasm_func.instruction(&Instruction::I32Shl); // * 64
                                ctx.wasm_func.instruction(&Instruction::GlobalGet(
                                    output_layout::TEXTURE_PTR_GLOBAL,
                                ));
                                ctx.wasm_func.instruction(&Instruction::I32Add);
                            } else {
                                translate_expression_component(*image, 0, ctx)?;
                            }
                        } else {
                            translate_expression_component(*image, 0, ctx)?;
                        }
                    } else {
                        translate_expression_component(*image, 0, ctx)?;
                    }
                } else {
                    // Handle model (WebGPU): Load pointer from uniform address
                    // translate_expression_component now follows the indirection automatically.
                    translate_expression_component(*image, 0, ctx)?;
                }

                // 2. Push coordinates (x, y, z)
                translate_expression_component(*coordinate, 0, ctx)?;
                translate_expression_component(*coordinate, 1, ctx)?;

                let coord_dim = match ctx.typifier.get(*coordinate, &ctx.module.types) {
                    naga::TypeInner::Vector { size, .. } => *size as u32,
                    _ => 1,
                };
                if coord_dim >= 3 {
                    translate_expression_component(*coordinate, 2, ctx)?;
                } else {
                    ctx.wasm_func.instruction(&Instruction::I32Const(0));
                }

                // 3. Call helper (expects texture_ptr, desc_addr, x, y, z)
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
                    if !ctx.uniform_locations.is_empty() {
                        if let Expression::GlobalVariable(h) = ctx.func.expressions[*image] {
                            if let Some(&(offset, base_ptr_idx)) = ctx.global_offsets.get(&h) {
                                if base_ptr_idx == output_layout::UNIFORM_PTR_GLOBAL {
                                    // Index model (WebGL)
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
                                    ctx.wasm_func.instruction(&Instruction::I32Const(64));
                                    ctx.wasm_func.instruction(&Instruction::I32Mul);
                                    ctx.wasm_func.instruction(&Instruction::GlobalGet(
                                        output_layout::TEXTURE_PTR_GLOBAL,
                                    ));
                                    ctx.wasm_func.instruction(&Instruction::I32Add);
                                } else {
                                    translate_expression_component(*image, 0, ctx)?;
                                }
                            } else {
                                translate_expression_component(*image, 0, ctx)?;
                            }
                        } else {
                            translate_expression_component(*image, 0, ctx)?;
                        }
                    } else {
                        // Handle model (WebGPU): Load pointer from uniform address
                        // translate_expression_component now follows the indirection automatically.
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
                    let arg_ty = ctx.typifier.get(*arg, &ctx.module.types);
                    let arg_count = super::types::component_count(arg_ty, &ctx.module.types);
                    let arg_idx = if arg_count > 1 { component_idx } else { 0 };

                    translate_expression_component(*arg, arg_idx, ctx)?;
                    if let Some(a1) = arg1 {
                        let a1_ty = ctx.typifier.get(*a1, &ctx.module.types);
                        let a1_count = super::types::component_count(a1_ty, &ctx.module.types);
                        let a1_idx = if a1_count > 1 { component_idx } else { 0 };
                        translate_expression_component(*a1, a1_idx, ctx)?;
                    }
                    let func_idx = *ctx.math_import_map.get(fun).expect("Math import missing");
                    ctx.wasm_func.instruction(&Instruction::Call(func_idx));
                }
                MathFunction::Abs => {
                    translate_expression_component(*arg, component_idx, ctx)?;
                    let ty = ctx.typifier.get(*arg, &ctx.module.types);
                    if is_integer_type(ty, &ctx.module.types) {
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
                        if is_integer_type(ty, &ctx.module.types) {
                            // i32.min: select if a < b
                            let temp_a = ctx.swap_i32_local;
                            let temp_b = ctx.swap_i32_local; // use i32 swap local explicitly
                            ctx.wasm_func.instruction(&Instruction::LocalSet(temp_b));
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
                        if is_integer_type(ty, &ctx.module.types) {
                            let temp_a = ctx.swap_i32_local;
                            let temp_b = ctx.swap_i32_local; // use i32 swap local explicitly
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
                    let is_int = is_integer_type(ty, &ctx.module.types);

                    if is_int {
                        // min(x, max)
                        translate_expression_component(x, component_idx, ctx)?;
                        translate_expression_component(max_val, component_idx, ctx)?;
                        let temp_a = ctx.swap_i32_local;
                        let temp_b = ctx.swap_i32_local;
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
                    if is_integer_type(ty, &ctx.module.types) {
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
                MathFunction::Fma => {
                    // fma(a, b, c) == a * b + c
                    let a = *arg;
                    let b = *arg1.as_ref().unwrap();
                    let c = *arg2.as_ref().unwrap();

                    let a_ty = ctx.typifier.get(a, &ctx.module.types);
                    let b_ty = ctx.typifier.get(b, &ctx.module.types);
                    let c_ty = ctx.typifier.get(c, &ctx.module.types);

                    let a_count = super::types::component_count(a_ty, &ctx.module.types);
                    let b_count = super::types::component_count(b_ty, &ctx.module.types);
                    let c_count = super::types::component_count(c_ty, &ctx.module.types);

                    let a_idx = if a_count > 1 { component_idx } else { 0 };
                    let b_idx = if b_count > 1 { component_idx } else { 0 };
                    let c_idx = if c_count > 1 { component_idx } else { 0 };

                    translate_expression_component(a, a_idx, ctx)?;
                    translate_expression_component(b, b_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                    translate_expression_component(c, c_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Add);
                }
                MathFunction::SmoothStep => {
                    // smoothstep(edge0, edge1, x)
                    let edge0 = *arg;
                    let edge1 = *arg1.as_ref().unwrap();
                    let x = *arg2.as_ref().unwrap();

                    let e0_ty = ctx.typifier.get(edge0, &ctx.module.types);
                    let e1_ty = ctx.typifier.get(edge1, &ctx.module.types);
                    let x_ty = ctx.typifier.get(x, &ctx.module.types);

                    let e0_count = super::types::component_count(e0_ty, &ctx.module.types);
                    let e1_count = super::types::component_count(e1_ty, &ctx.module.types);
                    let x_count = super::types::component_count(x_ty, &ctx.module.types);

                    let e0_idx = if e0_count > 1 { component_idx } else { 0 };
                    let e1_idx = if e1_count > 1 { component_idx } else { 0 };
                    let x_idx = if x_count > 1 { component_idx } else { 0 };

                    // t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0)
                    translate_expression_component(x, x_idx, ctx)?;
                    translate_expression_component(edge0, e0_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Sub);

                    translate_expression_component(edge1, e1_idx, ctx)?;
                    translate_expression_component(edge0, e0_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Sub);

                    ctx.wasm_func.instruction(&Instruction::F32Div);

                    // clamp to [0,1]
                    ctx.wasm_func.instruction(&Instruction::F32Const(1.0));
                    ctx.wasm_func.instruction(&Instruction::F32Min);
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    ctx.wasm_func.instruction(&Instruction::F32Max);

                    // store t into temp
                    let temp = ctx.swap_f32_local;
                    ctx.wasm_func.instruction(&Instruction::LocalSet(temp));

                    // t * t
                    ctx.wasm_func.instruction(&Instruction::LocalGet(temp));
                    ctx.wasm_func.instruction(&Instruction::LocalGet(temp));
                    ctx.wasm_func.instruction(&Instruction::F32Mul);

                    // 3.0 - 2.0 * t
                    ctx.wasm_func.instruction(&Instruction::F32Const(3.0));
                    ctx.wasm_func.instruction(&Instruction::LocalGet(temp));
                    ctx.wasm_func.instruction(&Instruction::F32Const(2.0));
                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                    ctx.wasm_func.instruction(&Instruction::F32Sub);

                    // t*t * (3 - 2*t)
                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                }
                MathFunction::Ldexp => {
                    // ldexp(mantissa, exponent) == mantissa * exp2(float(exponent))
                    let mant = *arg;
                    let exp = *arg1.as_ref().unwrap();

                    // exponent may be integer or float; handle scalar/vector cases
                    let exp_ty = ctx.typifier.get(exp, &ctx.module.types);
                    let exp_count = super::types::component_count(exp_ty, &ctx.module.types);
                    let exp_idx = if exp_count > 1 { component_idx } else { 0 };

                    // Emit: mant * exp2(float(exp))
                    translate_expression_component(mant, component_idx, ctx)?;
                    translate_expression_component(exp, exp_idx, ctx)?;
                    if is_integer_type(exp_ty, &ctx.module.types) {
                        ctx.wasm_func.instruction(&Instruction::F32ConvertI32S);
                    }
                    // Call gl_exp2 to compute exp2(exp)
                    let exp2_idx = *ctx
                        .math_import_map
                        .get(&MathFunction::Exp2)
                        .expect("Math import missing");
                    ctx.wasm_func.instruction(&Instruction::Call(exp2_idx));
                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                }
                MathFunction::Refract => {
                    // refract(I, N, eta)
                    let i = *arg;
                    let n = *arg1.as_ref().unwrap();
                    let eta = *arg2.as_ref().unwrap();

                    // component count (vec size)
                    let arg_ty = ctx.typifier.get(i, &ctx.module.types);
                    let count = super::types::component_count(arg_ty, &ctx.module.types);

                    // eta may be scalar or vector
                    let eta_ty = ctx.typifier.get(eta, &ctx.module.types);
                    let eta_count = super::types::component_count(eta_ty, &ctx.module.types);
                    let eta_idx = if eta_count > 1 { component_idx } else { 0 };

                    // We'll compute k and store it in a single scratch local (swap_f32_local),
                    // and recompute dot per-component when needed to avoid clobbering shared temps.
                    let temp_k = ctx.swap_f32_local;

                    // compute k = 1.0 - A * (1 - dot*dot)
                    // Push the 1.0 FIRST so the final subtraction is 1.0 - result
                    ctx.wasm_func.instruction(&Instruction::F32Const(1.0));

                    // Compute A = eta * eta
                    translate_expression_component(eta, eta_idx, ctx)?;
                    translate_expression_component(eta, eta_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Mul);

                    // Prepare for computing (1 - dot*dot): push 1.0 then compute dot twice and multiply
                    ctx.wasm_func.instruction(&Instruction::F32Const(1.0));

                    // first dot (N · I)
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    for j in 0..count {
                        translate_expression_component(n, j, ctx)?;
                        translate_expression_component(i, j, ctx)?;
                        ctx.wasm_func.instruction(&Instruction::F32Mul);
                        ctx.wasm_func.instruction(&Instruction::F32Add);
                    }

                    // second dot (N · I)
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    for j in 0..count {
                        translate_expression_component(n, j, ctx)?;
                        translate_expression_component(i, j, ctx)?;
                        ctx.wasm_func.instruction(&Instruction::F32Mul);
                        ctx.wasm_func.instruction(&Instruction::F32Add);
                    }

                    // dot * dot
                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                    // compute 1.0 - dot*dot
                    ctx.wasm_func.instruction(&Instruction::F32Sub);

                    // multiply A * (1 - dot*dot)
                    ctx.wasm_func.instruction(&Instruction::F32Mul);

                    // compute final k (now computes: 1.0 - result)
                    ctx.wasm_func.instruction(&Instruction::F32Sub);

                    // save k into temp local and test k < 0.0
                    ctx.wasm_func.instruction(&Instruction::LocalTee(temp_k));
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    ctx.wasm_func.instruction(&Instruction::F32Lt);

                    // if k < 0 -> return 0.0 for this component, else compute refracted component
                    ctx.wasm_func
                        .instruction(&Instruction::If(wasm_encoder::BlockType::Result(
                            ValType::F32,
                        )));
                    // true: total internal reflection -> zero component
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    ctx.wasm_func.instruction(&Instruction::Else);

                    // term1 = eta * I_comp
                    translate_expression_component(eta, eta_idx, ctx)?;
                    translate_expression_component(i, component_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Mul);

                    // term2 = eta * dot (recompute dot)
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    for j in 0..count {
                        translate_expression_component(n, j, ctx)?;
                        translate_expression_component(i, j, ctx)?;
                        ctx.wasm_func.instruction(&Instruction::F32Mul);
                        ctx.wasm_func.instruction(&Instruction::F32Add);
                    }
                    translate_expression_component(eta, eta_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Mul);

                    // add sqrt(k)
                    ctx.wasm_func.instruction(&Instruction::LocalGet(temp_k));
                    ctx.wasm_func.instruction(&Instruction::F32Sqrt);
                    ctx.wasm_func.instruction(&Instruction::F32Add);

                    // multiply by N_comp
                    translate_expression_component(n, component_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Mul);

                    // final: term1 - that
                    ctx.wasm_func.instruction(&Instruction::F32Sub);

                    ctx.wasm_func.instruction(&Instruction::End);
                }
                MathFunction::Reflect => {
                    // reflect(I, N) = I - 2 * dot(N, I) * N
                    let i = *arg;
                    let n = *arg1.as_ref().unwrap();
                    let arg_ty = ctx.typifier.get(i, &ctx.module.types);
                    let count = super::types::component_count(arg_ty, &ctx.module.types);

                    // compute dot = dot(N, I)
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    for j in 0..count {
                        translate_expression_component(n, j, ctx)?;
                        translate_expression_component(i, j, ctx)?;
                        ctx.wasm_func.instruction(&Instruction::F32Mul);
                        ctx.wasm_func.instruction(&Instruction::F32Add);
                    }
                    ctx.wasm_func
                        .instruction(&Instruction::LocalSet(ctx.swap_f32_local));

                    // I_comp - 2.0 * dot * N_comp
                    translate_expression_component(i, component_idx, ctx)?;
                    ctx.wasm_func
                        .instruction(&Instruction::LocalGet(ctx.swap_f32_local));
                    ctx.wasm_func.instruction(&Instruction::F32Const(2.0));
                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                    translate_expression_component(n, component_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                    ctx.wasm_func.instruction(&Instruction::F32Sub);
                }
                MathFunction::FaceForward => {
                    // faceforward(N, I, Nref) = (dot(Nref,I) < 0.0) ? N : -N
                    let n = *arg;
                    let i = *arg1.as_ref().unwrap();
                    let nref = *arg2.as_ref().unwrap();

                    // compute d = dot(nref, i)
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    let arg_ty = ctx.typifier.get(i, &ctx.module.types);
                    let count = super::types::component_count(arg_ty, &ctx.module.types);
                    for j in 0..count {
                        translate_expression_component(nref, j, ctx)?;
                        translate_expression_component(i, j, ctx)?;
                        ctx.wasm_func.instruction(&Instruction::F32Mul);
                        ctx.wasm_func.instruction(&Instruction::F32Add);
                    }
                    ctx.wasm_func
                        .instruction(&Instruction::LocalTee(ctx.swap_f32_local));
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    ctx.wasm_func.instruction(&Instruction::F32Lt);
                    ctx.wasm_func
                        .instruction(&Instruction::If(wasm_encoder::BlockType::Result(
                            ValType::F32,
                        )));
                    // true: return n_comp
                    translate_expression_component(n, component_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::Else);
                    // false: return -n_comp
                    translate_expression_component(n, component_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Neg);
                    ctx.wasm_func.instruction(&Instruction::End);
                }
                MathFunction::Distance => {
                    if component_idx == 0 {
                        let a = *arg;
                        let b = *arg1.as_ref().unwrap();
                        let arg_ty = ctx.typifier.get(a, &ctx.module.types);
                        let count = super::types::component_count(arg_ty, &ctx.module.types);
                        ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                        for j in 0..count {
                            translate_expression_component(a, j, ctx)?;
                            translate_expression_component(b, j, ctx)?;
                            ctx.wasm_func.instruction(&Instruction::F32Sub);
                            ctx.wasm_func.instruction(&Instruction::F32Mul);
                            ctx.wasm_func.instruction(&Instruction::F32Add);
                        }
                        ctx.wasm_func.instruction(&Instruction::F32Sqrt);
                    } else {
                        ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    }
                }
                MathFunction::Determinant => {
                    if component_idx == 0 {
                        let arg_ty = ctx.typifier.get(*arg, &ctx.module.types);
                        if let naga::TypeInner::Matrix { columns, rows, .. } = arg_ty {
                            match (*columns, *rows) {
                                (naga::VectorSize::Bi, naga::VectorSize::Bi) => {
                                    // mat2: a00=0 a10=1 a01=2 a11=3
                                    translate_expression_component(*arg, 0, ctx)?; // a00
                                    translate_expression_component(*arg, 3, ctx)?; // a11
                                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                                    translate_expression_component(*arg, 2, ctx)?; // a01
                                    translate_expression_component(*arg, 1, ctx)?; // a10
                                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                                    ctx.wasm_func.instruction(&Instruction::F32Sub);
                                    // stabilize the stack: save determinant into temp local and restore
                                    ctx.wasm_func
                                        .instruction(&Instruction::LocalSet(ctx.swap_f32_local));
                                    // restore value on stack
                                    ctx.wasm_func
                                        .instruction(&Instruction::LocalGet(ctx.swap_f32_local));
                                }
                                (naga::VectorSize::Tri, naga::VectorSize::Tri) => {
                                    // a00=0 a01=3 a02=6 a10=1 a11=4 a12=7 a20=2 a21=5 a22=8
                                    // Compute determinant using one temporary to avoid stack ordering errors
                                    let temp1 = ctx.swap_f32_local; // tmp for intermediate

                                    // tmp1 = a11*a22 - a12*a21
                                    translate_expression_component(*arg, 4, ctx)?; // a11
                                    translate_expression_component(*arg, 8, ctx)?; // a22
                                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                                    translate_expression_component(*arg, 7, ctx)?; // a12
                                    translate_expression_component(*arg, 5, ctx)?; // a21
                                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                                    ctx.wasm_func.instruction(&Instruction::F32Sub);
                                    ctx.wasm_func.instruction(&Instruction::LocalSet(temp1));

                                    // s = a00 * tmp1
                                    translate_expression_component(*arg, 0, ctx)?; // a00
                                    ctx.wasm_func.instruction(&Instruction::LocalGet(temp1));
                                    ctx.wasm_func.instruction(&Instruction::F32Mul);

                                    // - a01*(a10*a22 - a12*a20)
                                    translate_expression_component(*arg, 3, ctx)?; // a01
                                    translate_expression_component(*arg, 1, ctx)?; // a10
                                    translate_expression_component(*arg, 8, ctx)?; // a22
                                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                                    translate_expression_component(*arg, 7, ctx)?; // a12
                                    translate_expression_component(*arg, 2, ctx)?; // a20
                                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                                    ctx.wasm_func.instruction(&Instruction::F32Sub);
                                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                                    ctx.wasm_func.instruction(&Instruction::F32Neg);
                                    ctx.wasm_func.instruction(&Instruction::F32Add);

                                    // + a02 * (a10*a21 - a11*a20)
                                    translate_expression_component(*arg, 6, ctx)?; // a02
                                    translate_expression_component(*arg, 1, ctx)?; // a10
                                    translate_expression_component(*arg, 5, ctx)?; // a21
                                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                                    translate_expression_component(*arg, 4, ctx)?; // a11
                                    translate_expression_component(*arg, 2, ctx)?; // a20
                                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                                    ctx.wasm_func.instruction(&Instruction::F32Sub);
                                    ctx.wasm_func.instruction(&Instruction::F32Mul);
                                    ctx.wasm_func.instruction(&Instruction::F32Add);
                                    // stabilize the stack: save determinant into temp local and restore
                                    ctx.wasm_func
                                        .instruction(&Instruction::LocalSet(ctx.swap_f32_local));
                                    // restore value on stack
                                    ctx.wasm_func
                                        .instruction(&Instruction::LocalGet(ctx.swap_f32_local));
                                }
                                _ => {
                                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                                }
                            }
                        } else {
                            ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                        }
                    } else {
                        ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    }
                }
                MathFunction::Inverse => {
                    // Optimized matrix inversion using host imports to avoid huge instruction bloat
                    let arg_ty = ctx.typifier.get(*arg, &ctx.module.types);
                    if let naga::TypeInner::Matrix { columns, rows, .. } = arg_ty {
                        let count = (*columns as u32) * (*rows as u32);
                        let helper_idx = match (*columns, *rows) {
                            (naga::VectorSize::Bi, naga::VectorSize::Bi) => ctx.inverse_mat2_idx,
                            (naga::VectorSize::Tri, naga::VectorSize::Tri) => ctx.inverse_mat3_idx,
                            _ => None,
                        };

                        if let Some(func_idx) = helper_idx {
                            let frame_temp = ctx.frame_temp_idx.expect("Frame temp local missing");
                            // Space for both input and output matrices
                            let frame_size = count * 4 * 2;

                            // 1. Allocate frame storage
                            ctx.wasm_func.instruction(&Instruction::GlobalGet(
                                output_layout::FRAME_SP_GLOBAL,
                            ));
                            ctx.wasm_func
                                .instruction(&Instruction::LocalTee(frame_temp));
                            ctx.wasm_func
                                .instruction(&Instruction::I32Const(frame_size as i32));
                            ctx.wasm_func.instruction(&Instruction::I32Add);
                            ctx.wasm_func.instruction(&Instruction::GlobalSet(
                                output_layout::FRAME_SP_GLOBAL,
                            ));

                            // 2. Store input matrix to frame (at frame_temp)
                            for i in 0..count {
                                ctx.wasm_func
                                    .instruction(&Instruction::LocalGet(frame_temp));
                                translate_expression_component(*arg, i, ctx)?;
                                ctx.wasm_func.instruction(&Instruction::F32Store(
                                    wasm_encoder::MemArg {
                                        offset: (i * 4) as u64,
                                        align: 2,
                                        memory_index: 0,
                                    },
                                ));
                            }

                            // 3. Call host helper: helper(in_ptr, out_ptr)
                            ctx.wasm_func
                                .instruction(&Instruction::LocalGet(frame_temp)); // in_ptr
                            ctx.wasm_func
                                .instruction(&Instruction::LocalGet(frame_temp));
                            ctx.wasm_func
                                .instruction(&Instruction::I32Const((count * 4) as i32));
                            ctx.wasm_func.instruction(&Instruction::I32Add); // out_ptr
                            ctx.wasm_func.instruction(&Instruction::Call(func_idx));

                            // 4. Load the requested component from the output matrix on the frame
                            ctx.wasm_func
                                .instruction(&Instruction::LocalGet(frame_temp));
                            ctx.wasm_func.instruction(&Instruction::F32Load(
                                wasm_encoder::MemArg {
                                    offset: (count * 4 + component_idx * 4) as u64,
                                    align: 2,
                                    memory_index: 0,
                                },
                            ));

                            // 5. Restore frame pointer
                            ctx.wasm_func
                                .instruction(&Instruction::LocalGet(frame_temp));
                            ctx.wasm_func.instruction(&Instruction::GlobalSet(
                                output_layout::FRAME_SP_GLOBAL,
                            ));
                        } else {
                            ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                        }
                    } else {
                        ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    }
                }
                MathFunction::Modf => {
                    // modf(x, out ip): returns fractional part, stores integer part (trunc toward 0) into ip
                    let x = *arg;
                    let out_ptr = arg1.expect("Modf needs 2 arguments");

                    // ip_f = trunc(x)
                    translate_expression_component(x, component_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::F32Trunc);
                    // store ip_f in temp f32 local
                    ctx.wasm_func
                        .instruction(&Instruction::LocalSet(ctx.swap_f32_local));

                    // store integer part into *out_ptr (as i32)
                    translate_expression(out_ptr, ctx)?; // push pointer
                    ctx.wasm_func
                        .instruction(&Instruction::LocalGet(ctx.swap_f32_local));
                    ctx.wasm_func.instruction(&Instruction::I32TruncSatF32S);
                    ctx.wasm_func
                        .instruction(&Instruction::I32Store(wasm_encoder::MemArg {
                            offset: (component_idx * 4) as u64,
                            align: 2,
                            memory_index: 0,
                        }));

                    // fractional = x - ip_f
                    translate_expression_component(x, component_idx, ctx)?;
                    ctx.wasm_func
                        .instruction(&Instruction::LocalGet(ctx.swap_f32_local));
                    ctx.wasm_func.instruction(&Instruction::F32Sub);
                }
                MathFunction::Frexp => {
                    // frexp(x, out exp): returns mantissa, stores exponent as integer out
                    let x = *arg;
                    let out_ptr = arg1.expect("Frexp needs 2 arguments");

                    // scratch locals
                    let temp_orig = ctx.swap_f32_local; // store original x
                    let temp_e = ctx.swap_f32_local_2.unwrap_or(ctx.swap_f32_local); // secondary temp if available

                    // save original x
                    translate_expression_component(x, component_idx, ctx)?;
                    ctx.wasm_func.instruction(&Instruction::LocalSet(temp_orig));

                    // abs_x = abs(original)
                    ctx.wasm_func.instruction(&Instruction::LocalGet(temp_orig));
                    ctx.wasm_func.instruction(&Instruction::F32Abs);
                    ctx.wasm_func.instruction(&Instruction::LocalSet(temp_e)); // reuse temp_e for abs check temporarily

                    // if abs_x == 0.0 -> set exp = 0, mantissa = 0.0
                    ctx.wasm_func.instruction(&Instruction::LocalGet(temp_e));
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    ctx.wasm_func.instruction(&Instruction::F32Eq);

                    ctx.wasm_func
                        .instruction(&Instruction::If(wasm_encoder::BlockType::Result(
                            ValType::F32,
                        )));
                    // true: zero input -> write 0 to exp and return 0.0 mantissa
                    // store 0 to *out_ptr
                    translate_expression(out_ptr, ctx)?; // push pointer
                    ctx.wasm_func.instruction(&Instruction::I32Const(0));
                    ctx.wasm_func
                        .instruction(&Instruction::I32Store(wasm_encoder::MemArg {
                            offset: (component_idx * 4) as u64,
                            align: 2,
                            memory_index: 0,
                        }));
                    ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
                    ctx.wasm_func.instruction(&Instruction::Else);

                    // else: compute e = floor(log2(abs(x))) + 1
                    // restore abs_x into stack
                    ctx.wasm_func.instruction(&Instruction::LocalGet(temp_e)); // abs_x
                                                                               // call gl_log2
                    let log2_idx = *ctx
                        .math_import_map
                        .get(&MathFunction::Log2)
                        .expect("Math import missing");
                    ctx.wasm_func.instruction(&Instruction::Call(log2_idx));
                    ctx.wasm_func.instruction(&Instruction::F32Floor);
                    ctx.wasm_func.instruction(&Instruction::F32Const(1.0));
                    ctx.wasm_func.instruction(&Instruction::F32Add);
                    // save e_f as float
                    ctx.wasm_func.instruction(&Instruction::LocalSet(temp_e));

                    // store integer exponent into *out_ptr
                    translate_expression(out_ptr, ctx)?; // push pointer
                    ctx.wasm_func.instruction(&Instruction::LocalGet(temp_e));
                    ctx.wasm_func.instruction(&Instruction::I32TruncSatF32S);
                    ctx.wasm_func
                        .instruction(&Instruction::I32Store(wasm_encoder::MemArg {
                            offset: (component_idx * 4) as u64,
                            align: 2,
                            memory_index: 0,
                        }));

                    // mantissa = original_x * exp2(-e_f)
                    ctx.wasm_func.instruction(&Instruction::LocalGet(temp_orig));
                    ctx.wasm_func.instruction(&Instruction::LocalGet(temp_e));
                    ctx.wasm_func.instruction(&Instruction::F32Neg);
                    let exp2_idx = *ctx
                        .math_import_map
                        .get(&MathFunction::Exp2)
                        .expect("Math import missing");
                    ctx.wasm_func.instruction(&Instruction::Call(exp2_idx));
                    ctx.wasm_func.instruction(&Instruction::F32Mul);

                    ctx.wasm_func.instruction(&Instruction::End);
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
    if matches!(
        ty,
        naga::TypeInner::Pointer { .. } | naga::TypeInner::ValuePointer { .. }
    ) {
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

                    if offset > 0 {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Const(offset as i32));
                        ctx.wasm_func.instruction(&Instruction::I32Add);
                    }

                    // WebGPU indirection
                    if ctx.uniform_locations.is_empty()
                        && base_ptr_idx == output_layout::UNIFORM_PTR_GLOBAL
                    {
                        ctx.wasm_func
                            .instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                                offset: 0,
                                align: 2,
                                memory_index: 0,
                            }));
                    }
                } else {
                    // Fallback for unknown globals
                    let name: Option<&str> = var.name.as_deref();
                    match name {
                        Some("gl_Position") | Some("gl_Position_1") => {
                            ctx.wasm_func.instruction(&Instruction::GlobalGet(
                                output_layout::VARYING_PTR_GLOBAL,
                            ));
                        }
                        Some(val)
                            if val.starts_with("output")
                                || val == "outColor"
                                || val == "fragColor"
                                || val == "gl_FragColor" =>
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
            Expression::Access { base, index } => {
                translate_expression(*base, ctx)?;
                let base_ty = ctx.typifier.get(*base, &ctx.module.types);
                let element_size = match base_ty {
                    TypeInner::Pointer {
                        base: pointed_ty, ..
                    } => {
                        let element_inner = &ctx.module.types[*pointed_ty].inner;
                        match element_inner {
                            naga::TypeInner::Array { stride, .. } => *stride,
                            naga::TypeInner::Vector { .. } => 4,
                            naga::TypeInner::Matrix { rows, .. } => (*rows as u32) * 4,
                            _ => super::types::type_size(element_inner).unwrap_or(4),
                        }
                    }
                    TypeInner::ValuePointer { .. } => 4,
                    _ => 4,
                };
                translate_expression(*index, ctx)?;
                if element_size != 1 {
                    ctx.wasm_func
                        .instruction(&Instruction::I32Const(element_size as i32));
                    ctx.wasm_func.instruction(&Instruction::I32Mul);
                }
                ctx.wasm_func.instruction(&Instruction::I32Add);
            }
            Expression::AccessIndex { base, index } => {
                translate_expression(*base, ctx)?;
                let base_ty = ctx.typifier.get(*base, &ctx.module.types);
                let element_size = match base_ty {
                    TypeInner::Pointer {
                        base: pointed_ty, ..
                    } => {
                        let element_inner = &ctx.module.types[*pointed_ty].inner;
                        match element_inner {
                            naga::TypeInner::Array { stride, .. } => *stride,
                            naga::TypeInner::Vector { .. } => 4,
                            naga::TypeInner::Matrix { rows, .. } => (*rows as u32) * 4,
                            _ => super::types::type_size(element_inner).unwrap_or(4),
                        }
                    }
                    TypeInner::ValuePointer { .. } => 4,
                    _ => 4,
                };

                if *index > 0 {
                    ctx.wasm_func
                        .instruction(&Instruction::I32Const((*index * element_size) as i32));
                    ctx.wasm_func.instruction(&Instruction::I32Add);
                }
            }
            Expression::FunctionArgument(idx) => {
                let base_idx = ctx.argument_local_offsets.get(idx).cloned().unwrap_or(*idx);
                ctx.wasm_func.instruction(&Instruction::LocalGet(base_idx));
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
