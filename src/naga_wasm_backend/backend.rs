//! Core WASM code generation logic

use super::{BackendError, MemoryLayout, WasmBackend, WasmModule};
use naga::{valid::ModuleInfo, Module, front::Typifier};
use std::collections::HashMap;
use wasm_encoder::{
    CodeSection, CustomSection, ExportKind, ExportSection, Function, FunctionSection, Instruction,
    ImportSection, MemoryType, TypeSection, ValType,
};

/// Compile a Naga module to WASM bytecode
pub(super) fn compile_module(
    backend: &WasmBackend,
    module: &Module,
    info: &ModuleInfo,
    source: &str,
    stage: naga::ShaderStage,
) -> Result<WasmModule, BackendError> {
    tracing::info!(
        "Starting WASM compilation for module with {} entry points",
        module.entry_points.len()
    );

    let mut compiler = Compiler::new(backend, info, source, stage);
    compiler.compile(module)?;

    Ok(compiler.finish())
}

/// Internal compiler state
struct Compiler<'a> {
    _backend: &'a WasmBackend,
    _info: &'a ModuleInfo,
    _source: &'a str,
    stage: naga::ShaderStage,

    // WASM module sections
    types: TypeSection,
    imports: ImportSection,
    functions: FunctionSection,
    globals: wasm_encoder::GlobalSection,
    code: CodeSection,
    exports: ExportSection,

    // Tracking
    entry_points: HashMap<String, u32>,
    function_count: u32,
    naga_function_map: HashMap<naga::Handle<naga::Function>, u32>,
    global_offsets: HashMap<naga::Handle<naga::GlobalVariable>, u32>,

    // Debug info (if enabled)
    debug_generator: Option<super::debug::DwarfGenerator>,
}

impl<'a> Compiler<'a> {
    fn new(backend: &'a WasmBackend, info: &'a ModuleInfo, source: &'a str, stage: naga::ShaderStage) -> Self {
        let debug_generator = if backend.config.debug_info {
            Some(super::debug::DwarfGenerator::new(source))
        } else {
            None
        };

        Self {
            _backend: backend,
            _info: info,
            _source: source,
            stage,
            types: TypeSection::new(),
            imports: ImportSection::new(),
            functions: FunctionSection::new(),
            globals: wasm_encoder::GlobalSection::new(),
            code: CodeSection::new(),
            exports: ExportSection::new(),
            entry_points: HashMap::new(),
            function_count: 0,
            naga_function_map: HashMap::new(),
            global_offsets: HashMap::new(),
            debug_generator,
        }
    }

    fn compile(&mut self, module: &Module) -> Result<(), BackendError> {
        // Import memory from host
        self.imports.import("env", "memory", MemoryType {
            minimum: 1,
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        });

        // Define 4 globals for base pointers
        for _ in 0..4 {
            self.globals.global(
                wasm_encoder::GlobalType {
                    val_type: ValType::I32,
                    mutable: true,
                    shared: false,
                },
                &wasm_encoder::ConstExpr::i32_const(0),
            );
        }

        // Calculate global offsets per address space
        let mut uniform_offset = 0;
        let mut varying_offset = 0;
        let mut attr_offset = 0;
        let mut private_offset = 0;

        // First pass: find gl_Position and put it at the start of varying buffer
        for (handle, var) in module.global_variables.iter() {
            let is_position = if let Some(name) = &var.name {
                name == "gl_Position" || name == "gl_Position_1"
            } else {
                false
            };

            if is_position {
                self.global_offsets.insert(handle, 0);
                varying_offset = 16; // gl_Position is vec4 (16 bytes)
                break;
            }
        }

        // We need to know which variables are inputs/outputs
        // For now, let's look at the first entry point
        if let Some(ep) = module.entry_points.first() {
            for arg in &ep.function.arguments {
                // Naga GLSL frontend often uses Private for inputs
                // We can't easily link them back to GlobalVariable handles here
                // without more complex analysis.
            }
        }

        for (handle, var) in module.global_variables.iter() {
            if self.global_offsets.contains_key(&handle) {
                continue;
            }
            let size = super::types::type_size(&module.types[var.ty].inner).unwrap_or(4);
            
            // Heuristic for GLSL: 
            // - Uniform -> Uniform
            // - Private -> could be In, Out, or actual Private
            // For now, let's assume:
            // - If it has a name and we are in VS:
            //   - "gl_Position" -> Out (already handled)
            //   - other names -> In (Attributes)
            // This is a very rough heuristic.
            
            let offset = match var.space {
                naga::AddressSpace::Uniform => {
                    let o = uniform_offset;
                    uniform_offset += 16; // Each uniform gets 16 bytes to match ctx_uniform setters
                    o
                }
                naga::AddressSpace::Handle => 0,
                naga::AddressSpace::Private => {
                    // Heuristic: if it's not gl_Position and we are in VS, it's probably an attribute
                    // Actually, let's just put everything in Private for now unless it's gl_Position
                    let o = private_offset;
                    private_offset += size;
                    private_offset = (private_offset + 3) & !3;
                    o
                }
                _ => {
                    let o = varying_offset;
                    varying_offset += size;
                    varying_offset = (varying_offset + 3) & !3;
                    o
                }
            };
            self.global_offsets.insert(handle, offset);
        }

        // Compile all internal functions first
        for (handle, func) in module.functions.iter() {
            let func_idx = self.compile_function(func, module, None)?;
            self.naga_function_map.insert(handle, func_idx);
        }

        // Compile each entry point
        for (idx, entry_point) in module.entry_points.iter().enumerate() {
            crate::js_print(&format!("DEBUG: Entry point '{}' has {} arguments", entry_point.name, entry_point.function.arguments.len()));
            for (i, arg) in entry_point.function.arguments.iter().enumerate() {
                crate::js_print(&format!("DEBUG: Argument {}: name={:?}, binding={:?}", i, arg.name, arg.binding));
            }
            self.compile_entry_point(entry_point, module, idx)?;
        }

        Ok(())
    }

