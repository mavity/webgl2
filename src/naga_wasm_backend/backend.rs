//! Core WASM code generation logic

use super::{BackendError, MemoryLayout, WasmBackend, WasmModule};
use naga::{valid::ModuleInfo, Module};
use std::collections::HashMap;
use wasm_encoder::{
    CodeSection, CustomSection, ExportKind, ExportSection, Function, FunctionSection, Instruction,
    MemorySection, MemoryType, TypeSection, ValType,
};

/// Compile a Naga module to WASM bytecode
pub(super) fn compile_module(
    backend: &WasmBackend,
    module: &Module,
    _info: &ModuleInfo,
    source: &str,
) -> Result<WasmModule, BackendError> {
    tracing::info!(
        "Starting WASM compilation for module with {} entry points",
        module.entry_points.len()
    );

    let mut compiler = Compiler::new(backend, source);
    compiler.compile(module)?;

    Ok(compiler.finish())
}

/// Internal compiler state
struct Compiler<'a> {
    backend: &'a WasmBackend,
    source: &'a str,

    // WASM module sections
    types: TypeSection,
    functions: FunctionSection,
    code: CodeSection,
    exports: ExportSection,
    memory: MemorySection,

    // Tracking
    entry_points: HashMap<String, u32>,
    function_count: u32,

    // Debug info (if enabled)
    debug_generator: Option<super::debug::DwarfGenerator>,
}

impl<'a> Compiler<'a> {
    fn new(backend: &'a WasmBackend, source: &'a str) -> Self {
        let debug_generator = if backend.config.debug_info {
            Some(super::debug::DwarfGenerator::new(source))
        } else {
            None
        };

        Self {
            backend,
            source,
            types: TypeSection::new(),
            functions: FunctionSection::new(),
            code: CodeSection::new(),
            exports: ExportSection::new(),
            memory: MemorySection::new(),
            entry_points: HashMap::new(),
            function_count: 0,
            debug_generator,
        }
    }

    fn compile(&mut self, module: &Module) -> Result<(), BackendError> {
        // Set up memory (1 page = 64KB initially, growable)
        self.memory.memory(MemoryType {
            minimum: 1,
            maximum: Some(16), // Max 1MB
            memory64: false,
            shared: false,
            page_size_log2: None,
        });

        // For Phase 0: Just create a minimal empty function for each entry point
        for entry_point in &module.entry_points {
            tracing::debug!(
                "Processing entry point: {} (stage: {:?})",
                entry_point.name,
                entry_point.stage
            );
            self.compile_entry_point(entry_point)?;
        }

        Ok(())
    }

    fn compile_entry_point(&mut self, entry_point: &naga::EntryPoint) -> Result<(), BackendError> {
        // Phase 0: Create a minimal function that returns zeros
        // Type: (i32, i32, i32) -> (f32, f32, f32, f32)
        // This represents: (attr_ptr, uniform_ptr, varying_ptr) -> (x, y, z, w) for position/color

        let type_idx = self.types.len();
        self.types.ty()
            .function(
                vec![ValType::I32, ValType::I32, ValType::I32],
                vec![ValType::F32, ValType::F32, ValType::F32, ValType::F32],
            );

        let func_idx = self.function_count;
        self.functions.function(type_idx);

        // Create function body - just return zeros for now
        let mut func = Function::new(vec![]); // No locals yet

        // Return (0.0, 0.0, 0.0, 1.0)
        func.instruction(&Instruction::F32Const(0.0));
        func.instruction(&Instruction::F32Const(0.0));
        func.instruction(&Instruction::F32Const(0.0));
        func.instruction(&Instruction::F32Const(1.0));
        func.instruction(&Instruction::End);

        self.code.function(&func);

        // Export the function
        self.exports
            .export(&entry_point.name, ExportKind::Func, func_idx);

        self.entry_points.insert(entry_point.name.clone(), func_idx);
        self.function_count += 1;

        tracing::debug!(
            "Compiled entry point '{}' as function index {}",
            entry_point.name,
            func_idx
        );

        Ok(())
    }

    fn finish(self) -> WasmModule {
        // Assemble WASM module
        let mut module = wasm_encoder::Module::new();

        // Add standard sections
        module.section(&self.types);
        module.section(&self.functions);
        module.section(&self.memory);
        module.section(&self.exports);
        module.section(&self.code);

        // Add DWARF debug information if enabled
        let dwarf_bytes = if let Some(debug_gen) = self.debug_generator {
            let dwarf_data = debug_gen.finish();

            // Add custom sections for DWARF
            for (name, data) in dwarf_data {
                let custom = CustomSection {
                    name: std::borrow::Cow::Borrowed(&name),
                    data: std::borrow::Cow::Borrowed(&data),
                };
                module.section(&custom);
            }

            Some(vec![]) // Placeholder for separate DWARF file (future use)
        } else {
            None
        };

        let wasm_bytes = module.finish();

        tracing::info!(
            "Finished compilation: {} bytes, {} entry points",
            wasm_bytes.len(),
            self.entry_points.len()
        );

        WasmModule {
            wasm_bytes,
            dwarf_bytes,
            entry_points: self.entry_points,
            memory_layout: MemoryLayout::default(),
        }
    }
}
