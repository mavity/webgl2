//! Core WASM code generation logic

use super::{output_layout, BackendError, CompileConfig, MemoryLayout, WasmBackend, WasmModule};
use naga::{front::Typifier, valid::ModuleInfo, Module};
use std::collections::HashMap;
use wasm_encoder::{
    CodeSection, CustomSection, ExportKind, ExportSection, Function, FunctionSection,
    ImportSection, Instruction, MemoryType, TypeSection, ValType,
};

/// Compile a Naga module to WASM bytecode
pub(super) fn compile_module(
    backend: &WasmBackend,
    config: CompileConfig,
    name: Option<&str>,
) -> Result<WasmModule, BackendError> {
    tracing::info!(
        "Starting WASM compilation for module with {} entry points",
        config.module.entry_points.len()
    );

    let mut compiler = Compiler::new(backend, config, name);
    compiler.compile()?;

    Ok(compiler.finish())
}

/// Internal compiler state
struct Compiler<'a> {
    _backend: &'a WasmBackend,
    _info: &'a ModuleInfo,
    _source: &'a str,
    stage: naga::ShaderStage,
    attribute_locations: &'a HashMap<String, u32>,
    uniform_locations: &'a HashMap<String, u32>,
    varying_locations: &'a HashMap<String, u32>,
    varying_types: &'a HashMap<String, (u8, u32)>,
    uniform_types: &'a HashMap<String, (u8, u32)>,
    attribute_types: &'a HashMap<String, (u8, u32)>,
    name: Option<&'a str>,
    module: &'a Module,

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
    function_abis: HashMap<naga::Handle<naga::Function>, super::function_abi::FunctionABI>,
    global_offsets: HashMap<naga::Handle<naga::GlobalVariable>, (u32, u32)>,
    debug_step_idx: Option<u32>,
    /// Index of the low-level texel fetch host import (env.texture_texel_fetch)
    texture_texel_fetch_idx: Option<u32>,
    /// Index of the emitted module-local helper function `__webgl_texture_sample`
    webgl_texture_sample_idx: Option<u32>,

    // Debug info (if enabled)
    debug_generator: Option<super::debug::DwarfGenerator>,
}

impl<'a> Compiler<'a> {
    fn new(backend: &'a WasmBackend, config: CompileConfig<'a>, name: Option<&'a str>) -> Self {
        // DWARF generation is currently a placeholder/stub in the backend.
        // It is not used for coverage or runtime debugging.
        let debug_generator = None;

        Self {
            _backend: backend,
            _info: config.info,
            _source: config.source,
            stage: config.stage,
            attribute_locations: config.attribute_locations,
            uniform_locations: config.uniform_locations,
            varying_locations: config.varying_locations,
            varying_types: config.varying_types,
            uniform_types: config.uniform_types,
            attribute_types: config.attribute_types,
            name,
            module: config.module,
            types: TypeSection::new(),
            imports: ImportSection::new(),
            functions: FunctionSection::new(),
            globals: wasm_encoder::GlobalSection::new(),
            code: CodeSection::new(),
            exports: ExportSection::new(),
            entry_points: HashMap::new(),
            function_count: 0,
            naga_function_map: HashMap::new(),
            function_abis: HashMap::new(),
            global_offsets: HashMap::new(),
            debug_step_idx: None,
            texture_texel_fetch_idx: None,
            webgl_texture_sample_idx: None,
            debug_generator,
        }
    }

