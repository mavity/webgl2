//! WASM to GLSL Decompiler
//!
//! This module provides functionality to decompile WASM bytecode back into
//! GLSL-like C code. It implements the plan from docs/11.b-decompile-theory.md.
//!
//! # Architecture
//!
//! The decompiler is organized into four phases:
//!
//! 1. **Parser** (`parser.rs`): Uses `wasmparser` to extract function bodies
//!    and metadata from WASM bytecode.
//!
//! 2. **Lifter** (`lifter.rs`): Converts stack-based WASM instructions into
//!    a tree-based AST using a symbolic stack approach.
//!
//! 3. **AST** (`ast.rs`): Defines the intermediate representation with
//!    expressions (`Expr`) and statements (`Stmt`).
//!
//! 4. **Emitter** (`emitter.rs`): Generates GLSL source code from the AST.
//!
//! # Example
//!
//! ```ignore
//! use webgl2::decompiler::decompile_to_glsl;
//!
//! let wasm_bytes: &[u8] = /* ... */;
//! let glsl = decompile_to_glsl(wasm_bytes)?;
//! println!("{}", glsl);
//! ```

pub mod ast;
pub mod emitter;
pub mod lifter;
pub mod module;
pub mod parser;
pub mod simplifier;

use anyhow::Result;
use emitter::{Emitter, EmitterConfig};
use parser::parse_wasm;
use simplifier::simplify_expr;

/// Decompile WASM bytecode to GLSL source code.
///
/// This is the main entry point for the decompiler. It parses the WASM
/// module, lifts all functions to AST form, and emits GLSL code.
///
/// # Arguments
///
/// * `wasm_bytes` - The raw WASM bytecode to decompile
///
/// # Returns
///
/// A string containing the decompiled GLSL source code.
///
/// # Errors
///
/// Returns an error if the WASM bytecode is invalid or cannot be parsed.
pub fn decompile_to_glsl(wasm_bytes: &[u8]) -> Result<String> {
    decompile_to_glsl_with_config(wasm_bytes, EmitterConfig::default())
}

/// Decompile WASM bytecode to GLSL source code with custom configuration.
///
/// # Arguments
///
/// * `wasm_bytes` - The raw WASM bytecode to decompile
/// * `config` - Emitter configuration (GLSL version, indentation, etc.)
///
/// # Returns
///
/// A string containing the decompiled GLSL source code.
pub fn decompile_to_glsl_with_config(wasm_bytes: &[u8], config: EmitterConfig) -> Result<String> {
    let mut module = parse_wasm(wasm_bytes)?;

    // Phase 3: Simplify all expressions in all functions using egg
    for func in module.functions.values_mut() {
        simplify_function(func);
    }

    let mut emitter = Emitter::new(config);

    // Emit header
    emitter.emit_header();

    // Emit memory buffer declaration if needed
    // (We could make this configurable)
    emitter.emit_memory_buffer();

    // Emit all functions in WASM index order (deterministic)
    let mut indices: Vec<_> = module.functions.keys().copied().collect();
    indices.sort();

    for idx in indices {
        let func = &module.functions[&idx];
        let name = module.get_function_name(idx);
        emitter.emit_function(func, &name);
        // Add blank line between functions
    }

    Ok(emitter.finish())
}

/// Simplify all expressions in a function using equality saturation.
fn simplify_function(func: &mut ast::Function) {
    for stmt in &mut func.body {
        simplify_stmt(stmt);
    }
}

/// Recursively simplify expressions in a statement.
fn simplify_stmt(stmt: &mut ast::Stmt) {
    match stmt {
        ast::Stmt::LocalSet { value, .. } => {
            *value = simplify_expr(value);
        }
        ast::Stmt::GlobalSet { value, .. } => {
            *value = simplify_expr(value);
        }
        ast::Stmt::MemoryStore { addr, value, .. } => {
            *addr = simplify_expr(addr);
            *value = simplify_expr(value);
        }
        ast::Stmt::If {
            condition,
            then_body,
            else_body,
        } => {
            *condition = simplify_expr(condition);
            for s in then_body {
                simplify_stmt(s);
            }
            if let Some(else_stmts) = else_body {
                for s in else_stmts {
                    simplify_stmt(s);
                }
            }
        }
        ast::Stmt::Block { body } => {
            for s in body {
                simplify_stmt(s);
            }
        }
        ast::Stmt::Loop { body } => {
            for s in body {
                simplify_stmt(s);
            }
        }
        ast::Stmt::Return { value } => {
            if let Some(v) = value {
                *v = simplify_expr(v);
            }
        }
        ast::Stmt::ExprStmt(expr) => {
            *expr = simplify_expr(expr);
        }
        // Statements without expressions to simplify
        ast::Stmt::Break { .. }
        | ast::Stmt::Continue { .. }
        | ast::Stmt::Drop
        | ast::Stmt::Unknown(_) => {}
    }
}

/// Decompile a single WASM function to GLSL.
///
/// # Arguments
///
/// * `wasm_bytes` - The raw WASM bytecode
/// * `func_idx` - The function index to decompile
///
/// # Returns
///
/// A string containing the decompiled GLSL source code for the function.
pub fn decompile_function_to_glsl(wasm_bytes: &[u8], func_idx: u32) -> Result<String> {
    let module = parse_wasm(wasm_bytes)?;

    if let Some(func) = module.get_function(func_idx) {
        let name = module.get_function_name(func_idx);
        let mut emitter = Emitter::new(EmitterConfig::default());
        emitter.emit_function(func, &name);
        Ok(emitter.finish())
    } else {
        Err(anyhow::anyhow!("Function {} not found", func_idx))
    }
}

/// Get the parsed module for inspection.
///
/// This is useful for advanced use cases where you want to inspect
/// the AST before generating code.
pub fn parse_module(wasm_bytes: &[u8]) -> Result<DecompiledModule> {
    parse_wasm(wasm_bytes)
}

// Re-export key types for convenience
pub use ast::{BinOp, Expr, Function, ScalarType, Stmt, UnaryOp};
pub use module::DecompiledModule;
pub use simplifier::SimplifierConfig;

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal valid WASM module with one function that returns 42
    const MINIMAL_WASM: &[u8] = &[
        0x00, 0x61, 0x73, 0x6D, // magic
        0x01, 0x00, 0x00, 0x00, // version
        // Type section
        0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7F, // () -> i32
        // Function section
        0x03, 0x02, 0x01, 0x00, // function 0 uses type 0
        // Export section
        0x07, 0x08, 0x01, 0x04, 0x6D, 0x61, 0x69, 0x6E, 0x00, 0x00, // export "main" = func 0
        // Code section
        0x0A, 0x06, 0x01, 0x04, 0x00, 0x41, 0x2A, 0x0B, // func: i32.const 42, end
    ];

    #[test]
    fn test_decompile_minimal() {
        let result = decompile_to_glsl(MINIMAL_WASM);
        assert!(result.is_ok(), "Decompile failed: {:?}", result);
        let glsl = result.unwrap();
        assert!(glsl.contains("#version 300 es"));
        assert!(glsl.contains("int main()"));
        assert!(glsl.contains("return 42;"));
    }

    #[test]
    fn test_parse_module() {
        let result = parse_module(MINIMAL_WASM);
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.functions.len(), 1);
    }
}
