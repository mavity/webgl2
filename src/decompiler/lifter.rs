//! The Lifter: Converts stack-based WASM instructions into AST expressions and statements.
//!
//! This module implements the core decompilation logic using a symbolic stack
//! to track expressions and a control stack to handle structured control flow.

use super::ast::{BinOp, Expr, ScalarType, Stmt, UnaryOp};
use wasmparser::{Operator, ValType};

/// Frame types for control flow tracking.
#[derive(Debug, Clone)]
enum ControlFrame {
    Block {
        _stack_height: usize,
        body: Vec<Stmt>,
    },
    Loop {
        _stack_height: usize,
        body: Vec<Stmt>,
    },
    If {
        _stack_height: usize,
        condition: Expr,
        then_body: Vec<Stmt>,
    },
    Else {
        _stack_height: usize,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    },
}

/// The Lifter transforms WASM bytecode into our AST.
pub struct Lifter {
    /// The symbolic value stack
    value_stack: Vec<Expr>,
    /// The control flow stack
    control_stack: Vec<ControlFrame>,
    /// Statements for the current scope
    current_body: Vec<Stmt>,
    /// Parameter count for the function
    _param_count: u32,
    /// Local types (including parameters)
    _local_types: Vec<ScalarType>,
}

impl Lifter {
    /// Create a new lifter for a function.
    pub fn new(param_count: u32, local_types: Vec<ScalarType>) -> Self {
        Self {
            value_stack: Vec::new(),
            control_stack: Vec::new(),
            current_body: Vec::new(),
            _param_count: param_count,
            _local_types: local_types,
        }
    }

    /// Push an expression onto the symbolic stack.
    fn push(&mut self, expr: Expr) {
        self.value_stack.push(expr);
    }

    /// Pop an expression from the symbolic stack.
    fn pop(&mut self) -> Expr {
        self.value_stack
            .pop()
            .unwrap_or(Expr::Unknown("stack underflow".to_string()))
    }

    /// Emit a statement to the current scope.
    fn emit(&mut self, stmt: Stmt) {
        if let Some(frame) = self.control_stack.last_mut() {
            match frame {
                ControlFrame::Block { body, .. } => body.push(stmt),
                ControlFrame::Loop { body, .. } => body.push(stmt),
                ControlFrame::If { then_body, .. } => then_body.push(stmt),
                ControlFrame::Else { else_body, .. } => else_body.push(stmt),
            }
        } else {
            self.current_body.push(stmt);
        }
    }

    /// Get the type of a local variable.
    #[allow(dead_code)]
    fn local_type(&self, idx: u32) -> ScalarType {
        self._local_types
            .get(idx as usize)
            .copied()
            .unwrap_or(ScalarType::Int)
    }

