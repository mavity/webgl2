//! Core WASM code generation logic

use super::{output_layout, BackendError, CompileConfig, MemoryLayout, WasmBackend, WasmModule};
use naga::{front::Typifier, valid::ModuleInfo, Module};
use std::collections::HashMap;
use wasm_encoder::{
    BlockType, CodeSection, CustomSection, ExportKind, ExportSection, Function, FunctionSection,
    ImportSection, Instruction, MemoryType, NameMap, NameSection, TypeSection, ValType,
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

    // Run preparation pass to compute all function ABIs and manifests
    let registry = super::functions::prep_module(config.module, config.info);

    let mut compiler = Compiler::new(backend, config, name, &registry);
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
    uniform_map: HashMap<(u32, u32), (u32, output_layout::UniformLayout)>,
    entry_point_name: Option<&'a str>,
    exported_names: std::collections::HashSet<String>,
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
    function_registry: &'a super::functions::FunctionRegistry,
    global_offsets: HashMap<naga::Handle<naga::GlobalVariable>, (u32, u32)>,
    debug_step_idx: Option<u32>,
    /// Specialized samplers
    webgl_sampler_2d_idx: Option<u32>,
    webgl_sampler_3d_idx: Option<u32>,
    /// Index of the emitted module-local helper function `__webgl_image_load`
    webgl_image_load_idx: Option<u32>,
    /// Mapping of Naga math functions to their imported WASM function indices
    math_import_map: HashMap<naga::MathFunction, u32>,

    // Debug info (if enabled)
    debug_generator: Option<super::debug::DwarfGenerator>,
}