    fn compile_function(
        &mut self,
        func: &naga::Function,
        module: &naga::Module,
        entry_point: Option<&naga::EntryPoint>,
    ) -> Result<u32, BackendError> {
        let func_idx = self.function_count;
        self.function_count += 1;

        let mut params = vec![];
        let mut results = vec![];

        if let Some(_) = entry_point {
            // Entry point signature: (attr_ptr, uniform_ptr, varying_ptr, private_ptr) -> ()
            params = vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32];
        } else {
            // Internal function signature based on Naga
            crate::js_print(&format!("DEBUG: Internal function has {} arguments", func.arguments.len()));
            for (i, arg) in func.arguments.iter().enumerate() {
                let ty = super::types::naga_to_wasm_type(&module.types[arg.ty].inner)?;
                crate::js_print(&format!("DEBUG: Internal arg {}: type={:?}", i, ty));
                params.push(ty);
            }
            if let Some(ret) = &func.result {
                let ty = super::types::naga_to_wasm_type(&module.types[ret.ty].inner)?;
                crate::js_print(&format!("DEBUG: Internal return: type={:?}", ty));
                results.push(ty);
            }
        }

        let type_idx = self.types.len();
        self.types.ty().function(params, results);
        self.functions.function(type_idx);

        // Create function body
        let mut wasm_func = Function::new(vec![]); // TODO: locals

        if let Some(_) = entry_point {
            // Set globals from arguments
            for i in 0..4 {
                wasm_func.instruction(&Instruction::LocalGet(i));
                wasm_func.instruction(&Instruction::GlobalSet(i));
            }
        }

        let mut typifier = Typifier::new();
        let resolve_ctx = naga::proc::ResolveContext::with_locals(
            module,
            &func.local_variables,
            &func.arguments,
        );
        for (handle, _) in func.expressions.iter() {
            typifier.grow(handle, &func.expressions, &resolve_ctx).map_err(|e| BackendError::UnsupportedFeature(format!("Typifier error: {:?}", e)))?;
        }

        let stage = self.stage;
        let is_entry_point = entry_point.is_some();

        // Translate statements
        for stmt in &func.body {
            super::control_flow::translate_statement(
                stmt,
                func,
                module,
                &mut wasm_func,
                &self.global_offsets,
                stage,
                &typifier,
                &self.naga_function_map,
                is_entry_point,
            )?;
        }

        wasm_func.instruction(&Instruction::End);
        self.code.function(&wasm_func);

        Ok(func_idx)
    }

    fn compile_entry_point(
        &mut self,
        entry_point: &naga::EntryPoint,
        module: &naga::Module,
        _index: usize,
    ) -> Result<(), BackendError> {
        let func_idx = self.compile_function(&entry_point.function, module, Some(entry_point))?;

        // Export the function
        self.exports
            .export(&entry_point.name, ExportKind::Func, func_idx);

        self.entry_points.insert(entry_point.name.clone(), func_idx);

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
        module.section(&self.imports);
        module.section(&self.functions);
        module.section(&self.globals);
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
