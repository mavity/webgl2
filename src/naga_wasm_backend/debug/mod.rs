//! Debug module for DWARF generation
//!
//! Placeholder for DWARF generation infrastructure

pub mod dwarf;
pub mod spans;
pub mod stub;
pub mod variables;

pub use dwarf::DwarfGenerator;
pub use stub::JsStubGenerator;