    fn emit_texture_sample_helper(&mut self) {
        let type_index = self.types.len();
        self.types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::F32, ValType::F32],
            vec![ValType::F32, ValType::F32, ValType::F32, ValType::F32],
        );

        let func_idx = self.function_count;
        self.functions.function(type_index);
        self.function_count += 1;
        self.texture_texel_fetch_idx = Some(func_idx);

        let mut func = Function::new(vec![
            (6, ValType::I32), // locals 4..9: desc_addr, width, height, data_ptr, texel_x, texel_y
        ]);

        // 1. Compute descriptor address: ptr + unit * 32
        func.instruction(&Instruction::LocalGet(0));
        func.instruction(&Instruction::LocalGet(1));
        func.instruction(&Instruction::I32Const(32));
        func.instruction(&Instruction::I32Mul);
        func.instruction(&Instruction::I32Add);
        func.instruction(&Instruction::LocalSet(4));

        // 2. Load descriptor fields
        func.instruction(&Instruction::LocalGet(4));
        func.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        }));
        func.instruction(&Instruction::LocalSet(5)); // width

        func.instruction(&Instruction::LocalGet(4));
        func.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
            offset: 4,
            align: 2,
            memory_index: 0,
        }));
        func.instruction(&Instruction::LocalSet(6)); // height

        func.instruction(&Instruction::LocalGet(4));
        func.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
            offset: 8,
            align: 2,
            memory_index: 0,
        }));
        func.instruction(&Instruction::LocalSet(7)); // data_ptr

        // 3. Compute texel coords
        // texel_x = clamp(floor(u * width), 0, width - 1)
        func.instruction(&Instruction::LocalGet(2));
        func.instruction(&Instruction::LocalGet(5));
        func.instruction(&Instruction::F32ConvertI32S);
        func.instruction(&Instruction::F32Mul);
        func.instruction(&Instruction::F32Floor);
        func.instruction(&Instruction::I32TruncF32S);
        func.instruction(&Instruction::LocalSet(8));

        // Clamp x
        func.instruction(&Instruction::LocalGet(8));
        func.instruction(&Instruction::I32Const(0));
        func.instruction(&Instruction::LocalGet(8));
        func.instruction(&Instruction::I32Const(0));
        func.instruction(&Instruction::I32GtS);
        func.instruction(&Instruction::Select);
        func.instruction(&Instruction::LocalSet(8));

        func.instruction(&Instruction::LocalGet(8));
        func.instruction(&Instruction::LocalGet(5));
        func.instruction(&Instruction::I32Const(1));
        func.instruction(&Instruction::I32Sub);
        func.instruction(&Instruction::LocalGet(8));
        func.instruction(&Instruction::LocalGet(5));
        func.instruction(&Instruction::I32Const(1));
        func.instruction(&Instruction::I32Sub);
        func.instruction(&Instruction::I32LtS);
        func.instruction(&Instruction::Select);
        func.instruction(&Instruction::LocalSet(8));

        // texel_y = clamp(floor(v * height), 0, height - 1)
        func.instruction(&Instruction::LocalGet(3));
        func.instruction(&Instruction::LocalGet(6));
        func.instruction(&Instruction::F32ConvertI32S);
        func.instruction(&Instruction::F32Mul);
        func.instruction(&Instruction::F32Floor);
        func.instruction(&Instruction::I32TruncF32S);
        func.instruction(&Instruction::LocalSet(9));

        // Clamp y
        func.instruction(&Instruction::LocalGet(9));
        func.instruction(&Instruction::I32Const(0));
        func.instruction(&Instruction::LocalGet(9));
        func.instruction(&Instruction::I32Const(0));
        func.instruction(&Instruction::I32GtS);
        func.instruction(&Instruction::Select);
        func.instruction(&Instruction::LocalSet(9));

        func.instruction(&Instruction::LocalGet(9));
        func.instruction(&Instruction::LocalGet(6));
        func.instruction(&Instruction::I32Const(1));
        func.instruction(&Instruction::I32Sub);
        func.instruction(&Instruction::LocalGet(9));
        func.instruction(&Instruction::LocalGet(6));
        func.instruction(&Instruction::I32Const(1));
        func.instruction(&Instruction::I32Sub);
        func.instruction(&Instruction::I32LtS);
        func.instruction(&Instruction::Select);
        func.instruction(&Instruction::LocalSet(9));

        // 4. Compute byte offset: (texel_y * width + texel_x) * 4
        func.instruction(&Instruction::LocalGet(9));
        func.instruction(&Instruction::LocalGet(5));
        func.instruction(&Instruction::I32Mul);
        func.instruction(&Instruction::LocalGet(8));
        func.instruction(&Instruction::I32Add);
        func.instruction(&Instruction::I32Const(4));
        func.instruction(&Instruction::I32Mul);
        func.instruction(&Instruction::LocalGet(7));
        func.instruction(&Instruction::I32Add);
        func.instruction(&Instruction::LocalSet(4)); // Reuse 4 as address

        // 5. Load RGBA and normalize
        for i in 0..4 {
            func.instruction(&Instruction::LocalGet(4));
            func.instruction(&Instruction::I32Load8U(wasm_encoder::MemArg {
                offset: i as u64,
                align: 0,
                memory_index: 0,
            }));
            func.instruction(&Instruction::F32ConvertI32U);
            func.instruction(&Instruction::F32Const(255.0));
            func.instruction(&Instruction::F32Div);
        }

        func.instruction(&Instruction::End);
        self.code.function(&func);
    }

    fn compile(&mut self) -> Result<(), BackendError> {
        // Import memory from host
        self.imports.import(
            "env",
            "memory",
            MemoryType {
                minimum: 10, // 640KB
                maximum: None,
                memory64: false,
                shared: false,
                page_size_log2: None,
            },
        );

        // Import debug_step if shader debugging is enabled for this backend
        if self._backend.config.debug_shaders {
            self.imports.import(
                "env",
                "debug_step",
                wasm_encoder::EntityType::Function(self.types.len()),
            );
            // Signature: (line: i32, func_id: i32, result_ptr: i32) -> ()
            self.types
                .ty()
                .function(vec![ValType::I32, ValType::I32, ValType::I32], vec![]);
            self.debug_step_idx = Some(self.function_count);
            self.function_count += 1;
        }

        // Emit the module-local texture sampling helper
        self.emit_texture_sample_helper();

        // Define 6 globals for base pointers
        // 0: attr, 1: uniform, 2: varying, 3: private, 4: textures, 5: frame_sp
        for _ in 0..6 {
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
        let _attr_offset = 0;

        // First pass: find gl_Position and put it at the start of varying buffer
        for (handle, var) in self.module.global_variables.iter() {
            let is_position = if let Some(name) = &var.name {
                name == "gl_Position" || name == "gl_Position_1"
            } else {
                false
            };

            if is_position {
                self.global_offsets
                    .insert(handle, (0, output_layout::VARYING_PTR_GLOBAL));
                varying_offset = 16; // gl_Position is vec4 (16 bytes)
                break;
            }
        }

        // We need to know which variables are inputs/outputs
        // For now, let's look at the first entry point
        if let Some(ep) = self.module.entry_points.first() {
            for _arg in &ep.function.arguments {
                // Naga GLSL frontend often uses Private for inputs
                // We can't easily link them back to GlobalVariable handles here
                // without more complex analysis.
            }
        }

        for (handle, var) in self.module.global_variables.iter() {
            if self.global_offsets.contains_key(&handle) {
                continue;
            }
            let size = super::types::type_size(&self.module.types[var.ty].inner).unwrap_or(4);

            let (offset, base_ptr) = match var.space {
                naga::AddressSpace::Uniform | naga::AddressSpace::Handle => {
                    if let Some(name) = &var.name {
                        if let Some(&loc) = self.uniform_locations.get(name) {
                            output_layout::compute_uniform_offset(loc)
                        } else {
                            (0, output_layout::UNIFORM_PTR_GLOBAL)
                        }
                    } else {
                        (0, output_layout::UNIFORM_PTR_GLOBAL)
                    }
                }
                naga::AddressSpace::Private | naga::AddressSpace::Function => {
                    // Check if it's an output in FS
                    let is_output = if self.stage == naga::ShaderStage::Fragment {
                        if let Some(name) = &var.name {
                            let n = name.as_str();
                            n == "color"
                                || n == "gl_FragColor"
                                || n == "fragColor"
                                || n == "gl_FragColor_1"
                                || n.ends_with("Color")
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if is_output {
                        (0, output_layout::PRIVATE_PTR_GLOBAL)
                    } else if let Some(name) = &var.name {
                        if let Some(&loc) = self.attribute_locations.get(name) {
                            output_layout::compute_input_offset(loc, naga::ShaderStage::Vertex)
                        } else if let Some(&loc) = self.varying_locations.get(name) {
                            output_layout::compute_input_offset(loc, naga::ShaderStage::Fragment)
                        } else {
                            return Err(BackendError::InternalError(format!(
                                "Varying '{}' has no assigned location",
                                name
                            )));
                        }
                    } else {
                        return Err(BackendError::InternalError(
                            "Unmapped anonymous global in Private/Function address space"
                                .to_string(),
                        ));
                    }
                }
                // Handle explicit In/Out address spaces (used in newer Naga versions)
                // Note: We can't match directly on AddressSpace::In/Out if they don't exist in this version,
                // but we can use a catch-all with a check if we knew the enum variants.
                // Since we are in a match, we'll assume anything else that looks like an input/output
                // falls here.
                _ => {
                    // Check if it's an output in FS (AddressSpace::Out)
                    let is_fs_output = if self.stage == naga::ShaderStage::Fragment {
                        // If it's AddressSpace::Out (which falls here), it's an output
                        // We can't check the enum variant easily if it's not imported or available,
                        // but we can infer from context or just assume non-uniform/private globals in FS are outputs?
                        // Actually, let's rely on the fact that inputs in FS are varyings, and outputs are color.

                        // If it has a location binding, it might be an output
                        // Note: GlobalVariable binding is Option<ResourceBinding>, which doesn't have Location.
                        // Location bindings are on FunctionArgument/FunctionResult.
                        // So we can't check location here for globals.

                        // Fallback to name check
                        if let Some(name) = &var.name {
                            let n = name.as_str();
                            n == "color"
                                || n == "fragColor"
                                || n == "gl_FragColor"
                                || n.ends_with("Color")
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if is_fs_output {
                        (0, output_layout::PRIVATE_PTR_GLOBAL) // private_ptr for FS output
                    } else {
                        // Otherwise assume varying (VS output or FS input)
                        let o = varying_offset;
                        varying_offset += size;
                        varying_offset = (varying_offset + 3) & !3;
                        (o, 2)
                    }
                }
            };
            self.global_offsets.insert(handle, (offset, base_ptr));
        }

        // Compile all internal functions first
        for (handle, func) in self.module.functions.iter() {
            let func_idx = self.compile_function(func, None, Some(handle))?;
            self.naga_function_map.insert(handle, func_idx);
        }

        // Compile each entry point
        for (idx, entry_point) in self.module.entry_points.iter().enumerate() {
            self.compile_entry_point(entry_point, idx)?;
        }
        Ok(())
    }

    fn compile_function(
        &mut self,
        func: &naga::Function,
        entry_point: Option<&naga::EntryPoint>,
        func_handle: Option<naga::Handle<naga::Function>>,
    ) -> Result<u32, BackendError> {
        let func_idx = self.function_count;
        self.function_count += 1;

        let mut params: Vec<ValType> = vec![];
        let mut results: Vec<ValType> = vec![];
        let mut argument_local_offsets = HashMap::new();
        let mut current_param_idx = 0;

        if entry_point.is_some() {
            // Entry point signature: (type, attr_ptr, uniform_ptr, varying_ptr, private_ptr, texture_ptr) -> ()
            params = vec![
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
            ];
            current_param_idx = 6;
        } else {
            // Internal function - use FunctionABI for signature
            let param_types: Vec<_> = func.arguments.iter().map(|arg| arg.ty).collect();
            let result_type = func.result.as_ref().map(|r| r.ty);

            let abi =
                super::function_abi::FunctionABI::compute(self.module, &param_types, result_type)
                    .map_err(|e| {
                    BackendError::UnsupportedFeature(format!("FunctionABI error: {:?}", e))
                })?;

            // Store ABI for call lowering
            if let Some(fh) = func_handle {
                self.function_abis.insert(fh, abi.clone());
            }

            params = abi.param_valtypes();
            results = abi.result_valtypes();

            // Map argument handles to parameter indices for flattened params
            // For now, simple sequential mapping (Frame params need special handling)
            for i in 0..func.arguments.len() {
                argument_local_offsets.insert(i as u32, current_param_idx);
                current_param_idx += params.len() as u32; // Simplified, should be per-arg
            }
        }

        let type_idx = self.types.len();
        self.types.ty().function(params, results);
        self.functions.function(type_idx);

        let mut typifier = Typifier::new();
        let resolve_ctx = naga::proc::ResolveContext::with_locals(
            self.module,
            &func.local_variables,
            &func.arguments,
        );
        for (handle, _expr) in func.expressions.iter() {
            // crate::js_print(&format!("DEBUG: Expr {:?}: {:?}", handle, expr));
            typifier
                .grow(handle, &func.expressions, &resolve_ctx)
                .map_err(|e| {
                    BackendError::UnsupportedFeature(format!("Typifier error: {:?}", e))
                })?;
        }

        // Calculate local variable offsets
        let mut local_offsets = HashMap::new();
        let mut current_local_offset = 0;
        for (handle, var) in func.local_variables.iter() {
            let size = super::types::type_size(&self.module.types[var.ty].inner).unwrap_or(4);
            local_offsets.insert(handle, current_local_offset);
            current_local_offset += size;
        }

        // Attempt to discover local variables that are initialized from globals (pointer origin tracing).
        // We scan the function body for Store statements that assign a global-derived pointer to a local.
        // Make this robust to wrapped pointers like AccessIndex/Access and wrapped values (As/Load/AccessIndex/Swizzle).
        let mut local_origins: HashMap<
            naga::Handle<naga::LocalVariable>,
            naga::Handle<naga::GlobalVariable>,
        > = HashMap::new();
        for (stmt, _span) in func.body.span_iter() {
            if let naga::Statement::Store { pointer, value } = stmt {
                // First, find if the store pointer resolves to a LocalVariable (walk AccessIndex/Access wrappers)
                let mut ptr_cur = *pointer;
                let mut local_handle_opt: Option<naga::Handle<naga::LocalVariable>> = None;
                loop {
                    match func.expressions[ptr_cur] {
                        naga::Expression::LocalVariable(lh) => {
                            local_handle_opt = Some(lh);
                            break;
                        }
                        naga::Expression::AccessIndex { base: b, .. } => {
                            ptr_cur = b;
                        }
                        naga::Expression::Access { base: b, .. } => {
                            ptr_cur = b;
                        }
                        _ => break,
                    }
                }

                if let Some(local_handle) = local_handle_opt {
                    // Walk the value expression to find an originating GlobalVariable, allowing several wrapper forms.
                    let mut cur = *value;
                    // Prevent infinite loops by tracking visited nodes
                    let mut visited = std::collections::HashSet::new();
                    loop {
                        if !visited.insert(cur) {
                            break; // cycle
                        }
                        match func.expressions[cur] {
                            naga::Expression::GlobalVariable(g) => {
                                local_origins.insert(local_handle, g);
                                break;
                            }
                            naga::Expression::Load { pointer: p } => {
                                cur = p;
                            }
                            naga::Expression::AccessIndex { base: b, .. } => {
                                cur = b;
                            }
                            naga::Expression::Access { base: b, .. } => {
                                cur = b;
                            }
                            naga::Expression::As { expr, .. } => {
                                cur = expr;
                            }
                            naga::Expression::Swizzle { vector, .. } => {
                                cur = vector;
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                }
            }
        }

        let mut call_result_locals = HashMap::new();
        let mut locals_types = vec![];
        let mut next_local_idx = current_param_idx;

        // Add scratch F32 locals first so float temporaries land in f32-typed locals.
        // scratch F32 region starts at param_count
        let scratch_base = next_local_idx;
        locals_types.push((32, ValType::F32)); // 32 scratch f32s
        next_local_idx += 32;

        // Track CallResult expressions and their declaration indices
        let mut call_result_decl_indices: Vec<(naga::Handle<naga::Expression>, u32, usize)> =
            Vec::new();

        // Map CallResult expressions to WASM locals (place them after scratch F32 region)
        for (handle, expr) in func.expressions.iter() {
            if let naga::Expression::CallResult(func_handle) = expr {
                let called_func = &self.module.functions[*func_handle];
                if let Some(ret) = &called_func.result {
                    let types = super::types::naga_to_wasm_types(&self.module.types[ret.ty].inner)?;
                    let decl_idx = next_local_idx;
                    let num_components = types.len();
                    call_result_decl_indices.push((handle, decl_idx, num_components));
                    for _ in 0..num_components {
                        locals_types.push((1, ValType::F32));
                    }
                    next_local_idx += num_components as u32;
                }
            }
        }

        // Now add scratch I32 locals after the F32 regions
        locals_types.push((32, ValType::I32)); // 32 scratch i32s
        next_local_idx += 32;

        // Add explicit swap locals at the END to preserve existing indices
        // These will be used by store_components_to_memory instead of scanning
        let swap_i32_local = next_local_idx;
        locals_types.push((1, ValType::I32)); // swap_i32_local
        next_local_idx += 1;

        let swap_f32_local = next_local_idx;
        locals_types.push((1, ValType::F32)); // swap_f32_local
        next_local_idx += 1;

        // Add frame temp local (conservative allocation for Phase 4)
        let frame_temp_local = next_local_idx;
        locals_types.push((1, ValType::I32)); // frame_temp
        next_local_idx += 1;

        // Suppress unused warning for last increment
        let _ = next_local_idx;

        // Build a flattened local types vector (locals only, not params) for
        // downstream logic that needs to know a specific local's declared type.
        let mut flattened_local_types: Vec<ValType> = Vec::new();
        for (count, vtype) in &locals_types {
            for _ in 0..*count {
                flattened_local_types.push(*vtype);
            }
        }

        // wasm-encoder preserves local declaration order.
        // Simply map handle to declaration index.
        for (handle, decl_idx, _num_components) in call_result_decl_indices {
            call_result_locals.insert(handle, decl_idx);
        }

        eprintln!(
            "[debug] flattened_local_types (first 64): {:?}",
            &flattened_local_types[..std::cmp::min(flattened_local_types.len(), 64)]
        );

        // Create function body
        let mut wasm_func = Function::new(locals_types);

        if entry_point.is_some() {
            // Set globals from arguments
            // 0: attr, 1: uniform, 2: varying, 3: private, 4: textures
            // Arguments are (type, attr, uniform, varying, private, texture)
            for i in 0..5 {
                wasm_func.instruction(&Instruction::LocalGet(i as u32 + 1));
                wasm_func.instruction(&Instruction::GlobalSet(i as u32));
            }

            // Initialize frame stack pointer to base address
            wasm_func.instruction(&Instruction::I32Const(
                output_layout::FRAME_STACK_BASE as i32,
            ));
            wasm_func.instruction(&Instruction::GlobalSet(output_layout::FRAME_SP_GLOBAL));
        }

        let stage = self.stage;
        let is_entry_point = entry_point.is_some();

        // Translate statements
        let mut ctx = super::TranslationContext {
            func,
            module: self.module,
            source: self._source,
            wasm_func: &mut wasm_func,
            global_offsets: &self.global_offsets,
            local_offsets: &local_offsets,
            call_result_locals: &call_result_locals,
            stage,
            debug_shaders: self._backend.config.debug_shaders,
            debug_step_idx: self.debug_step_idx,
            typifier: &typifier,
            naga_function_map: &self.naga_function_map,
            function_abis: &self.function_abis,
            argument_local_offsets: &argument_local_offsets,
            attribute_locations: self.attribute_locations,
            uniform_locations: self.uniform_locations,
            varying_locations: self.varying_locations,
            varying_types: self.varying_types,
            uniform_types: self.uniform_types,
            attribute_types: self.attribute_types,
            local_origins: &local_origins,
            is_entry_point,
            scratch_base,
            swap_i32_local,
            swap_f32_local,
            // Local types and parameter count for type-aware lowering
            local_types: &flattened_local_types,
            param_count: current_param_idx,
            texture_texel_fetch_idx: self.texture_texel_fetch_idx,
            webgl_texture_sample_idx: self.webgl_texture_sample_idx,
            frame_temp_idx: Some(frame_temp_local),
        };

        for (stmt, span) in func.body.span_iter() {
            super::control_flow::translate_statement(stmt, span, &mut ctx)?;
        }

        wasm_func.instruction(&Instruction::End);
        self.code.function(&wasm_func);

        // Export internal functions in debug mode
        if entry_point.is_none() && self._backend.config.debug_shaders {
            self.exports
                .export(&format!("func_{}", func_idx), ExportKind::Func, func_idx);
        }

        Ok(func_idx)
    }

    fn compile_entry_point(
        &mut self,
        entry_point: &naga::EntryPoint,
        _index: usize,
    ) -> Result<(), BackendError> {
        let func_idx = self.compile_function(&entry_point.function, Some(entry_point), None)?;

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

        // Generate JS stub if enabled
        let debug_stub = if self._backend.config.debug_shaders {
            let generator =
                super::debug::JsStubGenerator::new(self._source, self.module, self.name);
            Some(generator.generate())
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
            debug_stub,
            entry_points: self.entry_points,
            memory_layout: MemoryLayout::default(),
        }
    }
}
