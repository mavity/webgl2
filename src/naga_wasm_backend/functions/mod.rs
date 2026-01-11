//! Function registry and preparation pass.
//!
//! This module provides infrastructure for analyzing Naga functions and computing
//! their ABI and frame requirements in a preparation pass, before code emission.

mod prep;
mod registry;

#[cfg(test)]
mod tests;

pub use prep::prep_module;
pub use registry::{FunctionManifest, FunctionRegistry};
