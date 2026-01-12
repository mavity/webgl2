//! AST definitions for the WASM to GLSL decompiler.
//!
//! This module defines the intermediate representation (IR) used to translate
//! stack-based WASM instructions into a tree-based structure suitable for
//! GLSL code generation.

/// Scalar types that map to GLSL primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarType {
    Int,    // i32 -> int
    Long,   // i64 -> not directly supported in GLSL ES, treat as int
    Float,  // f32 -> float
    Double, // f64 -> not directly supported in GLSL ES, treat as float
}

impl ScalarType {
    /// Returns the GLSL type name for this scalar type.
    pub fn glsl_name(&self) -> &'static str {
        match self {
            ScalarType::Int | ScalarType::Long => "int",
            ScalarType::Float | ScalarType::Double => "float",
        }
    }
}

/// Binary operators supported in expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Rem,

    // Bitwise
    And,
    Or,
    Xor,
    Shl,
    ShrS, // signed shift right
    ShrU, // unsigned shift right

    // Comparison
    Eq,
    Ne,
    LtS, // signed less than
    LtU, // unsigned less than
    LeS,
    LeU,
    GtS,
    GtU,
    GeS,
    GeU,
}

impl BinOp {
    /// Returns the GLSL operator symbol.
    pub fn glsl_op(&self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::And => "&",
            BinOp::Or => "|",
            BinOp::Xor => "^",
            BinOp::Shl => "<<",
            BinOp::ShrS | BinOp::ShrU => ">>",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::LtS | BinOp::LtU => "<",
            BinOp::LeS | BinOp::LeU => "<=",
            BinOp::GtS | BinOp::GtU => ">",
            BinOp::GeS | BinOp::GeU => ">=",
        }
    }
}

/// Unary operators supported in expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not, // bitwise not
    Eqz, // equal to zero (boolean)
    Abs,
    Ceil,
    Floor,
    Trunc,
    Nearest, // round to nearest
    Sqrt,
}

impl UnaryOp {
    /// Returns true if this operator uses a function call syntax in GLSL.
    pub fn is_function(&self) -> bool {
        matches!(
            self,
            UnaryOp::Abs
                | UnaryOp::Ceil
                | UnaryOp::Floor
                | UnaryOp::Trunc
                | UnaryOp::Nearest
                | UnaryOp::Sqrt
        )
    }

    /// Returns the GLSL representation.
    pub fn glsl_name(&self) -> &'static str {
        match self {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "~",
            UnaryOp::Eqz => "!", // will be rendered as !(expr != 0) or similar
            UnaryOp::Abs => "abs",
            UnaryOp::Ceil => "ceil",
            UnaryOp::Floor => "floor",
            UnaryOp::Trunc => "trunc",
            UnaryOp::Nearest => "round",
            UnaryOp::Sqrt => "sqrt",
        }
    }
}

/// Expression nodes in the AST.
///
/// Expressions are pure values that can be nested arbitrarily.
#[derive(Debug, Clone)]
pub enum Expr {
    /// Integer constant
    ConstI32(i32),
    /// Long constant (i64)
    ConstI64(i64),
    /// Float constant
    ConstF32(f32),
    /// Double constant
    ConstF64(f64),

    /// Read a local variable
    LocalGet(u32),
    /// Read a global variable
    GlobalGet(u32),

    /// Binary operation
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    /// Unary operation
    UnaryOp { op: UnaryOp, operand: Box<Expr> },

    /// Type conversion
    Convert {
        from: ScalarType,
        to: ScalarType,
        operand: Box<Expr>,
    },

    /// Memory load
    MemoryLoad {
        ty: ScalarType,
        offset: u32,
        addr: Box<Expr>,
    },

    /// Function call
    Call { func_idx: u32, args: Vec<Expr> },

    /// Select (ternary operator)
    Select {
        condition: Box<Expr>,
        true_val: Box<Expr>,
        false_val: Box<Expr>,
    },

    /// Vector constructor (e.g., vec4(x, y, z, w))
    VecConstruct { components: Vec<Expr> },

    /// Vector binary operation (component-wise)
    VecBinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    /// A placeholder for unsupported or complex expressions
    Unknown(String),
}

/// Statement nodes in the AST.
///
/// Statements have side effects and form the body of functions.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Assign to a local variable
    LocalSet { local_idx: u32, value: Expr },

    /// Assign to a global variable
    GlobalSet { global_idx: u32, value: Expr },

    /// Memory store
    MemoryStore {
        ty: ScalarType,
        offset: u32,
        addr: Expr,
        value: Expr,
    },

    /// If statement (with optional else)
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },

    /// Block (for structured control flow)
    Block { body: Vec<Stmt> },

    /// Loop
    Loop { body: Vec<Stmt> },

    /// Break out of block (br)
    Break { depth: u32 },

    /// Continue to loop start (br for loop)
    Continue { depth: u32 },

    /// Return from function
    Return { value: Option<Expr> },

    /// Expression statement (for side effects like calls)
    ExprStmt(Expr),

    /// Drop a value from the stack (no-op in GLSL)
    Drop,

    /// Placeholder for unsupported statements
    Unknown(String),
}

/// A decompiled function.
#[derive(Debug, Clone)]
pub struct Function {
    /// Function index in the WASM module
    pub func_idx: u32,
    /// Number of parameters
    pub param_count: u32,
    /// Parameter types
    pub param_types: Vec<ScalarType>,
    /// Return type (None for void)
    pub return_type: Option<ScalarType>,
    /// Local variable types (excluding parameters)
    pub local_types: Vec<ScalarType>,
    /// Function body
    pub body: Vec<Stmt>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_type_glsl_name() {
        assert_eq!(ScalarType::Int.glsl_name(), "int");
        assert_eq!(ScalarType::Float.glsl_name(), "float");
    }

    #[test]
    fn test_binop_glsl_op() {
        assert_eq!(BinOp::Add.glsl_op(), "+");
        assert_eq!(BinOp::Mul.glsl_op(), "*");
        assert_eq!(BinOp::Eq.glsl_op(), "==");
    }

    #[test]
    fn test_expr_construction() {
        let expr = Expr::BinaryOp {
            op: BinOp::Add,
            left: Box::new(Expr::ConstI32(1)),
            right: Box::new(Expr::ConstI32(2)),
        };
        match expr {
            Expr::BinaryOp { op, .. } => assert_eq!(op, BinOp::Add),
            _ => panic!("Expected BinaryOp"),
        }
    }
}