    /// Process a single WASM operator.
    pub fn process_operator(&mut self, op: &Operator) {
        match op {
            // Constants
            Operator::I32Const { value } => self.push(Expr::ConstI32(*value)),
            Operator::I64Const { value } => self.push(Expr::ConstI64(*value)),
            Operator::F32Const { value } => self.push(Expr::ConstF32(f32::from_bits(value.bits()))),
            Operator::F64Const { value } => self.push(Expr::ConstF64(f64::from_bits(value.bits()))),

            // Local variables
            Operator::LocalGet { local_index } => self.push(Expr::LocalGet(*local_index)),
            Operator::LocalSet { local_index } => {
                let value = self.pop();
                self.emit(Stmt::LocalSet {
                    local_idx: *local_index,
                    value,
                });
            }
            Operator::LocalTee { local_index } => {
                let value = self.pop();
                self.emit(Stmt::LocalSet {
                    local_idx: *local_index,
                    value: value.clone(),
                });
                self.push(value);
            }

            // Global variables
            Operator::GlobalGet { global_index } => self.push(Expr::GlobalGet(*global_index)),
            Operator::GlobalSet { global_index } => {
                let value = self.pop();
                self.emit(Stmt::GlobalSet {
                    global_idx: *global_index,
                    value,
                });
            }

            // i32 Binary operations
            Operator::I32Add => self.binary_op(BinOp::Add),
            Operator::I32Sub => self.binary_op(BinOp::Sub),
            Operator::I32Mul => self.binary_op(BinOp::Mul),
            Operator::I32DivS => self.binary_op(BinOp::Div),
            Operator::I32DivU => self.binary_op(BinOp::Div),
            Operator::I32RemS => self.binary_op(BinOp::Rem),
            Operator::I32RemU => self.binary_op(BinOp::Rem),
            Operator::I32And => self.binary_op(BinOp::And),
            Operator::I32Or => self.binary_op(BinOp::Or),
            Operator::I32Xor => self.binary_op(BinOp::Xor),
            Operator::I32Shl => self.binary_op(BinOp::Shl),
            Operator::I32ShrS => self.binary_op(BinOp::ShrS),
            Operator::I32ShrU => self.binary_op(BinOp::ShrU),

            // i32 Comparison
            Operator::I32Eq => self.binary_op(BinOp::Eq),
            Operator::I32Ne => self.binary_op(BinOp::Ne),
            Operator::I32LtS => self.binary_op(BinOp::LtS),
            Operator::I32LtU => self.binary_op(BinOp::LtU),
            Operator::I32LeS => self.binary_op(BinOp::LeS),
            Operator::I32LeU => self.binary_op(BinOp::LeU),
            Operator::I32GtS => self.binary_op(BinOp::GtS),
            Operator::I32GtU => self.binary_op(BinOp::GtU),
            Operator::I32GeS => self.binary_op(BinOp::GeS),
            Operator::I32GeU => self.binary_op(BinOp::GeU),

            // i32 Unary
            Operator::I32Eqz => self.unary_op(UnaryOp::Eqz),

            // f32 Binary operations
            Operator::F32Add => self.binary_op(BinOp::Add),
            Operator::F32Sub => self.binary_op(BinOp::Sub),
            Operator::F32Mul => self.binary_op(BinOp::Mul),
            Operator::F32Div => self.binary_op(BinOp::Div),

            // f32 Comparison
            Operator::F32Eq => self.binary_op(BinOp::Eq),
            Operator::F32Ne => self.binary_op(BinOp::Ne),
            Operator::F32Lt => self.binary_op(BinOp::LtS),
            Operator::F32Le => self.binary_op(BinOp::LeS),
            Operator::F32Gt => self.binary_op(BinOp::GtS),
            Operator::F32Ge => self.binary_op(BinOp::GeS),

            // f32 Unary
            Operator::F32Neg => self.unary_op(UnaryOp::Neg),
            Operator::F32Abs => self.unary_op(UnaryOp::Abs),
            Operator::F32Ceil => self.unary_op(UnaryOp::Ceil),
            Operator::F32Floor => self.unary_op(UnaryOp::Floor),
            Operator::F32Trunc => self.unary_op(UnaryOp::Trunc),
            Operator::F32Nearest => self.unary_op(UnaryOp::Nearest),
            Operator::F32Sqrt => self.unary_op(UnaryOp::Sqrt),

            // f64 operations (mapped to float in GLSL)
            Operator::F64Add => self.binary_op(BinOp::Add),
            Operator::F64Sub => self.binary_op(BinOp::Sub),
            Operator::F64Mul => self.binary_op(BinOp::Mul),
            Operator::F64Div => self.binary_op(BinOp::Div),
            Operator::F64Neg => self.unary_op(UnaryOp::Neg),
            Operator::F64Abs => self.unary_op(UnaryOp::Abs),
            Operator::F64Ceil => self.unary_op(UnaryOp::Ceil),
            Operator::F64Floor => self.unary_op(UnaryOp::Floor),
            Operator::F64Sqrt => self.unary_op(UnaryOp::Sqrt),

            // Conversions
            Operator::I32TruncF32S | Operator::I32TruncF32U => {
                let operand = self.pop();
                self.push(Expr::Convert {
                    from: ScalarType::Float,
                    to: ScalarType::Int,
                    operand: Box::new(operand),
                });
            }
            Operator::F32ConvertI32S | Operator::F32ConvertI32U => {
                let operand = self.pop();
                self.push(Expr::Convert {
                    from: ScalarType::Int,
                    to: ScalarType::Float,
                    operand: Box::new(operand),
                });
            }
            Operator::I32ReinterpretF32 => {
                // Bit-cast, keep as-is for now
                let operand = self.pop();
                self.push(Expr::Convert {
                    from: ScalarType::Float,
                    to: ScalarType::Int,
                    operand: Box::new(operand),
                });
            }
            Operator::F32ReinterpretI32 => {
                let operand = self.pop();
                self.push(Expr::Convert {
                    from: ScalarType::Int,
                    to: ScalarType::Float,
                    operand: Box::new(operand),
                });
            }

            // Memory operations
            Operator::I32Load { memarg } => {
                let addr = self.pop();
                self.push(Expr::MemoryLoad {
                    ty: ScalarType::Int,
                    offset: memarg.offset as u32,
                    addr: Box::new(addr),
                });
            }
            Operator::F32Load { memarg } => {
                let addr = self.pop();
                self.push(Expr::MemoryLoad {
                    ty: ScalarType::Float,
                    offset: memarg.offset as u32,
                    addr: Box::new(addr),
                });
            }
            Operator::I32Store { memarg } => {
                let value = self.pop();
                let addr = self.pop();
                self.emit(Stmt::MemoryStore {
                    ty: ScalarType::Int,
                    offset: memarg.offset as u32,
                    addr,
                    value,
                });
            }
            Operator::F32Store { memarg } => {
                let value = self.pop();
                let addr = self.pop();
                self.emit(Stmt::MemoryStore {
                    ty: ScalarType::Float,
                    offset: memarg.offset as u32,
                    addr,
                    value,
                });
            }

            // Control flow
            Operator::Block { blockty: _ } => {
                self.control_stack.push(ControlFrame::Block {
                    _stack_height: self.value_stack.len(),
                    body: Vec::new(),
                });
            }
            Operator::Loop { blockty: _ } => {
                self.control_stack.push(ControlFrame::Loop {
                    _stack_height: self.value_stack.len(),
                    body: Vec::new(),
                });
            }
            Operator::If { blockty: _ } => {
                let condition = self.pop();
                self.control_stack.push(ControlFrame::If {
                    _stack_height: self.value_stack.len(),
                    condition,
                    then_body: Vec::new(),
                });
            }
            Operator::Else => {
                if let Some(ControlFrame::If {
                    _stack_height,
                    condition: _,
                    then_body,
                }) = self.control_stack.pop()
                {
                    // Get condition from the if frame by peeking before pop
                    // Since we already popped, we need to reconstruct
                    self.control_stack.push(ControlFrame::Else {
                        _stack_height,
                        then_body,
                        else_body: Vec::new(),
                    });
                }
            }
            Operator::End => {
                if let Some(frame) = self.control_stack.pop() {
                    let stmt = match frame {
                        ControlFrame::Block { body, .. } => Stmt::Block { body },
                        ControlFrame::Loop { body, .. } => Stmt::Loop { body },
                        ControlFrame::If {
                            condition,
                            then_body,
                            ..
                        } => Stmt::If {
                            condition,
                            then_body,
                            else_body: None,
                        },
                        ControlFrame::Else {
                            then_body,
                            else_body,
                            ..
                        } => {
                            // We need to get the condition from somewhere
                            // For now, use a placeholder
                            Stmt::If {
                                condition: Expr::Unknown("if condition".to_string()),
                                then_body,
                                else_body: Some(else_body),
                            }
                        }
                    };
                    self.emit(stmt);
                }
            }
            Operator::Br { relative_depth } => {
                // Determine if this is a break or continue based on target frame
                let depth = *relative_depth;
                let is_loop = self
                    .control_stack
                    .iter()
                    .rev()
                    .nth(depth as usize)
                    .map(|f| matches!(f, ControlFrame::Loop { .. }))
                    .unwrap_or(false);

                if is_loop {
                    self.emit(Stmt::Continue { depth });
                } else {
                    self.emit(Stmt::Break { depth });
                }
            }
            Operator::BrIf { relative_depth } => {
                let condition = self.pop();
                let depth = *relative_depth;
                let is_loop = self
                    .control_stack
                    .iter()
                    .rev()
                    .nth(depth as usize)
                    .map(|f| matches!(f, ControlFrame::Loop { .. }))
                    .unwrap_or(false);

                let branch_stmt = if is_loop {
                    Stmt::Continue { depth }
                } else {
                    Stmt::Break { depth }
                };

                self.emit(Stmt::If {
                    condition,
                    then_body: vec![branch_stmt],
                    else_body: None,
                });
            }

            Operator::Return => {
                let value = if !self.value_stack.is_empty() {
                    Some(self.pop())
                } else {
                    None
                };
                self.emit(Stmt::Return { value });
            }

            Operator::Select => {
                let condition = self.pop();
                let false_val = self.pop();
                let true_val = self.pop();
                self.push(Expr::Select {
                    condition: Box::new(condition),
                    true_val: Box::new(true_val),
                    false_val: Box::new(false_val),
                });
            }

            Operator::Drop => {
                self.pop();
                self.emit(Stmt::Drop);
            }

            Operator::Call { function_index } => {
                // For now, we don't know the arity, so we'll handle this minimally
                self.push(Expr::Call {
                    func_idx: *function_index,
                    args: Vec::new(),
                });
            }

            Operator::Nop => {}
            Operator::Unreachable => {
                self.emit(Stmt::Unknown("unreachable".to_string()));
            }

            // SIMD / v128 operations (Phase 2: map to vec4)
            Operator::V128Const { value } => {
                // Interpret as 4 floats
                let bytes = value.bytes();
                let f0 = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                let f1 = f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
                let f2 = f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
                let f3 = f32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
                self.push(Expr::VecConstruct {
                    components: vec![
                        Expr::ConstF32(f0),
                        Expr::ConstF32(f1),
                        Expr::ConstF32(f2),
                        Expr::ConstF32(f3),
                    ],
                });
            }
            Operator::F32x4Add => self.vec_binary_op(BinOp::Add),
            Operator::F32x4Sub => self.vec_binary_op(BinOp::Sub),
            Operator::F32x4Mul => self.vec_binary_op(BinOp::Mul),
            Operator::F32x4Div => self.vec_binary_op(BinOp::Div),

            // Default: unknown instruction
            _ => {
                self.push(Expr::Unknown(format!("{:?}", op)));
            }
        }
    }

