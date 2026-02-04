//! Module container for decompiled functions.
//!
//! This module provides a container that holds the results of parsing
//! a WASM module and stores decompiled functions.

use super::ast::Function;
use std::collections::HashMap;

/// A decompiled WASM module.
#[derive(Debug, Clone)]
pub struct DecompiledModule {
    /// Map from function index to decompiled function
    pub functions: HashMap<u32, Function>,
    /// Function names (from export or name section)
    pub function_names: HashMap<u32, String>,
    /// Number of imported functions (offset for code section functions)
    pub import_count: u32,
}

impl DecompiledModule {
    /// Create a new empty module.
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            function_names: HashMap::new(),
            import_count: 0,
        }
    }

    /// Add a decompiled function.
    pub fn add_function(&mut self, func: Function) {
        self.functions.insert(func.func_idx, func);
    }

    /// Set a function name.
    pub fn set_function_name(&mut self, idx: u32, name: String) {
        self.function_names.insert(idx, name);
    }

    /// Get a function by index.
    pub fn get_function(&self, idx: u32) -> Option<&Function> {
        self.functions.get(&idx)
    }

    /// Get function name, or generate a default one.
    pub fn get_function_name(&self, idx: u32) -> String {
        self.function_names
            .get(&idx)
            .cloned()
            .unwrap_or_else(|| format!("func{}", idx))
    }
}

impl Default for DecompiledModule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_add_function() {
        let mut module = DecompiledModule::new();
        let func = Function {
            func_idx: 0,
            param_count: 0,
            param_types: vec![],
            return_type: None,
            local_types: vec![],
            body: vec![],
        };
        module.add_function(func);
        assert!(module.get_function(0).is_some());
    }

    #[test]
    fn test_module_function_names() {
        let mut module = DecompiledModule::new();
        module.set_function_name(0, "main".to_string());
        assert_eq!(module.get_function_name(0), "main");
        assert_eq!(module.get_function_name(1), "func1");
    }
}
