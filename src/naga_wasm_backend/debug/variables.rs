//! Variable debug information tracking

/// Debug information for a variable
#[derive(Debug, Clone)]
pub struct VariableDebugInfo {
    /// Variable name
    pub name: String,
    /// WASM local index
    pub local_index: u32,
    /// Type description
    pub type_desc: String,
}