    /// Helper for binary operations.
    fn binary_op(&mut self, op: BinOp) {
        let right = self.pop();
        let left = self.pop();
        self.push(Expr::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        });
    }

    /// Helper for unary operations.
    fn unary_op(&mut self, op: UnaryOp) {
        let operand = self.pop();
        self.push(Expr::UnaryOp {
            op,
            operand: Box::new(operand),
        });
    }

    /// Helper for vector binary operations.
    fn vec_binary_op(&mut self, op: BinOp) {
        let right = self.pop();
        let left = self.pop();
        self.push(Expr::VecBinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        });
    }

    /// Finish lifting and return the function body.
    pub fn finish(mut self, return_type: Option<ScalarType>) -> Vec<Stmt> {
        // If there's a value left on the stack and the function returns something,
        // add an implicit return
        if !self.value_stack.is_empty() && return_type.is_some() {
            let value = self.pop();
            self.current_body.push(Stmt::Return { value: Some(value) });
        }
        self.current_body
    }
}

/// Convert wasmparser ValType to our ScalarType.
pub fn valtype_to_scalar(ty: ValType) -> ScalarType {
    match ty {
        ValType::I32 => ScalarType::Int,
        ValType::I64 => ScalarType::Long,
        ValType::F32 => ScalarType::Float,
        ValType::F64 => ScalarType::Double,
        _ => ScalarType::Int, // Default for unsupported types
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifter_const() {
        let mut lifter = Lifter::new(0, vec![]);
        lifter.process_operator(&Operator::I32Const { value: 42 });
        assert_eq!(lifter.value_stack.len(), 1);
        match &lifter.value_stack[0] {
            Expr::ConstI32(v) => assert_eq!(*v, 42),
            _ => panic!("Expected ConstI32"),
        }
    }

    #[test]
    fn test_lifter_add() {
        let mut lifter = Lifter::new(0, vec![]);
        lifter.process_operator(&Operator::I32Const { value: 1 });
        lifter.process_operator(&Operator::I32Const { value: 2 });
        lifter.process_operator(&Operator::I32Add);
        assert_eq!(lifter.value_stack.len(), 1);
        match &lifter.value_stack[0] {
            Expr::BinaryOp { op, .. } => assert_eq!(*op, BinOp::Add),
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_lifter_local_set() {
        let mut lifter = Lifter::new(1, vec![ScalarType::Int]);
        lifter.process_operator(&Operator::I32Const { value: 10 });
        lifter.process_operator(&Operator::LocalSet { local_index: 0 });
        assert_eq!(lifter.value_stack.len(), 0);
        assert_eq!(lifter.current_body.len(), 1);
    }
}
