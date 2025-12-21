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
    uniform_locations: &HashMap<String, u32>,
    varying_locations: &HashMap<String, u32>,
) -> Result<WasmModule, BackendError> {
    tracing::info!(
        "Starting WASM compilation for module with {} entry points",
        module.entry_points.len()
    );

    let mut compiler = Compiler::new(backend, info, source, stage, uniform_locations, varying_locations);
    compiler.compile(module)?;

    Ok(compiler.finish())
}

/// Internal compiler state
struct Compiler<'a> {
    _backend: &'a WasmBackend,
    _info: &'a ModuleInfo,
    _source: &'a str,
    stage: naga::ShaderStage,
    uniform_locations: &'a HashMap<String, u32>,
    varying_locations: &'a HashMap<String, u32>,

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
    global_offsets: HashMap<naga::Handle<naga::GlobalVariable>, (u32, u32)>,
    argument_local_offsets: HashMap<u32, u32>,

    // Debug info (if enabled)
    debug_generator: Option<super::debug::DwarfGenerator>,
}

impl<'a> Compiler<'a> {
    fn new(
        backend: &'a WasmBackend,
        info: &'a ModuleInfo,
        source: &'a str,
        stage: naga::ShaderStage,
        uniform_locations: &'a HashMap<String, u32>,
        varying_locations: &'a HashMap<String, u32>,
    ) -> Self {
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
            uniform_locations,
            varying_locations,
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
            argument_local_offsets: HashMap::new(),
            debug_generator,
        }
    }

    fn compile(&mut self, module: &Module) -> Result<(), BackendError> {
        // Import memory from host
        self.imports.import("env", "memory", MemoryType {
            minimum: 10, // 640KB
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        });

        // Define 5 globals for base pointers
        // 0: attr, 1: uniform, 2: varying, 3: private, 4: textures
        for _ in 0..5 {
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
        let mut varying_offset = 0;
        let mut private_offset = 0;
        let mut attr_offset = 0;

        // First pass: find gl_Position and put it at the start of varying buffer
        for (handle, var) in module.global_variables.iter() {
            let is_position = if let Some(name) = &var.name {
                name == "gl_Position" || name == "gl_Position_1"
            } else {
                false
            };

            if is_position {
                self.global_offsets.insert(handle, (0, 2));
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
            
            let (offset, base_ptr) = match var.space {
                naga::AddressSpace::Uniform | naga::AddressSpace::Handle => {
                    if let Some(name) = &var.name {
                        if let Some(&loc) = self.uniform_locations.get(name) {
                            let offset = loc * 64;
                            (offset, 1)
                        } else {
                            (0, 1)
                        }
                    } else {
                        (0, 1)
                    }
                }
                naga::AddressSpace::Private | naga::AddressSpace::Function => {
                    // Check if it's an output in FS
                    let is_output = if self.stage == naga::ShaderStage::Fragment {
                        if let Some(name) = &var.name {
                            name == "color" || name == "gl_FragColor" || name == "fragColor" || name == "gl_FragColor_1"
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if is_output {
                        (0, 3)
                    } else if let Some(name) = &var.name {
                        if let Some(&loc) = self.varying_locations.get(name) {
                            let offset = (loc + 1) * 16;
                            (offset, 2)
                        } else {
                            let o = private_offset;
                            private_offset += size;
                            private_offset = (private_offset + 3) & !3;
                            (o, 3)
                        }
                    } else {
                        let o = private_offset;
                        private_offset += size;
                        private_offset = (private_offset + 3) & !3;
                        (o, 3)
                    }
                }
                _ => {
                    let o = varying_offset;
                    varying_offset += size;
                    varying_offset = (varying_offset + 3) & !3;
                    (o, 2)
                }
            };
            self.global_offsets.insert(handle, (offset, base_ptr));
        }

        // Compile all internal functions first
        for (handle, func) in module.functions.iter() {
            let func_idx = self.compile_function(func, module, None)?;
            self.naga_function_map.insert(handle, func_idx);
        }

        // Compile each entry point
        for (idx, entry_point) in module.entry_points.iter().enumerate() {
            crate::js_print(&format!("DEBUG: Entry point '{}' stage={:?} has {} arguments", entry_point.name, entry_point.stage, entry_point.function.arguments.len()));
            for (i, arg) in entry_point.function.arguments.iter().enumerate() {
                crate::js_print(&format!("DEBUG: Argument {}: name={:?}, binding={:?}", i, arg.name, arg.binding));
            }
            if let Some(ref result) = entry_point.function.result {
                crate::js_print(&format!("DEBUG: Entry point result: binding={:?}", result.binding));
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

        let mut params: Vec<ValType> = vec![];
        let mut results: Vec<ValType> = vec![];
        let mut argument_local_offsets = HashMap::new();
        let mut current_param_idx = 0;

        if let Some(_) = entry_point {
            // Entry point signature: (type, attr_ptr, uniform_ptr, varying_ptr, private_ptr, texture_ptr) -> ()
            params = vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32, ValType::I32, ValType::I32];
            current_param_idx = 6;
        } else {
            // Internal function signature based on Naga
            crate::js_print(&format!("DEBUG: Internal function has {} arguments", func.arguments.len()));
            for (i, arg) in func.arguments.iter().enumerate() {
                let types = super::types::naga_to_wasm_types(&module.types[arg.ty].inner)?;
                crate::js_print(&format!("DEBUG: Internal arg {}: types={:?}", i, types));
                argument_local_offsets.insert(i as u32, current_param_idx);
                current_param_idx += types.len() as u32;
                params.extend(types);
            }
            if let Some(ret) = &func.result {
                let types = super::types::naga_to_wasm_types(&module.types[ret.ty].inner)?;
                crate::js_print(&format!("DEBUG: Internal return: types={:?}", types));
                results.extend(types);
            }
        }

        let type_idx = self.types.len();
        self.types.ty().function(params, results);
        self.functions.function(type_idx);

        let mut typifier = Typifier::new();
        let resolve_ctx = naga::proc::ResolveContext::with_locals(
            module,
            &func.local_variables,
            &func.arguments,
        );
        for (handle, expr) in func.expressions.iter() {
            // crate::js_print(&format!("DEBUG: Expr {:?}: {:?}", handle, expr));
            typifier.grow(handle, &func.expressions, &resolve_ctx).map_err(|e| BackendError::UnsupportedFeature(format!("Typifier error: {:?}", e)))?;
        }

        // Calculate local variable offsets
        let mut local_offsets = HashMap::new();
        let mut current_local_offset = 0;
        for (handle, var) in func.local_variables.iter() {
            let size = super::types::type_size(&module.types[var.ty].inner).unwrap_or(4);
            local_offsets.insert(handle, current_local_offset);
            current_local_offset += size;
        }

        // Map CallResult expressions to WASM locals
        let mut call_result_locals = HashMap::new();
        let mut locals_types = vec![];
        let mut next_local_idx = current_param_idx;

        for (handle, expr) in func.expressions.iter() {
            if let naga::Expression::CallResult(func_handle) = expr {
                let called_func = &module.functions[*func_handle];
                if let Some(ret) = &called_func.result {
                    let types = super::types::naga_to_wasm_types(&module.types[ret.ty].inner)?;
                    call_result_locals.insert(handle, next_local_idx);
                    for _ in 0..types.len() {
                        locals_types.push((1, ValType::F32));
                    }
                    next_local_idx += types.len() as u32;
                }
            }
        }

        // Add scratch locals for complex operations (like texture sampling)
        let scratch_base = next_local_idx;
        locals_types.push((32, ValType::I32)); // 32 scratch i32s
        locals_types.push((32, ValType::F32)); // 32 scratch f32s
        next_local_idx += 64;

        // Create function body
        let mut wasm_func = Function::new(locals_types);

        if let Some(_) = entry_point {
            // Set globals from arguments
            // 0: attr, 1: uniform, 2: varying, 3: private, 4: textures
            // Arguments are (type, attr, uniform, varying, private, texture)
            for i in 0..5 {
                wasm_func.instruction(&Instruction::LocalGet(i as u32 + 1));
                wasm_func.instruction(&Instruction::GlobalSet(i as u32));
            }
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
                &local_offsets,
                &call_result_locals,
                stage,
                &typifier,
                &self.naga_function_map,
                &argument_local_offsets,
                is_entry_point,
                scratch_base,
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