impl<'a> Compiler<'a> {
    fn new(
        backend: &'a WasmBackend,
        config: CompileConfig<'a>,
        name: Option<&'a str>,
        function_registry: &'a super::functions::FunctionRegistry,
    ) -> Self {
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
            uniform_map: output_layout::get_uniform_map(config.module, config.info, config.stage),
            entry_point_name: config.entry_point,
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
            function_registry,
            global_offsets: HashMap::new(),
            exported_names: std::collections::HashSet::new(),
            debug_step_idx: None,
            webgl_sampler_2d_idx: None,
            webgl_sampler_3d_idx: None,
            webgl_image_load_idx: None,
            math_import_map: HashMap::new(),
            debug_generator,
        }
    }

    fn has_image_sampling(&self) -> (bool, bool) {
        let mut has_2d = false;
        let mut has_3d = false;
        // Check all types in the module for images
        for (_, ty) in self.module.types.iter() {
            if let naga::TypeInner::Image { dim, .. } = ty.inner {
                match dim {
                    naga::ImageDimension::D2 => has_2d = true,
                    naga::ImageDimension::D3 => has_3d = true,
                    _ => {}
                }
            }
        }
        (has_2d, has_3d)
    }

    fn has_image_load(&self) -> bool {
        let check_expressions = |func: &naga::Function| {
            func.expressions
                .iter()
                .any(|(_, expr)| matches!(expr, naga::Expression::ImageLoad { .. }))
        };

        self.module
            .functions
            .iter()
            .any(|(_, f)| check_expressions(f))
            || self
                .module
                .entry_points
                .iter()
                .any(|ep| check_expressions(&ep.function))
    }

    fn emit_image_load_helper(&mut self) {
        let type_index = self.types.len();
        self.types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
            vec![ValType::F32, ValType::F32, ValType::F32, ValType::F32],
        );

        let func_idx = self.function_count;
        self.functions.function(type_index);
        self.function_count += 1;
        self.webgl_image_load_idx = Some(func_idx);

        let mut func = Function::new(vec![
            (7, ValType::I32), // locals 4..10: desc_addr, width, height, data_ptr, bpp, layout, address
        ]);

        let l_desc_addr = 4;
        let l_width = 5;
        let l_height = 6;
        let l_data_ptr = 7;
        let l_bpp = 8;
        let l_layout = 9;
        let l_addr = 10;

        // 1. Descriptor address is passed directly in arg 1
        func.instruction(&Instruction::LocalGet(1));
        func.instruction(&Instruction::LocalSet(l_desc_addr));

        // 2. Load descriptor fields
        func.instruction(&Instruction::LocalGet(l_desc_addr));
        func.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
            offset: output_layout::TEX_WIDTH_OFFSET,
            align: 2,
            memory_index: 0,
        }));
        func.instruction(&Instruction::LocalSet(l_width));

        func.instruction(&Instruction::LocalGet(l_desc_addr));
        func.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
            offset: output_layout::TEX_HEIGHT_OFFSET,
            align: 2,
            memory_index: 0,
        }));
        func.instruction(&Instruction::LocalSet(l_height));

        func.instruction(&Instruction::LocalGet(l_desc_addr));
        func.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
            offset: output_layout::TEX_DATA_PTR_OFFSET,
            align: 2,
            memory_index: 0,
        }));
        func.instruction(&Instruction::LocalSet(l_data_ptr));

        func.instruction(&Instruction::LocalGet(l_desc_addr));
        func.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
            offset: output_layout::TEX_BPP_OFFSET,
            align: 2,
            memory_index: 0,
        }));
        func.instruction(&Instruction::LocalSet(l_bpp));

        func.instruction(&Instruction::LocalGet(l_desc_addr));
        func.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
            offset: output_layout::TEX_LAYOUT_OFFSET,
            align: 2,
            memory_index: 0,
        }));
        func.instruction(&Instruction::LocalSet(l_layout));

        // 3. Compute byte offset
        func.instruction(&Instruction::LocalGet(l_layout));
        func.instruction(&Instruction::I32Const(1)); // Tiled8x8
        func.instruction(&Instruction::I32Eq);
        func.instruction(&Instruction::If(wasm_encoder::BlockType::Empty));
        // Tiled8x8
        func.instruction(&Instruction::LocalGet(l_width));
        func.instruction(&Instruction::I32Const(7));
        func.instruction(&Instruction::I32Add);
        func.instruction(&Instruction::I32Const(3));
        func.instruction(&Instruction::I32ShrU); // tiles_w

        func.instruction(&Instruction::LocalGet(3)); // y
        func.instruction(&Instruction::I32Const(3));
        func.instruction(&Instruction::I32ShrU); // tile_y
        func.instruction(&Instruction::I32Mul); // tile_y * tiles_w

        func.instruction(&Instruction::LocalGet(2)); // x
        func.instruction(&Instruction::I32Const(3));
        func.instruction(&Instruction::I32ShrU); // tile_x
        func.instruction(&Instruction::I32Add); // tile_idx

        func.instruction(&Instruction::I32Const(6));
        func.instruction(&Instruction::I32Shl); // tile_idx * 64

        func.instruction(&Instruction::LocalGet(3)); // y
        func.instruction(&Instruction::I32Const(7));
        func.instruction(&Instruction::I32And); // inner_y
        func.instruction(&Instruction::I32Const(3));
        func.instruction(&Instruction::I32Shl); // inner_y * 8

        func.instruction(&Instruction::LocalGet(2)); // x
        func.instruction(&Instruction::I32Const(7));
        func.instruction(&Instruction::I32And); // inner_x

        func.instruction(&Instruction::I32Add); // inner_idx
        func.instruction(&Instruction::I32Add); // total_idx
        func.instruction(&Instruction::LocalSet(l_addr));
        func.instruction(&Instruction::Else);
        // Linear
        func.instruction(&Instruction::LocalGet(3)); // y
        func.instruction(&Instruction::LocalGet(l_width));
        func.instruction(&Instruction::I32Mul);
        func.instruction(&Instruction::LocalGet(2)); // x
        func.instruction(&Instruction::I32Add);
        func.instruction(&Instruction::LocalSet(l_addr));
        func.instruction(&Instruction::End);

        func.instruction(&Instruction::LocalGet(l_addr));
        func.instruction(&Instruction::LocalGet(l_bpp));
        func.instruction(&Instruction::I32Mul);
        func.instruction(&Instruction::LocalGet(l_data_ptr)); // data_ptr
        func.instruction(&Instruction::I32Add);
        func.instruction(&Instruction::LocalSet(l_addr)); // address

        // 4. Load RGBA
        for i in 0..4 {
            func.instruction(&Instruction::LocalGet(l_addr));
            func.instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                offset: i * 4,
                align: 2,
                memory_index: 0,
            }));
        }

        self.code.function(&func);
    }

    fn emit_sampler(&mut self, dim: naga::ImageDimension) -> u32 {
        let is_3d = dim == naga::ImageDimension::D3;
        // Params: 0: texture_desc_addr, 1: sampler_desc_addr, 2: u, 3: v, [4: w]
        let mut params = vec![ValType::I32, ValType::I32, ValType::F32, ValType::F32];
        if is_3d {
            params.push(ValType::F32);
        }

        let type_index = self.types.len();
        self.types.ty().function(
            params,
            vec![ValType::F32, ValType::F32, ValType::F32, ValType::F32],
        );

        let func_idx = self.function_count;
        self.functions.function(type_index);
        self.function_count += 1;

        let p_count = if is_3d { 5 } else { 4 };
        let mut func = Function::new(vec![
            (24, ValType::I32), // locals: width... tz, addr, ws, wt, wr, layout, minf, magf, x0, y0, x1, y1, z0, z1, loop_cnt, temp
            (11, ValType::F32), // locals: res_r, res_g, res_b, res_a, wx, wy, wz, tmp_r, tmp_g, tmp_b, tmp_a
        ]);

        let l_tex_desc = 0;
        let l_sam_desc = 1;

        let l_width = p_count;
        let l_height = p_count + 1;
        let l_ptr = p_count + 2;
        let l_depth = p_count + 3;
        let l_format = p_count + 4;
        let l_bpp = p_count + 5;
        let l_tx = p_count + 6;
        let l_ty = p_count + 7;
        let l_tz = p_count + 8;
        let l_addr = p_count + 9;
        let l_wrap_s = p_count + 10;
        let l_wrap_t = p_count + 11;
        let l_wrap_r = p_count + 12;
        let l_layout = p_count + 13;
        let l_min_filter = p_count + 14;
        let l_mag_filter = p_count + 15;
        let l_x0 = p_count + 16;
        let l_y0 = p_count + 17;
        let l_x1 = p_count + 18;
        let l_y1 = p_count + 19;
        let l_z0 = p_count + 20;
        let l_z1 = p_count + 21;
        let l_loop_cnt = p_count + 22;
        // p_count + 23 is temp

        let l_res_r = p_count + 24;
        let l_res_g = p_count + 25;
        let l_res_b = p_count + 26;
        let l_res_a = p_count + 27;
        let l_wx = p_count + 28;
        let l_wy = p_count + 29;
        let l_wz = p_count + 30;
        let l_tmp_a = p_count + 34;

        // 1. Load texture metadata from l_tex_desc
        {
            let load_tex = |func: &mut Function, offset: u64, local: u32| {
                func.instruction(&Instruction::LocalGet(l_tex_desc));
                func.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                    offset,
                    align: 2,
                    memory_index: 0,
                }));
                func.instruction(&Instruction::LocalSet(local));
            };

            load_tex(&mut func, output_layout::TEX_WIDTH_OFFSET, l_width);
            load_tex(&mut func, output_layout::TEX_HEIGHT_OFFSET, l_height);
            load_tex(&mut func, output_layout::TEX_DATA_PTR_OFFSET, l_ptr);
            load_tex(&mut func, output_layout::TEX_DEPTH_OFFSET, l_depth);
            load_tex(&mut func, output_layout::TEX_FORMAT_OFFSET, l_format);
            load_tex(&mut func, output_layout::TEX_BPP_OFFSET, l_bpp);
        }

        // 2. Load sampler metadata from l_sam_desc
        {
            func.instruction(&Instruction::LocalGet(l_sam_desc));
            func.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                offset: output_layout::TEX_WRAP_S_OFFSET,
                align: 2,
                memory_index: 0,
            }));
            func.instruction(&Instruction::LocalSet(l_wrap_s));

            func.instruction(&Instruction::LocalGet(l_sam_desc));
            func.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                offset: output_layout::TEX_WRAP_T_OFFSET,
                align: 2,
                memory_index: 0,
            }));
            func.instruction(&Instruction::LocalSet(l_wrap_t));

            func.instruction(&Instruction::LocalGet(l_sam_desc));
            func.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                offset: output_layout::TEX_WRAP_R_OFFSET,
                align: 2,
                memory_index: 0,
            }));
            func.instruction(&Instruction::LocalSet(l_wrap_r));

            func.instruction(&Instruction::LocalGet(l_sam_desc));
            func.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                offset: output_layout::TEX_LAYOUT_OFFSET,
                align: 2,
                memory_index: 0,
            }));
            func.instruction(&Instruction::LocalSet(l_layout));

            // Load filters
            func.instruction(&Instruction::LocalGet(l_sam_desc));
            func.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                offset: output_layout::TEX_MIN_FILTER_OFFSET,
                align: 2,
                memory_index: 0,
            }));
            func.instruction(&Instruction::LocalSet(l_min_filter));

            func.instruction(&Instruction::LocalGet(l_sam_desc));
            func.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                offset: output_layout::TEX_MAG_FILTER_OFFSET,
                align: 2,
                memory_index: 0,
            }));
            func.instruction(&Instruction::LocalSet(l_mag_filter));
        }

        // 3. Compute texel coords and weights
        {
            // Detect if linear filtering is requested
            // 0x2601 = LINEAR
            // 0x2701 = LINEAR_MIPMAP_NEAREST
            // 0x2703 = LINEAR_MIPMAP_LINEAR
            let is_linear = |func: &mut Function, local: u32| {
                func.instruction(&Instruction::LocalGet(local));
                func.instruction(&Instruction::I32Const(0x2601)); // LINEAR
                func.instruction(&Instruction::I32Eq);

                func.instruction(&Instruction::LocalGet(local));
                func.instruction(&Instruction::I32Const(0x2701)); // LINEAR_MIPMAP_NEAREST
                func.instruction(&Instruction::I32Eq);
                func.instruction(&Instruction::I32Or);

                func.instruction(&Instruction::LocalGet(local));
                func.instruction(&Instruction::I32Const(0x2703)); // LINEAR_MIPMAP_LINEAR
                func.instruction(&Instruction::I32Eq);
                func.instruction(&Instruction::I32Or);
            };

            is_linear(&mut func, l_mag_filter);
            is_linear(&mut func, l_min_filter);
            func.instruction(&Instruction::I32Or);

            func.instruction(&Instruction::If(BlockType::Empty));
            let mut compute_linear = |coord_param: u32,
                                      size_local: u32,
                                      x0_local: u32,
                                      x1_local: u32,
                                      w_local: u32,
                                      wrap_local: u32| {
                // tx_raw = coord * size - 0.5
                func.instruction(&Instruction::LocalGet(coord_param));
                func.instruction(&Instruction::LocalGet(size_local));
                func.instruction(&Instruction::F32ConvertI32S);
                func.instruction(&Instruction::F32Mul);
                func.instruction(&Instruction::F32Const(0.5));
                func.instruction(&Instruction::F32Sub);

                // x0 = floor(tx_raw)
                func.instruction(&Instruction::F32Floor);
                func.instruction(&Instruction::LocalTee(l_tmp_a)); // temp use l_tmp_a
                func.instruction(&Instruction::I32TruncF32S);
                func.instruction(&Instruction::LocalSet(x0_local));

                // wx = tx_raw - floor(tx_raw)
                func.instruction(&Instruction::LocalGet(coord_param));
                func.instruction(&Instruction::LocalGet(size_local));
                func.instruction(&Instruction::F32ConvertI32S);
                func.instruction(&Instruction::F32Mul);
                func.instruction(&Instruction::F32Const(0.5));
                func.instruction(&Instruction::F32Sub); // tx_raw
                func.instruction(&Instruction::LocalGet(l_tmp_a)); // floor(tx_raw)
                func.instruction(&Instruction::F32Sub);
                func.instruction(&Instruction::LocalSet(w_local)); // w_local = wx

                // x1 = x0 + 1
                func.instruction(&Instruction::LocalGet(x0_local));
                func.instruction(&Instruction::I32Const(1));
                func.instruction(&Instruction::I32Add);
                func.instruction(&Instruction::LocalSet(x1_local));

                // Wrap/Clamp x0 and x1
                let mut apply_wrap = |local: u32, wrap_local: u32| {
                    // Detect wrap mode
                    func.instruction(&Instruction::LocalGet(wrap_local));
                    func.instruction(&Instruction::I32Const(0x2901)); // GL_REPEAT
                    func.instruction(&Instruction::I32Eq);
                    func.instruction(&Instruction::If(BlockType::Empty));
                    // Repeat: val = (val % size + size) % size
                    func.instruction(&Instruction::LocalGet(local));
                    func.instruction(&Instruction::LocalGet(size_local));
                    func.instruction(&Instruction::I32RemS);
                    func.instruction(&Instruction::LocalGet(size_local));
                    func.instruction(&Instruction::I32Add);
                    func.instruction(&Instruction::LocalGet(size_local));
                    func.instruction(&Instruction::I32RemS);
                    func.instruction(&Instruction::LocalSet(local));
                    func.instruction(&Instruction::Else);
                    // Clamp to Edge (and fallback)
                    func.instruction(&Instruction::LocalGet(local));
                    func.instruction(&Instruction::I32Const(0));
                    func.instruction(&Instruction::LocalGet(local));
                    func.instruction(&Instruction::I32Const(0));
                    func.instruction(&Instruction::I32GtS);
                    func.instruction(&Instruction::Select);
                    func.instruction(&Instruction::LocalSet(local));

                    func.instruction(&Instruction::LocalGet(local));
                    func.instruction(&Instruction::LocalGet(size_local));
                    func.instruction(&Instruction::I32Const(1));
                    func.instruction(&Instruction::I32Sub);
                    func.instruction(&Instruction::LocalGet(local));
                    func.instruction(&Instruction::LocalGet(size_local));
                    func.instruction(&Instruction::I32Const(1));
                    func.instruction(&Instruction::I32Sub);
                    func.instruction(&Instruction::I32LtS);
                    func.instruction(&Instruction::Select);
                    func.instruction(&Instruction::LocalSet(local));
                    func.instruction(&Instruction::End);
                };
                apply_wrap(x0_local, wrap_local);
                apply_wrap(x1_local, wrap_local);
            };
            compute_linear(2, l_width, l_x0, l_x1, l_wx, l_wrap_s);
            compute_linear(3, l_height, l_y0, l_y1, l_wy, l_wrap_t);
            if is_3d {
                compute_linear(4, l_depth, l_z0, l_z1, l_wz, l_wrap_r);
            } else {
                func.instruction(&Instruction::F32Const(0.0));
                func.instruction(&Instruction::LocalSet(l_wz));
            }

            func.instruction(&Instruction::Else);

            // Nearest-neighbor
            let mut compute_nearest =
                |coord_param: u32, size_local: u32, res_local: u32, wrap_local: u32| {
                    func.instruction(&Instruction::LocalGet(coord_param));
                    func.instruction(&Instruction::LocalGet(size_local));
                    func.instruction(&Instruction::F32ConvertI32S);
                    func.instruction(&Instruction::F32Mul);
                    func.instruction(&Instruction::F32Floor);
                    func.instruction(&Instruction::I32TruncF32S);
                    func.instruction(&Instruction::LocalSet(res_local));

                    // Detect wrap mode
                    func.instruction(&Instruction::LocalGet(wrap_local));
                    func.instruction(&Instruction::I32Const(0x2901)); // GL_REPEAT
                    func.instruction(&Instruction::I32Eq);
                    func.instruction(&Instruction::If(BlockType::Empty));
                    // Repeat: val = (val % size + size) % size
                    func.instruction(&Instruction::LocalGet(res_local));
                    func.instruction(&Instruction::LocalGet(size_local));
                    func.instruction(&Instruction::I32RemS);
                    func.instruction(&Instruction::LocalGet(size_local));
                    func.instruction(&Instruction::I32Add);
                    func.instruction(&Instruction::LocalGet(size_local));
                    func.instruction(&Instruction::I32RemS);
                    func.instruction(&Instruction::LocalSet(res_local));
                    func.instruction(&Instruction::Else);
                    // Clamp 0
                    func.instruction(&Instruction::LocalGet(res_local));
                    func.instruction(&Instruction::I32Const(0));
                    func.instruction(&Instruction::LocalGet(res_local));
                    func.instruction(&Instruction::I32Const(0));
                    func.instruction(&Instruction::I32GtS);
                    func.instruction(&Instruction::Select);
                    func.instruction(&Instruction::LocalSet(res_local));

                    // Clamp size-1
                    func.instruction(&Instruction::LocalGet(res_local));
                    func.instruction(&Instruction::LocalGet(size_local));
                    func.instruction(&Instruction::I32Const(1));
                    func.instruction(&Instruction::I32Sub);
                    func.instruction(&Instruction::LocalGet(res_local));
                    func.instruction(&Instruction::LocalGet(size_local));
                    func.instruction(&Instruction::I32Const(1));
                    func.instruction(&Instruction::I32Sub);
                    func.instruction(&Instruction::I32LtS);
                    func.instruction(&Instruction::Select);
                    func.instruction(&Instruction::LocalSet(res_local));
                    func.instruction(&Instruction::End);
                };

            compute_nearest(2, l_width, l_x0, l_wrap_s);
            compute_nearest(3, l_height, l_y0, l_wrap_t);
            if is_3d {
                compute_nearest(4, l_depth, l_z0, l_wrap_r);
            }
            // x1 = x0, y1 = y0, wx = 0, wy = 0, wz = 0
            func.instruction(&Instruction::LocalGet(l_x0));
            func.instruction(&Instruction::LocalSet(l_x1));
            func.instruction(&Instruction::LocalGet(l_y0));
            func.instruction(&Instruction::LocalSet(l_y1));
            if is_3d {
                func.instruction(&Instruction::LocalGet(l_z0));
                func.instruction(&Instruction::LocalSet(l_z1));
            }
            func.instruction(&Instruction::F32Const(0.0));
            func.instruction(&Instruction::LocalSet(l_wx));
            func.instruction(&Instruction::F32Const(0.0));
            func.instruction(&Instruction::LocalSet(l_wy));
            func.instruction(&Instruction::F32Const(0.0));
            func.instruction(&Instruction::LocalSet(l_wz));

            func.instruction(&Instruction::End); // End If Linear
        }

        // 4. Initialize results
        func.instruction(&Instruction::F32Const(0.0));
        func.instruction(&Instruction::LocalSet(l_res_r));
        func.instruction(&Instruction::F32Const(0.0));
        func.instruction(&Instruction::LocalSet(l_res_g));
        func.instruction(&Instruction::F32Const(0.0));
        func.instruction(&Instruction::LocalSet(l_res_b));
        func.instruction(&Instruction::F32Const(0.0));
        func.instruction(&Instruction::LocalSet(l_res_a));

        func.instruction(&Instruction::I32Const(0));
        func.instruction(&Instruction::LocalSet(l_loop_cnt));

        func.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));

        // Compute curr_x, curr_y, curr_z
        // curr_x = (cnt & 1) ? x1 : x0
        func.instruction(&Instruction::LocalGet(l_loop_cnt));
        func.instruction(&Instruction::I32Const(1));
        func.instruction(&Instruction::I32And);
        func.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        func.instruction(&Instruction::LocalGet(l_x1));
        func.instruction(&Instruction::Else);
        func.instruction(&Instruction::LocalGet(l_x0));
        func.instruction(&Instruction::End);
        func.instruction(&Instruction::LocalSet(l_tx));

        // curr_y = (cnt & 2) ? y1 : y0
        func.instruction(&Instruction::LocalGet(l_loop_cnt));
        func.instruction(&Instruction::I32Const(2));
        func.instruction(&Instruction::I32And);
        func.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        func.instruction(&Instruction::LocalGet(l_y1));
        func.instruction(&Instruction::Else);
        func.instruction(&Instruction::LocalGet(l_y0));
        func.instruction(&Instruction::End);
        func.instruction(&Instruction::LocalSet(l_ty));

        if is_3d {
            // curr_z = (cnt & 4) ? z1 : z0
            func.instruction(&Instruction::LocalGet(l_loop_cnt));
            func.instruction(&Instruction::I32Const(4));
            func.instruction(&Instruction::I32And);
            func.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
            func.instruction(&Instruction::LocalGet(l_z1));
            func.instruction(&Instruction::Else);
            func.instruction(&Instruction::LocalGet(l_z0));
            func.instruction(&Instruction::End);
            func.instruction(&Instruction::LocalSet(l_tz));
        }

        // Compute weight
        // w_curr_x = (cnt & 1) ? wx : (1 - wx)
        func.instruction(&Instruction::LocalGet(l_loop_cnt));
        func.instruction(&Instruction::I32Const(1));
        func.instruction(&Instruction::I32And);
        func.instruction(&Instruction::If(BlockType::Result(ValType::F32)));
        func.instruction(&Instruction::LocalGet(l_wx));
        func.instruction(&Instruction::Else);
        func.instruction(&Instruction::F32Const(1.0));
        func.instruction(&Instruction::LocalGet(l_wx));
        func.instruction(&Instruction::F32Sub);
        func.instruction(&Instruction::End);

        // w_curr_y = (cnt & 2) ? wy : (1 - wy)
        func.instruction(&Instruction::LocalGet(l_loop_cnt));
        func.instruction(&Instruction::I32Const(2));
        func.instruction(&Instruction::I32And);
        func.instruction(&Instruction::If(BlockType::Result(ValType::F32)));
        func.instruction(&Instruction::LocalGet(l_wy));
        func.instruction(&Instruction::Else);
        func.instruction(&Instruction::F32Const(1.0));
        func.instruction(&Instruction::LocalGet(l_wy));
        func.instruction(&Instruction::F32Sub);
        func.instruction(&Instruction::End);
        func.instruction(&Instruction::F32Mul);

        if is_3d {
            // w_curr_z = (cnt & 4) ? wz : (1 - wz)
            func.instruction(&Instruction::LocalGet(l_loop_cnt));
            func.instruction(&Instruction::I32Const(4));
            func.instruction(&Instruction::I32And);
            func.instruction(&Instruction::If(BlockType::Result(ValType::F32)));
            func.instruction(&Instruction::LocalGet(l_wz));
            func.instruction(&Instruction::Else);
            func.instruction(&Instruction::F32Const(1.0));
            func.instruction(&Instruction::LocalGet(l_wz));
            func.instruction(&Instruction::F32Sub);
            func.instruction(&Instruction::End);
            func.instruction(&Instruction::F32Mul);
        }

        func.instruction(&Instruction::LocalSet(l_tmp_a)); // weight

        // 4b. Compute byte address
        func.instruction(&Instruction::LocalGet(l_layout));
        func.instruction(&Instruction::I32Const(1)); // Tiled8x8
        func.instruction(&Instruction::I32Eq);
        func.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        // Tiled Logic (Z-aware for 3D)
        func.instruction(&Instruction::LocalGet(l_width));
        func.instruction(&Instruction::I32Const(7));
        func.instruction(&Instruction::I32Add);
        func.instruction(&Instruction::I32Const(3));
        func.instruction(&Instruction::I32ShrU); // width_in_tiles

        if is_3d {
            func.instruction(&Instruction::LocalGet(l_tz));
            func.instruction(&Instruction::LocalGet(l_height));
            func.instruction(&Instruction::I32Const(7));
            func.instruction(&Instruction::I32Add);
            func.instruction(&Instruction::I32Const(3));
            func.instruction(&Instruction::I32ShrU); // height_in_tiles
            func.instruction(&Instruction::I32Mul);
            func.instruction(&Instruction::LocalGet(l_ty));
            func.instruction(&Instruction::I32Const(3));
            func.instruction(&Instruction::I32ShrU);
            func.instruction(&Instruction::I32Add);
        } else {
            func.instruction(&Instruction::LocalGet(l_ty));
            func.instruction(&Instruction::I32Const(3));
            func.instruction(&Instruction::I32ShrU);
        }
        func.instruction(&Instruction::I32Mul);
        func.instruction(&Instruction::LocalGet(l_tx));
        func.instruction(&Instruction::I32Const(3));
        func.instruction(&Instruction::I32ShrU);
        func.instruction(&Instruction::I32Add);
        func.instruction(&Instruction::I32Const(6));
        func.instruction(&Instruction::I32Shl); // * 64
        func.instruction(&Instruction::LocalGet(l_ty));
        func.instruction(&Instruction::I32Const(7));
        func.instruction(&Instruction::I32And);
        func.instruction(&Instruction::I32Const(3));
        func.instruction(&Instruction::I32Shl); // * 8
        func.instruction(&Instruction::LocalGet(l_tx));
        func.instruction(&Instruction::I32Const(7));
        func.instruction(&Instruction::I32And);
        func.instruction(&Instruction::I32Add);
        func.instruction(&Instruction::I32Add);
        func.instruction(&Instruction::Else);
        // Linear Logic
        if is_3d {
            func.instruction(&Instruction::LocalGet(l_tz));
            func.instruction(&Instruction::LocalGet(l_height));
            func.instruction(&Instruction::I32Mul);
            func.instruction(&Instruction::LocalGet(l_ty));
            func.instruction(&Instruction::I32Add);
            func.instruction(&Instruction::LocalGet(l_width));
            func.instruction(&Instruction::I32Mul);
            func.instruction(&Instruction::LocalGet(l_tx));
            func.instruction(&Instruction::I32Add);
        } else {
            func.instruction(&Instruction::LocalGet(l_ty));
            func.instruction(&Instruction::LocalGet(l_width));
            func.instruction(&Instruction::I32Mul);
            func.instruction(&Instruction::LocalGet(l_tx));
            func.instruction(&Instruction::I32Add);
        }
        func.instruction(&Instruction::End); // End If Tiled

        // Convert index to absolute address
        func.instruction(&Instruction::LocalGet(l_bpp));
        func.instruction(&Instruction::I32Mul);
        func.instruction(&Instruction::LocalGet(l_ptr));
        func.instruction(&Instruction::I32Add);
        func.instruction(&Instruction::LocalSet(l_addr)); // REAL address

        // 5. Load and accumulate
        func.instruction(&Instruction::LocalGet(l_format));
        func.instruction(&Instruction::I32Const(0x8058)); // RGBA8
        func.instruction(&Instruction::I32Eq);
        func.instruction(&Instruction::LocalGet(l_format));
        func.instruction(&Instruction::I32Const(0x1908)); // RGBA
        func.instruction(&Instruction::I32Eq);
        func.instruction(&Instruction::I32Or);
        func.instruction(&Instruction::If(wasm_encoder::BlockType::Empty));
        for i in 0..4 {
            func.instruction(&Instruction::LocalGet(l_addr));
            func.instruction(&Instruction::I32Load8U(wasm_encoder::MemArg {
                offset: i as u64,
                align: 0,
                memory_index: 0,
            }));
            func.instruction(&Instruction::F32ConvertI32U);
            func.instruction(&Instruction::F32Const(255.0));
            func.instruction(&Instruction::F32Div);
            func.instruction(&Instruction::LocalGet(l_tmp_a)); // weight
            func.instruction(&Instruction::F32Mul);
            func.instruction(&Instruction::LocalGet(l_res_r + i as u32));
            func.instruction(&Instruction::F32Add);
            func.instruction(&Instruction::LocalSet(l_res_r + i as u32));
        }
        func.instruction(&Instruction::Else);
        func.instruction(&Instruction::LocalGet(l_format));
        func.instruction(&Instruction::I32Const(0x8814)); // RGBA32F
        func.instruction(&Instruction::I32Eq);
        func.instruction(&Instruction::If(wasm_encoder::BlockType::Empty));
        for i in 0..4 {
            func.instruction(&Instruction::LocalGet(l_addr));
            func.instruction(&Instruction::F32Load(wasm_encoder::MemArg {
                offset: (i * 4) as u64,
                align: 2,
                memory_index: 0,
            }));
            func.instruction(&Instruction::LocalGet(l_tmp_a)); // weight
            func.instruction(&Instruction::F32Mul);
            func.instruction(&Instruction::LocalGet(l_res_r + i as u32));
            func.instruction(&Instruction::F32Add);
            func.instruction(&Instruction::LocalSet(l_res_r + i as u32));
        }
        func.instruction(&Instruction::Else);
        // Handle R32F (0x822E), RG32F simplified
        func.instruction(&Instruction::LocalGet(l_format));
        func.instruction(&Instruction::I32Const(0x822E)); // R32F
        func.instruction(&Instruction::I32Eq);
        func.instruction(&Instruction::If(wasm_encoder::BlockType::Empty));
        func.instruction(&Instruction::LocalGet(l_addr));
        func.instruction(&Instruction::F32Load(wasm_encoder::MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        }));
        func.instruction(&Instruction::LocalGet(l_tmp_a)); // weight
        func.instruction(&Instruction::F32Mul);
        func.instruction(&Instruction::LocalGet(l_res_r));
        func.instruction(&Instruction::F32Add);
        func.instruction(&Instruction::LocalSet(l_res_r));

        func.instruction(&Instruction::LocalGet(l_tmp_a)); // weight (for alpha)
        func.instruction(&Instruction::LocalGet(l_res_a));
        func.instruction(&Instruction::F32Add);
        func.instruction(&Instruction::LocalSet(l_res_a));
        func.instruction(&Instruction::End);
        func.instruction(&Instruction::End);
        func.instruction(&Instruction::End);

        // Loop management
        func.instruction(&Instruction::LocalGet(l_loop_cnt));
        func.instruction(&Instruction::I32Const(1));
        func.instruction(&Instruction::I32Add);
        func.instruction(&Instruction::LocalTee(l_loop_cnt));
        func.instruction(&Instruction::I32Const(if is_3d { 8 } else { 4 }));
        func.instruction(&Instruction::I32LtU);
        func.instruction(&Instruction::BrIf(0));
        func.instruction(&Instruction::End); // End Loop

        func.instruction(&Instruction::LocalGet(l_res_r));
        func.instruction(&Instruction::LocalGet(l_res_g));
        func.instruction(&Instruction::LocalGet(l_res_b));
        func.instruction(&Instruction::LocalGet(l_res_a));

        func.instruction(&Instruction::End);
        self.code.function(&func);

        func_idx
    }

    fn compile(&mut self) -> Result<(), BackendError> {
        // Import memory from host
        self.imports.import(
            "env",
            "memory",
            MemoryType {
                minimum: 100, // 6.4MB
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

        // Add math imports for transcendental functions
        let math_funcs = [
            (naga::MathFunction::Sin, "gl_sin", 1),
            (naga::MathFunction::Cos, "gl_cos", 1),
            (naga::MathFunction::Tan, "gl_tan", 1),
            (naga::MathFunction::Asin, "gl_asin", 1),
            (naga::MathFunction::Acos, "gl_acos", 1),
            (naga::MathFunction::Atan, "gl_atan", 1),
            (naga::MathFunction::Atan2, "gl_atan2", 2),
            (naga::MathFunction::Exp, "gl_exp", 1),
            (naga::MathFunction::Exp2, "gl_exp2", 1),
            (naga::MathFunction::Log, "gl_log", 1),
            (naga::MathFunction::Log2, "gl_log2", 1),
            (naga::MathFunction::Pow, "gl_pow", 2),
            (naga::MathFunction::Sinh, "gl_sinh", 1),
            (naga::MathFunction::Cosh, "gl_cosh", 1),
            (naga::MathFunction::Tanh, "gl_tanh", 1),
            (naga::MathFunction::Asinh, "gl_asinh", 1),
            (naga::MathFunction::Acosh, "gl_acosh", 1),
            (naga::MathFunction::Atanh, "gl_atanh", 1),
        ];

        for (func, name, param_count) in math_funcs {
            let type_idx = self.types.len();
            let params = vec![ValType::F32; param_count];
            let results = vec![ValType::F32];
            self.types.ty().function(params, results);

            self.imports
                .import("env", name, wasm_encoder::EntityType::Function(type_idx));
            self.math_import_map.insert(func, self.function_count);
            self.function_count += 1;
        }

        // Emit the module-local texture sampling helpers
        let (need_2d, need_3d) = self.has_image_sampling();
        if need_2d {
            self.webgl_sampler_2d_idx = Some(self.emit_sampler(naga::ImageDimension::D2));
        }
        if need_3d {
            self.webgl_sampler_3d_idx = Some(self.emit_sampler(naga::ImageDimension::D3));
        }

        if self.has_image_load() {
            self.emit_image_load_helper();
        }

        let global_names = [
            "ACTIVE_ATTR_PTR",
            "ACTIVE_UNIFORM_PTR",
            "ACTIVE_VARYING_PTR",
            "ACTIVE_PRIVATE_PTR",
            "ACTIVE_TEXTURE_PTR",
            "ACTIVE_FRAME_SP",
        ];

        for (i, name) in global_names.iter().enumerate() {
            self.imports.import(
                "env",
                name,
                wasm_encoder::EntityType::Global(wasm_encoder::GlobalType {
                    val_type: ValType::I32,
                    mutable: true,
                    shared: false,
                }),
            );
        }

        // Number of global imports we registered; used to offset module-local global indices
        let global_import_count = global_names.len() as u32;

        // Calculate global offsets per address space
        let mut varying_offset = 32; // User varyings start after Position and PointSize (16+16=32)

        // First pass: find gl_Position and gl_PointSize and put them at fixed offsets
        for (handle, var) in self.module.global_variables.iter() {
            if let Some(name) = &var.name {
                if name == "gl_Position" || name == "gl_Position_1" {
                    self.global_offsets
                        .insert(handle, (0, output_layout::VARYING_PTR_GLOBAL));
                } else if name == "gl_PointSize" {
                    self.global_offsets
                        .insert(handle, (16, output_layout::VARYING_PTR_GLOBAL));
                }
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
                    // For both Uniform and Handle (in index model), storage is in uniform memory.
                    // Handles (samplers/images) store their unit index as an i32 in WebGL.
                    let default_base_ptr = output_layout::UNIFORM_PTR_GLOBAL;

                    // Try to match by binding first (preferred for WebGPU)
                    let found_by_binding = if let Some(rb) = &var.binding {
                        self.uniform_map
                            .get(&(rb.group, rb.binding))
                            .map(|(offset, _)| (*offset, default_base_ptr))
                    } else {
                        None
                    };

                    if let Some(res) = found_by_binding {
                        res
                    } else if let Some(name) = &var.name {
                        if let Some(&loc) = self.uniform_locations.get(name) {
                            let off = if matches!(var.space, naga::AddressSpace::Handle) {
                                output_layout::get_webgl_uniform_data_offset(loc)
                            } else {
                                output_layout::compute_uniform_offset(loc).0
                            };
                            (off, default_base_ptr)
                        } else {
                            (0, default_base_ptr)
                        }
                    } else {
                        (0, default_base_ptr)
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
                        if name == "gl_Position" || name == "gl_Position_1" {
                            (0, output_layout::VARYING_PTR_GLOBAL)
                        } else if let Some(&loc) = self.attribute_locations.get(name) {
                            output_layout::compute_input_offset(loc, naga::ShaderStage::Vertex)
                        } else if let Some(&loc) = self.varying_locations.get(name) {
                            output_layout::compute_input_offset(loc, naga::ShaderStage::Fragment)
                        } else {
                            if self.stage == naga::ShaderStage::Vertex {
                                let o = varying_offset;
                                varying_offset += size;
                                varying_offset = (varying_offset + 3) & !3;
                                (o, output_layout::VARYING_PTR_GLOBAL)
                            } else {
                                (0, output_layout::VARYING_PTR_GLOBAL)
                            }
                        }
                    } else {
                        if self.stage == naga::ShaderStage::Vertex {
                            let o = varying_offset;
                            varying_offset += size;
                            varying_offset = (varying_offset + 3) & !3;
                            (o, output_layout::VARYING_PTR_GLOBAL)
                        } else {
                            (0, output_layout::VARYING_PTR_GLOBAL)
                        }
                    }
                }
                // Handle explicit In/Out address spaces (used in newer Naga versions)
                _ => {
                    // Check if it's an output in FS (AddressSpace::Out)
                    let is_fs_output = if self.stage == naga::ShaderStage::Fragment {
                        if let Some(name) = &var.name {
                            let n = name.as_str();
                            n == "color"
                                || n == "fragColor"
                                || n == "gl_FragColor"
                                || n == "outColor"
                                || n.ends_with("Color")
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if is_fs_output {
                        (0, output_layout::PRIVATE_PTR_GLOBAL)
                    } else {
                        let o = varying_offset;
                        varying_offset += size;
                        varying_offset = (varying_offset + 3) & !3;
                        (o, output_layout::VARYING_PTR_GLOBAL)
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
            if entry_point.stage != self.stage {
                continue;
            }
            if let Some(target) = self.entry_point_name {
                if entry_point.name != target {
                    continue;
                }
            }
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

        if let Some(ep) = entry_point {
            match ep.stage {
                naga::ShaderStage::Vertex => {
                    // Turbo VS: (vertex_id: 0, instance_id: 1, varying_out_ptr: 2)
                    params = vec![ValType::I32, ValType::I32, ValType::I32];
                }
                naga::ShaderStage::Fragment => {
                    // Turbo FS: (varying_in_ptr: 0, private_ptr: 1)
                    params = vec![ValType::I32, ValType::I32];
                }
                _ => {
                    return Err(BackendError::InternalError(format!(
                        "Unsupported shader stage: {:?}",
                        ep.stage
                    )));
                }
            }
            results = vec![];
            current_param_idx = params.len() as u32;
        } else {
            let manifest = self
                .function_registry
                .get_function(func_handle.expect("Function handle missing for non-entrypoint"))
                .ok_or_else(|| {
                    BackendError::InternalError(format!(
                        "Pre-computed manifest missing for function {:?}",
                        func_handle
                    ))
                })?;
            let abi = &manifest.abi;

            params = abi.param_valtypes();
            results = abi.result_valtypes();

            // Map argument handles to parameter indices
            let mut param_offset = 0;
            for (i, arg_abi) in abi.params.iter().enumerate() {
                argument_local_offsets.insert(i as u32, current_param_idx + param_offset);
                let count = match arg_abi {
                    super::function_abi::ParameterABI::Flattened { valtypes, .. } => {
                        valtypes.len() as u32
                    }
                    super::function_abi::ParameterABI::Frame { .. } => 1,
                };
                param_offset += count;
            }
            current_param_idx += params.len() as u32;
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
            typifier
                .grow(handle, &func.expressions, &resolve_ctx)
                .map_err(|e| {
                    BackendError::UnsupportedFeature(format!("Typifier error: {:?}", e))
                })?;
        }

        // Calculate proper memory layout for private memory region
        // This replaces the old hardcoded offsets (2048 for locals, 4096 for FragDepth)
        // with a calculated, validated layout
        let memory_layout =
            super::memory_layout::PrivateMemoryLayout::compute(self.module, func, self.stage)?;

        // Log the calculated layout for debugging
        tracing::debug!(
            "Private memory layout - Frag outputs: {} bytes, Locals: {} bytes (start: {}), \
             FragDepth: {:?}, Total: {} bytes",
            memory_layout.frag_outputs_size,
            memory_layout.locals_size,
            memory_layout.locals_start,
            memory_layout.frag_depth_offset,
            memory_layout.total_size
        );

        // Use the pre-calculated local variable offsets from the memory_layout
        let local_offsets = memory_layout.local_offsets.clone();

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

        // Add explicit swap locals at the END to preserve existing indices
        // These will be used by store_components_to_memory instead of scanning
        let swap_i32_local = next_local_idx;
        locals_types.push((1, ValType::I32)); // swap_i32_local
        next_local_idx += 1;

        let swap_f32_local = next_local_idx;
        locals_types.push((1, ValType::F32)); // swap_f32_local
        next_local_idx += 1;

        // Detect need for Float Modulo swap local
        let mut uses_float_modulo = false;
        for (_handle, expr) in func.expressions.iter() {
            if let naga::Expression::Binary {
                op: naga::BinaryOperator::Modulo,
                left,
                ..
            } = expr
            {
                let resolution = typifier.get(*left, &self.module.types);
                match resolution {
                    naga::TypeInner::Scalar(scalar) if scalar.kind == naga::ScalarKind::Float => {
                        uses_float_modulo = true;
                        break;
                    }
                    naga::TypeInner::Vector { scalar, .. }
                        if scalar.kind == naga::ScalarKind::Float =>
                    {
                        uses_float_modulo = true;
                        break;
                    }
                    _ => {}
                }
            }
        }

        let swap_f32_local_2 = if uses_float_modulo {
            let idx = next_local_idx;
            locals_types.push((1, ValType::F32)); // swap_f32_local_2 for float modulo
            next_local_idx += 1;
            Some(idx)
        } else {
            None
        };

        // Add frame temp local (conservative allocation for Phase 4)
        let frame_temp_local = next_local_idx;
        locals_types.push((1, ValType::I32)); // frame_temp
        next_local_idx += 1;

        // Phase 4: Explicit locals for image sampling results (4 f32s)
        let uses_sampling = func
            .expressions
            .iter()
            .any(|(_, expr)| matches!(expr, naga::Expression::ImageSample { .. }));

        let sample_f32_locals = if uses_sampling {
            let idx = next_local_idx;
            locals_types.push((4, ValType::F32)); // r, g, b, a
            next_local_idx += 4;
            Some(idx)
        } else {
            None
        };

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

        // Create function body
        let mut wasm_func = Function::new(locals_types);

        if let Some(ep) = entry_point {
            // Tier 1 logic: Shared globals (Attributes, Uniforms, Textures) are already set by the host. Zero preamble cost.
            // Tier 2 logic: Sync volatile pointers (Varying, Private) from arguments to globals for internal accesses.
            match ep.stage {
                naga::ShaderStage::Vertex => {
                    // Argument 2 is varying_out_ptr. Sync it to VARYING_PTR_GLOBAL.
                    wasm_func.instruction(&Instruction::LocalGet(2));
                    wasm_func
                        .instruction(&Instruction::GlobalSet(output_layout::VARYING_PTR_GLOBAL));
                }
                naga::ShaderStage::Fragment => {
                    // Argument 0 is varying_in_ptr. Sync it to VARYING_PTR_GLOBAL (offset by imports).
                    wasm_func.instruction(&Instruction::LocalGet(0));
                    wasm_func
                        .instruction(&Instruction::GlobalSet(output_layout::VARYING_PTR_GLOBAL));
                    // Argument 1 is private_ptr. Sync it to PRIVATE_PTR_GLOBAL.
                    wasm_func.instruction(&Instruction::LocalGet(1));
                    wasm_func
                        .instruction(&Instruction::GlobalSet(output_layout::PRIVATE_PTR_GLOBAL));
                }
                _ => {}
            }
        }

        let stage = self.stage;
        let is_entry_point = entry_point.is_some();

        // Initialize local variables that have init expressions
        // This must happen before any statement execution
        for (handle, var) in func.local_variables.iter() {
            if let Some(init_expr) = var.init {
                // Get the local offset
                if let Some(&offset) = local_offsets.get(&handle) {
                    // Emit Store for the initialization
                    let value_ty = &self.module.types[var.ty].inner;
                    let num_components =
                        super::types::component_count(value_ty, &self.module.types);
                    let use_i32_store = super::expressions::is_integer_type(value_ty);

                    for i in 0..num_components {
                        // Compute address: private_ptr + offset + (i * 4)
                        wasm_func.instruction(&Instruction::GlobalGet(
                            output_layout::PRIVATE_PTR_GLOBAL,
                        ));
                        wasm_func.instruction(&Instruction::I32Const((offset + i * 4) as i32));
                        wasm_func.instruction(&Instruction::I32Add);

                        // Evaluate init expression component
                        // We need a temporary context for this
                        let mut init_ctx = super::TranslationContext {
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
                            math_import_map: &self.math_import_map,
                            typifier: &typifier,
                            naga_function_map: &self.naga_function_map,
                            function_registry: self.function_registry,
                            argument_local_offsets: &argument_local_offsets,
                            attribute_locations: self.attribute_locations,
                            uniform_locations: self.uniform_locations,
                            varying_locations: self.varying_locations,
                            varying_types: self.varying_types,
                            uniform_types: self.uniform_types,
                            attribute_types: self.attribute_types,
                            local_origins: &local_origins,
                            is_entry_point,
                            private_memory_layout: Some(&memory_layout),
                            swap_i32_local,
                            swap_f32_local,
                            swap_f32_local_2,
                            local_types: &flattened_local_types,
                            param_count: current_param_idx,
                            webgl_sampler_2d_idx: self.webgl_sampler_2d_idx,
                            webgl_sampler_3d_idx: self.webgl_sampler_3d_idx,
                            webgl_image_load_idx: self.webgl_image_load_idx,
                            frame_temp_idx: Some(frame_temp_local),
                            sample_f32_locals,
                            block_stack: Vec::new(),
                        };
                        super::expressions::translate_expression_component(
                            init_expr,
                            i,
                            &mut init_ctx,
                        )?;

                        // Store the value
                        if use_i32_store {
                            wasm_func.instruction(&Instruction::I32Store(wasm_encoder::MemArg {
                                offset: 0,
                                align: 2,
                                memory_index: 0,
                            }));
                        } else {
                            wasm_func.instruction(&Instruction::F32Store(wasm_encoder::MemArg {
                                offset: 0,
                                align: 2,
                                memory_index: 0,
                            }));
                        }
                    }
                }
            }
        }

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
            math_import_map: &self.math_import_map,
            typifier: &typifier,
            naga_function_map: &self.naga_function_map,
            function_registry: self.function_registry,
            argument_local_offsets: &argument_local_offsets,
            attribute_locations: self.attribute_locations,
            uniform_locations: self.uniform_locations,
            varying_locations: self.varying_locations,
            varying_types: self.varying_types,
            uniform_types: self.uniform_types,
            attribute_types: self.attribute_types,
            local_origins: &local_origins,
            is_entry_point,
            private_memory_layout: Some(&memory_layout),
            swap_i32_local,
            swap_f32_local,
            swap_f32_local_2,
            // Local types and parameter count for type-aware lowering
            local_types: &flattened_local_types,
            param_count: current_param_idx,
            webgl_sampler_2d_idx: self.webgl_sampler_2d_idx,
            webgl_sampler_3d_idx: self.webgl_sampler_3d_idx,
            webgl_image_load_idx: self.webgl_image_load_idx,
            frame_temp_idx: Some(frame_temp_local),
            sample_f32_locals,
            block_stack: Vec::new(),
        };

        for (stmt, span) in func.body.span_iter() {
            super::control_flow::translate_statement(stmt, span, &mut ctx)?;
        }

        if let Some(ep) = entry_point {
            // Tier 3: Results are already stored in memory via shared globals. Return void.
        }

        wasm_func.instruction(&Instruction::End);
        self.code.function(&wasm_func);

        // Export internal functions in debug mode
        if entry_point.is_none() && self._backend.config.debug_shaders {
            let name = format!("func_{}", func_idx);
            if self.exported_names.insert(name.clone()) {
                self.exports.export(&name, ExportKind::Func, func_idx);
            }
        }

        Ok(func_idx)
    }

    fn compile_entry_point(
        &mut self,
        entry_point: &naga::EntryPoint,
        _index: usize,
    ) -> Result<(), BackendError> {
        let func_idx = self.compile_function(&entry_point.function, Some(entry_point), None)?;

        // Export the function as its original name if not already exported
        if self.exported_names.insert(entry_point.name.clone()) {
            self.exports
                .export(&entry_point.name, ExportKind::Func, func_idx);
        }

        // Alias to "main" only if it matches our target entry point,
        // or if no target was specified and it's the first one.
        let is_target = match self.entry_point_name {
            Some(target) => entry_point.name == target,
            None => true, // Default to first available if not specified
        };

        if is_target && self.exported_names.insert("main".to_string()) {
            self.exports.export("main", ExportKind::Func, func_idx);
        }

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

        // Add Name section for debugging and validation
        let mut names = NameSection::new();
        let mut func_names = NameMap::new();
        let mut has_names = false;

        if let Some(idx) = self.webgl_sampler_2d_idx {
            func_names.append(idx, "__webgl_sampler_2d");
            has_names = true;
        }

        if let Some(idx) = self.webgl_sampler_3d_idx {
            func_names.append(idx, "__webgl_sampler_3d");
            has_names = true;
        }

        if has_names {
            names.functions(&func_names);
            module.section(&names);
        }

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
            table_index: 0,
        }
    }
}
