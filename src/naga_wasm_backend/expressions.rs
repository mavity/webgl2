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
    global_offsets: &HashMap<naga::Handle<naga::GlobalVariable>, u32>,
    stage: naga::ShaderStage,
    typifier: &Typifier,
    naga_function_map: &HashMap<naga::Handle<naga::Function>, u32>,
    is_entry_point: bool,
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
                translate_expression(c.init, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
            } else {
                wasm_func.instruction(&Instruction::F32Const(0.0));
            }
        }
        Expression::Compose { components, .. } => {
            if (component_idx as usize) < components.len() {
                translate_expression_component(components[component_idx as usize], 0, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
            } else {
                wasm_func.instruction(&Instruction::F32Const(0.0));
            }
        }
        Expression::Binary { op, left, right } => {
            translate_expression_component(*left, component_idx, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
            translate_expression_component(*right, component_idx, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
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
        Expression::FunctionArgument(idx) => {
            if is_entry_point {
                // Map Naga argument to attribute data
                // For now, we assume each argument is a vec4 (16 bytes)
                wasm_func.instruction(&Instruction::GlobalGet(0)); // attr_ptr
                wasm_func.instruction(&Instruction::I32Const((*idx * 16 + component_idx * 4) as i32));
                wasm_func.instruction(&Instruction::I32Add);
                wasm_func.instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));
            } else {
                // If it's an internal function, we use LocalGet
                wasm_func.instruction(&Instruction::LocalGet(*idx));
            }
        }
        Expression::GlobalVariable(handle) => {
            // This should probably not be called directly for a value, but if it is, we load it
            translate_expression(expr_handle, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
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
        Expression::Load { pointer } => {
            translate_expression(*pointer, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
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
            // If base is a pointer, we add the index and then load the component
            // If base is a value, we add the index and then load the component?
            // Actually, Naga's AccessIndex on a pointer produces a pointer.
            // AccessIndex on a value produces a value.
            
            // For now, let's assume it's a pointer
            translate_expression(*base, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
            // Add the index offset
            wasm_func.instruction(&Instruction::I32Const((*index * 4) as i32));
            wasm_func.instruction(&Instruction::I32Add);
            
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
    global_offsets: &HashMap<naga::Handle<naga::GlobalVariable>, u32>,
    stage: naga::ShaderStage,
    typifier: &Typifier,
    naga_function_map: &HashMap<naga::Handle<naga::Function>, u32>,
    is_entry_point: bool,
) -> Result<(), BackendError> {
    let expr = &func.expressions[expr_handle];
    match expr {
        Expression::Literal(literal) => {
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
        }
        Expression::Constant(c_handle) => {
            let c = &module.constants[*c_handle];
            // For now, we only support simple literals in constants
            let init_expr = &module.global_expressions[c.init];
            match init_expr {
                Expression::Literal(literal) => {
                    match literal {
                        Literal::F32(f) => {
                            wasm_func.instruction(&Instruction::F32Const(*f));
                        }
                        Literal::I32(i) => {
                            wasm_func.instruction(&Instruction::I32Const(*i));
                        }
                        _ => {
                            wasm_func.instruction(&Instruction::F32Const(0.0));
                        }
                    }
                }
                _ => {
                    wasm_func.instruction(&Instruction::F32Const(0.0));
                }
            }
        }
        Expression::Compose { components, .. } => {
            for &comp in components {
                translate_expression(comp, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
            }
        }
        Expression::Binary { op, left, right } => {
            translate_expression(*left, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
            translate_expression(*right, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
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
        Expression::LocalVariable(_handle) => {
            wasm_func.instruction(&Instruction::F32Const(0.0));
        }
        Expression::FunctionArgument(idx) => {
            if is_entry_point {
                // Map Naga argument to attribute data
                // For now, we assume each argument is a vec4 (16 bytes)
                // and we load the first component.
                wasm_func.instruction(&Instruction::GlobalGet(0)); // attr_ptr
                wasm_func.instruction(&Instruction::I32Const((*idx * 16) as i32));
                wasm_func.instruction(&Instruction::I32Add);
                wasm_func.instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));
            } else {
                // If it's an internal function, we use LocalGet
                wasm_func.instruction(&Instruction::LocalGet(*idx));
            }
        }
        Expression::GlobalVariable(handle) => {
            let var = &module.global_variables[*handle];
            let is_position = if let Some(name) = &var.name {
                name == "gl_Position" || name == "gl_Position_1"
            } else {
                false
            };

            let base_ptr_idx = if is_position {
                2 // varying_ptr
            } else {
                match var.space {
                    naga::AddressSpace::Uniform => 1,
                    naga::AddressSpace::Private | naga::AddressSpace::Function => {
                        // Heuristic: if it's a vertex shader and it's a global variable with a name,
                        // it's probably an attribute.
                        if stage == naga::ShaderStage::Vertex && var.name.is_some() {
                            0 // attr_ptr
                        } else {
                            3 // private_ptr
                        }
                    }
                    _ => 2, // Default to varying_ptr for Out/etc.
                }
            };
            wasm_func.instruction(&Instruction::GlobalGet(base_ptr_idx));
            if let Some(&offset) = global_offsets.get(handle) {
                wasm_func.instruction(&Instruction::I32Const(offset as i32));
                wasm_func.instruction(&Instruction::I32Add);
            }
        }
        Expression::AccessIndex { base, index } => {
            translate_expression(*base, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
            // Assume each index is 4 bytes (float)
            wasm_func.instruction(&Instruction::I32Const((*index * 4) as i32));
            wasm_func.instruction(&Instruction::I32Add);
        }
        Expression::Load { pointer } => {
            translate_expression(*pointer, func, module, wasm_func, global_offsets, stage, typifier, naga_function_map, is_entry_point)?;
            wasm_func.instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                offset: 0,
                align: 2,
                memory_index: 0,
            }));
        }
        _ => {
            wasm_func.instruction(&Instruction::F32Const(0.0));
        }
    }
    Ok(())
}
