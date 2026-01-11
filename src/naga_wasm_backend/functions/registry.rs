//! Function registry: stores computed ABIs and frame manifests for all functions.

use super::super::function_abi::FunctionABI;
use naga::{Function, Handle};
use std::collections::HashMap;

/// Manifest of ABI and frame requirements for a single function.
#[derive(Debug, Clone)]
pub struct FunctionManifest {
    /// The computed function ABI (parameters and result layout).
    pub abi: FunctionABI,
    /// Bytes needed on FRAME_SP linear stack for this function's frame.
    pub linear_frame_size: u32,
    /// Whether this function needs frame allocation during calls.
    pub needs_frame_alloc: bool,
}

/// Registry of all function manifests computed during the preparation pass.
#[derive(Debug, Clone)]
pub struct FunctionRegistry {
    /// Manifests for regular Naga functions (indexed by function handle).
    manifests: HashMap<Handle<Function>, FunctionManifest>,
    /// Manifests for entry points (indexed by entry point name).
    entry_manifests: HashMap<String, FunctionManifest>,
}

impl FunctionRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            manifests: HashMap::new(),
            entry_manifests: HashMap::new(),
        }
    }

    /// Insert a manifest for a regular function.
    pub fn insert_function(&mut self, handle: Handle<Function>, manifest: FunctionManifest) {
        self.manifests.insert(handle, manifest);
    }

    /// Insert a manifest for an entry point.
    pub fn insert_entry_point(&mut self, name: String, manifest: FunctionManifest) {
        self.entry_manifests.insert(name, manifest);
    }

    /// Get the manifest for a regular function.
    pub fn get_function(&self, handle: Handle<Function>) -> Option<&FunctionManifest> {
        self.manifests.get(&handle)
    }

    /// Get the manifest for an entry point.
    pub fn get_entry_point(&self, name: &str) -> Option<&FunctionManifest> {
        self.entry_manifests.get(name)
    }

    /// Get the number of registered functions.
    pub fn function_count(&self) -> usize {
        self.manifests.len()
    }

    /// Get the number of registered entry points.
    pub fn entry_point_count(&self) -> usize {
        self.entry_manifests.len()
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
