//! GLSL Emitter: Converts the AST into GLSL source code.
//!
//! This module implements the code generation phase, turning our
//! intermediate representation into readable GLSL code.

use super::ast::{Expr, Function, ScalarType, Stmt, UnaryOp};
use std::fmt::Write;

/// Configuration for the GLSL emitter.
#[derive(Debug, Clone)]
pub struct EmitterConfig {
    /// GLSL version (e.g., 300 for GLSL ES 3.0)
    pub glsl_version: u32,
    /// Whether to use ES flavor
    pub use_es: bool,
    /// Indent string (e.g., "  " or "\t")
    pub indent: String,
}

impl Default for EmitterConfig {
    fn default() -> Self {
        Self {
            glsl_version: 300,
            use_es: true,
            indent: "    ".to_string(),
        }
    }
}

/// The GLSL emitter.
pub struct Emitter {
    config: EmitterConfig,
    output: String,
    indent_level: usize,
}

impl Emitter {
    /// Create a new emitter with the given configuration.
    pub fn new(config: EmitterConfig) -> Self {
        Self {
            config,
            output: String::new(),
            indent_level: 0,
        }
    }

    /// Create a new emitter with default configuration.
    pub fn default_config() -> Self {
        Self::new(EmitterConfig::default())
    }

    /// Get the current indentation string.
    fn indent(&self) -> String {
        self.config.indent.repeat(self.indent_level)
    }

    /// Write a line with current indentation.
    fn write_line(&mut self, line: &str) {
        let _ = writeln!(self.output, "{}{}", self.indent(), line);
    }

    /// Write without newline.
    #[allow(dead_code)]
    fn write(&mut self, text: &str) {
        let _ = write!(self.output, "{}", text);
    }

    /// Emit the GLSL header (version, precision).
    pub fn emit_header(&mut self) {
        if self.config.use_es {
            self.write_line(&format!("#version {} es", self.config.glsl_version));
            self.write_line("precision highp float;");
            self.write_line("precision highp int;");
        } else {
            self.write_line(&format!("#version {}", self.config.glsl_version));
        }
        self.write_line("");
    }

    /// Emit a memory buffer declaration (for WASM linear memory).
    pub fn emit_memory_buffer(&mut self) {
        self.write_line("// WASM linear memory mapped to buffer");
        self.write_line("layout(std430, binding = 0) buffer MemoryBuffer {");
        self.indent_level += 1;
        self.write_line("int memory[];");
        self.indent_level -= 1;
        self.write_line("};");
        self.write_line("");
    }

    /// Emit a function.
    pub fn emit_function(&mut self, func: &Function, name: &str) {
        // Return type
        let return_type = func.return_type.map(|t| t.glsl_name()).unwrap_or("void");

        // Parameters
        let params: Vec<String> = func
            .param_types
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("{} p{}", ty.glsl_name(), i))
            .collect();
        let params_str = params.join(", ");

        self.write_line(&format!("{} {}({}) {{", return_type, name, params_str));
        self.indent_level += 1;

        // Local variable declarations (excluding parameters)
        for (i, ty) in func.local_types.iter().enumerate() {
            let local_idx = func.param_count as usize + i;
            self.write_line(&format!("{} v{};", ty.glsl_name(), local_idx));
        }
        if !func.local_types.is_empty() {
            self.write_line("");
        }

        // Function body
        for stmt in &func.body {
            self.emit_stmt(stmt, func.param_count);
        }

