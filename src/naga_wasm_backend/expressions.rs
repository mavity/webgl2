//! Expression translation from Naga IR to WASM
//!
//! Phase 0: Placeholder for future expression handling

use super::BackendError;

use wasm_encoder::{Instruction, Function};
use naga::{Expression, BinaryOperator, ScalarKind, Literal};
use std::collections::HashMap;

/// Translate a Naga expression to WASM instructions
pub fn translate_expression(
    expr_handle: naga::Handle<Expression>,
    func: &naga::Function,
    module: &naga::Module,
    wasm_func: &mut Function,
    global_offsets: &HashMap<naga::Handle<naga::GlobalVariable>, u32>,
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
                translate_expression(comp, func, module, wasm_func, global_offsets)?;
            }
        }
        Expression::Binary { op, left, right } => {
            translate_expression(*left, func, module, wasm_func, global_offsets)?;
            translate_expression(*right, func, module, wasm_func, global_offsets)?;
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
        Expression::GlobalVariable(handle) => {
            if let Some(&offset) = global_offsets.get(handle) {
                wasm_func.instruction(&Instruction::I32Const(offset as i32));
            } else {
                wasm_func.instruction(&Instruction::I32Const(0));
            }
        }
        _ => {
            wasm_func.instruction(&Instruction::F32Const(0.0));
        }
    }
    Ok(())
}
