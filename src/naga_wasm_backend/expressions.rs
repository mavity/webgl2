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
            image, coordinate, ..
        } => {
            // Call module-local sampling helper
            // Signature: (texture_ptr: i32, unit: i32, u: f32, v: f32) -> (f32, f32, f32, f32)

            if let Some(tex_fetch_idx) = ctx.webgl_texture_sample_idx {
                // 1. Push texture_ptr (global)
                ctx.wasm_func
                    .instruction(&Instruction::GlobalGet(output_layout::TEXTURE_PTR_GLOBAL));

                // 2. Get texture unit index from the image expression
                translate_expression_component(*image, 0, ctx)?;

                // If the image expression is a GlobalVariable, load the value (texture unit index)
                if let Expression::GlobalVariable(_) = ctx.func.expressions[*image] {
                    ctx.wasm_func
                        .instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 2,
                            memory_index: 0,
                        }));
                }

                // 3. Push U coordinate
                translate_expression_component(*coordinate, 0, ctx)?;

                // 4. Push V coordinate
                translate_expression_component(*coordinate, 1, ctx)?;

                // 5. Call host texel fetch import -> returns (f32, f32, f32, f32) on WASM stack
                ctx.wasm_func.instruction(&Instruction::Call(tex_fetch_idx));

                // 6. Store all 4 results in explicit sampling locals
                // The multivalue return produces 4 f32 values on the stack: [r, g, b, a]
                // We need to store them in reverse order due to stack semantics (last value on top).
                let sample_base = ctx
                    .sample_f32_locals
                    .expect("Sampling locals not allocated for function with ImageSample?");

                ctx.wasm_func
                    .instruction(&Instruction::LocalSet(sample_base + 3)); // a (top of stack)
                ctx.wasm_func
                    .instruction(&Instruction::LocalSet(sample_base + 2)); // b
                ctx.wasm_func
                    .instruction(&Instruction::LocalSet(sample_base + 1)); // g
                ctx.wasm_func
                    .instruction(&Instruction::LocalSet(sample_base)); // r

                // 7. Load the requested component
                ctx.wasm_func
                    .instruction(&Instruction::LocalGet(sample_base + component_idx));
            } else {
                // Fallback: return black if helper not emitted
                ctx.wasm_func.instruction(&Instruction::F32Const(0.0));
            }
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
