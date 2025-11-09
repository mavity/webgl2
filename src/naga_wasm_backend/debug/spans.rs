//! Source span tracking for DWARF line mappings

/// Source location span
#[derive(Debug, Clone)]
pub struct SourceSpan {
    /// File name
    pub file: String,
    /// Line number (1-based)
    pub line: u32,
    /// Column number (0-based)
    pub column: u32,
}