        self.indent_level -= 1;
        self.write_line("}");
    }

    /// Emit a statement.
    fn emit_stmt(&mut self, stmt: &Stmt, param_count: u32) {
        match stmt {
            Stmt::LocalSet { local_idx, value } => {
                let var_name = self.local_name(*local_idx, param_count);
                let expr_str = self.expr_to_string(value, param_count);
                self.write_line(&format!("{} = {};", var_name, expr_str));
            }
            Stmt::GlobalSet { global_idx, value } => {
                let expr_str = self.expr_to_string(value, param_count);
                self.write_line(&format!("g{} = {};", global_idx, expr_str));
            }
            Stmt::MemoryStore {
                ty,
                offset,
                addr,
                value,
            } => {
                let addr_str = self.expr_to_string(addr, param_count);
                let value_str = self.expr_to_string(value, param_count);
                // Assume 4-byte aligned for int/float
                let index_expr = if *offset == 0 {
                    format!("({}) >> 2", addr_str)
                } else {
                    format!("(({}) + {}) >> 2", addr_str, offset)
                };
                match ty {
                    ScalarType::Int | ScalarType::Long => {
                        self.write_line(&format!("memory[{}] = {};", index_expr, value_str));
                    }
                    ScalarType::Float | ScalarType::Double => {
                        self.write_line(&format!(
                            "memory[{}] = floatBitsToInt({});",
                            index_expr, value_str
                        ));
                    }
                }
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let cond_str = self.expr_to_string(condition, param_count);
                self.write_line(&format!("if ({}) {{", cond_str));
                self.indent_level += 1;
                for s in then_body {
                    self.emit_stmt(s, param_count);
                }
                self.indent_level -= 1;
                if let Some(else_stmts) = else_body {
                    self.write_line("} else {");
                    self.indent_level += 1;
                    for s in else_stmts {
                        self.emit_stmt(s, param_count);
                    }
                    self.indent_level -= 1;
                }
                self.write_line("}");
            }
            Stmt::Block { body } => {
                self.write_line("{");
                self.indent_level += 1;
                for s in body {
                    self.emit_stmt(s, param_count);
                }
                self.indent_level -= 1;
                self.write_line("}");
            }
            Stmt::Loop { body } => {
                self.write_line("while (true) {");
                self.indent_level += 1;
                for s in body {
                    self.emit_stmt(s, param_count);
                }
                self.indent_level -= 1;
                self.write_line("}");
            }
            Stmt::Break { depth: _ } => {
                self.write_line("break;");
            }
            Stmt::Continue { depth: _ } => {
                self.write_line("continue;");
            }
            Stmt::Return { value } => {
                if let Some(val) = value {
                    let val_str = self.expr_to_string(val, param_count);
                    self.write_line(&format!("return {};", val_str));
                } else {
                    self.write_line("return;");
                }
            }
            Stmt::ExprStmt(expr) => {
                let expr_str = self.expr_to_string(expr, param_count);
                self.write_line(&format!("{};", expr_str));
            }
            Stmt::Drop => {
                // No-op in GLSL
            }
            Stmt::Unknown(desc) => {
                self.write_line(&format!("/* unknown: {} */", desc));
            }
        }
    }

    /// Convert an expression to a string.
    fn expr_to_string(&self, expr: &Expr, param_count: u32) -> String {
        match expr {
            Expr::ConstI32(v) => format!("{}", v),
            Expr::ConstI64(v) => format!("{}", v),
            Expr::ConstF32(v) => {
                if v.fract() == 0.0 && !v.is_nan() && !v.is_infinite() {
                    format!("{:.1}", v)
                } else {
                    format!("{}", v)
                }
            }
            Expr::ConstF64(v) => {
                if v.fract() == 0.0 && !v.is_nan() && !v.is_infinite() {
                    format!("{:.1}", v)
                } else {
                    format!("{}", v)
                }
            }
            Expr::LocalGet(idx) => self.local_name(*idx, param_count),
            Expr::GlobalGet(idx) => format!("g{}", idx),
            Expr::BinaryOp { op, left, right } => {
                let left_str = self.expr_to_string(left, param_count);
                let right_str = self.expr_to_string(right, param_count);
                format!("({} {} {})", left_str, op.glsl_op(), right_str)
            }
            Expr::UnaryOp { op, operand } => {
                let operand_str = self.expr_to_string(operand, param_count);
                if op.is_function() {
                    format!("{}({})", op.glsl_name(), operand_str)
                } else if matches!(op, UnaryOp::Eqz) {
                    format!("({} == 0)", operand_str)
                } else {
                    format!("({}{})", op.glsl_name(), operand_str)
                }
            }
            Expr::Convert {
                from: _,
                to,
                operand,
            } => {
                let operand_str = self.expr_to_string(operand, param_count);
                format!("{}({})", to.glsl_name(), operand_str)
            }
            Expr::MemoryLoad { ty, offset, addr } => {
                let addr_str = self.expr_to_string(addr, param_count);
                let index_expr = if *offset == 0 {
                    format!("({}) >> 2", addr_str)
                } else {
                    format!("(({}) + {}) >> 2", addr_str, offset)
                };
                match ty {
                    ScalarType::Int | ScalarType::Long => {
                        format!("memory[{}]", index_expr)
                    }
                    ScalarType::Float | ScalarType::Double => {
                        format!("intBitsToFloat(memory[{}])", index_expr)
                    }
                }
            }
            Expr::Call { func_idx, args } => {
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| self.expr_to_string(a, param_count))
                    .collect();
                format!("func{}({})", func_idx, args_str.join(", "))
            }
            Expr::Select {
                condition,
                true_val,
                false_val,
            } => {
                let cond_str = self.expr_to_string(condition, param_count);
                let true_str = self.expr_to_string(true_val, param_count);
                let false_str = self.expr_to_string(false_val, param_count);
                format!("({} != 0 ? {} : {})", cond_str, true_str, false_str)
            }
            Expr::VecConstruct { components } => {
                let comps: Vec<String> = components
                    .iter()
                    .map(|c| self.expr_to_string(c, param_count))
                    .collect();
                let vec_type = match components.len() {
                    2 => "vec2",
                    3 => "vec3",
                    4 => "vec4",
                    _ => "vec4",
                };
                format!("{}({})", vec_type, comps.join(", "))
            }
            Expr::VecBinaryOp { op, left, right } => {
                let left_str = self.expr_to_string(left, param_count);
                let right_str = self.expr_to_string(right, param_count);
                format!("({} {} {})", left_str, op.glsl_op(), right_str)
            }
            Expr::Unknown(desc) => {
                format!("/* unknown: {} */", desc)
            }
        }
    }

    /// Generate a local variable name.
    fn local_name(&self, idx: u32, param_count: u32) -> String {
        if idx < param_count {
            format!("p{}", idx)
        } else {
            format!("v{}", idx)
        }
    }

    /// Get the generated output.
    pub fn finish(self) -> String {
        self.output
    }
}

