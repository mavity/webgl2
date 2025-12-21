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
    info: &ModuleInfo,
    source: &str,
) -> Result<WasmModule, BackendError> {
    tracing::info!(
        "Starting WASM compilation for module with {} entry points",
        module.entry_points.len()
    );

    let mut compiler = Compiler::new(backend, info, source);
    compiler.compile(module)?;

    Ok(compiler.finish())
}

/// Internal compiler state
struct Compiler<'a> {
    _backend: &'a WasmBackend,
    _info: &'a ModuleInfo,
    _source: &'a str,

    // WASM module sections
    types: TypeSection,
    functions: FunctionSection,
    code: CodeSection,
    exports: ExportSection,
    memory: MemorySection,

    // Tracking
    entry_points: HashMap<String, u32>,
    function_count: u32,
    global_offsets: HashMap<naga::Handle<naga::GlobalVariable>, u32>,

    // Debug info (if enabled)
    debug_generator: Option<super::debug::DwarfGenerator>,
}

impl<'a> Compiler<'a> {
    fn new(backend: &'a WasmBackend, info: &'a ModuleInfo, source: &'a str) -> Self {
        let debug_generator = if backend.config.debug_info {
            Some(super::debug::DwarfGenerator::new(source))
        } else {
            None
        };

        Self {
            _backend: backend,
            _info: info,
            _source: source,
            types: TypeSection::new(),
            functions: FunctionSection::new(),
            code: CodeSection::new(),
            exports: ExportSection::new(),
            memory: MemorySection::new(),
            entry_points: HashMap::new(),
            function_count: 0,
            global_offsets: HashMap::new(),
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

        // Calculate global offsets
        let mut offset = 0;
        for (handle, var) in module.global_variables.iter() {
            self.global_offsets.insert(handle, offset);
            let size = super::types::type_size(&module.types[var.ty].inner)?;
            offset += size;
            // Align to 16 bytes
            offset = (offset + 15) & !15;
        }

        // Compile each entry point
        for (idx, entry_point) in module.entry_points.iter().enumerate() {
            tracing::debug!(
                "Processing entry point: {} (stage: {:?})",
                entry_point.name,
                entry_point.stage
            );
            self.compile_entry_point(entry_point, module, idx)?;
        }

        Ok(())
    }

    fn compile_entry_point(
        &mut self,
        entry_point: &naga::EntryPoint,
        module: &naga::Module,
        _index: usize,
    ) -> Result<(), BackendError> {
        // Signature: (attr_ptr, uniform_ptr, varying_ptr) -> ()
        let params = vec![ValType::I32, ValType::I32, ValType::I32];
        let results = vec![];

        let type_idx = self.types.len();
        self.types.ty().function(params, results);

        let func_idx = self.function_count;
        self.functions.function(type_idx);

        // Create function body
        let mut wasm_func = Function::new(vec![]); // No locals yet

        // Translate statements
        for stmt in &entry_point.function.body {
            super::control_flow::translate_statement(
                stmt,
                &entry_point.function,
                module,
                &mut wasm_func,
                &self.global_offsets,
            )?;
        }

        wasm_func.instruction(&Instruction::End);
        self.code.function(&wasm_func);

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