/// Convenience function to decompile a function to GLSL.
pub fn function_to_glsl(func: &Function, name: &str, config: Option<EmitterConfig>) -> String {
    let mut emitter = Emitter::new(config.unwrap_or_default());
    emitter.emit_header();
    emitter.emit_function(func, name);
    emitter.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompiler::ast::BinOp;

    #[test]
    fn test_emitter_simple_function() {
        let func = Function {
            func_idx: 0,
            param_count: 2,
            param_types: vec![ScalarType::Float, ScalarType::Float],
            return_type: Some(ScalarType::Float),
            local_types: vec![],
            body: vec![Stmt::Return {
                value: Some(Expr::BinaryOp {
                    op: BinOp::Add,
                    left: Box::new(Expr::LocalGet(0)),
                    right: Box::new(Expr::LocalGet(1)),
                }),
            }],
        };

        let output = function_to_glsl(&func, "add", None);
        assert!(output.contains("float add(float p0, float p1)"));
        assert!(output.contains("return (p0 + p1);"));
    }

    #[test]
    fn test_emitter_if_stmt() {
        let func = Function {
            func_idx: 0,
            param_count: 1,
            param_types: vec![ScalarType::Int],
            return_type: Some(ScalarType::Int),
            local_types: vec![],
            body: vec![Stmt::If {
                condition: Expr::LocalGet(0),
                then_body: vec![Stmt::Return {
                    value: Some(Expr::ConstI32(1)),
                }],
                else_body: Some(vec![Stmt::Return {
                    value: Some(Expr::ConstI32(0)),
                }]),
            }],
        };

        let output = function_to_glsl(&func, "test", None);
        assert!(output.contains("if (p0)"));
        assert!(output.contains("return 1;"));
        assert!(output.contains("} else {"));
        assert!(output.contains("return 0;"));
    }

    #[test]
    fn test_emitter_vec4() {
        let func = Function {
            func_idx: 0,
            param_count: 0,
            param_types: vec![],
            return_type: None,
            local_types: vec![],
            body: vec![Stmt::Return {
                value: Some(Expr::VecConstruct {
                    components: vec![
                        Expr::ConstF32(1.0),
                        Expr::ConstF32(0.0),
                        Expr::ConstF32(0.0),
                        Expr::ConstF32(1.0),
                    ],
                }),
            }],
        };

        let output = function_to_glsl(&func, "red", None);
        assert!(output.contains("vec4(1.0, 0.0, 0.0, 1.0)"));
    }
}
