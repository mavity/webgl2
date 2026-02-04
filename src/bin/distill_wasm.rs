//! WebGL2 WASM Coverage Distiller
//!
//! This tool instruments WASM binaries with coverage tracking.
//! It uses DWARF debug info to map instrumentation points to source lines.

use anyhow::{Context, Result};
use clap::Parser;
use gimli::RunTimeEndian;
use rustc_demangle::demangle;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use walrus::{ir, ConstExpr, Module, ModuleConfig};

#[derive(Parser)]
#[command(name = "distill_wasm")]
#[command(about = "Instrument WASM with coverage tracking", long_about = None)]
struct Cli {
    /// Input WASM file
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Output instrumented WASM file
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    // Determine output path
    let output_path = cli.output.unwrap_or_else(|| {
        let mut p = cli.input.clone();
        let stem = p.file_stem().unwrap().to_string_lossy();
        p.set_file_name(format!("{}.instrumented.wasm", stem));
        p
    });

    tracing::info!("Instrumenting WASM: {:?} → {:?}", cli.input, output_path);

    // Read raw bytes for DWARF parsing
    let wasm_bytes = std::fs::read(&cli.input).context("Failed to read input WASM file")?;

    // Load WASM module with walrus
    let config = ModuleConfig::new();
    let mut module = config
        .parse_file(&cli.input)
        .context("Failed to parse input WASM")?;

    // Step 1: Analyze DWARF and build mapping
    let (func_to_probes, probe_to_loc) = build_coverage_mapping(&wasm_bytes, &module)?;
    tracing::info!("Found {} locations tied to probes", probe_to_loc.len());

    let (mapping_data, total_probes) = create_coverage_data(&probe_to_loc)?;
    tracing::info!("Total probe IDs tracked: {}", total_probes);

    // Emit custom section for external tools/tests
    module.customs.add(walrus::RawCustomSection {
        name: "cov_mapping".to_string(),
        data: mapping_data.clone(),
    });

    // Check stack pointer
    if let Some(sp_id) = find_exported_global(&module, "__stack_pointer") {
        if let Some(sp_val) = get_global_val(&module, sp_id) {
            tracing::info!("__stack_pointer is initialized to {}", sp_val);
        }
    } else {
        // Try finding it by name in globals even if not exported
        let mut found = false;
        for global in module.globals.iter() {
            if let Some(name) = &global.name {
                if name == "__stack_pointer" {
                    if let walrus::GlobalKind::Local(ConstExpr::Value(ir::Value::I32(v))) =
                        global.kind
                    {
                        tracing::info!("__stack_pointer (local) is initialized to {}", v);
                        found = true;
                    }
                }
            }
        }
        if !found {
            tracing::warn!("__stack_pointer not found");
        }
    }

    // Step 3: Allocate mapping data segment (read-only)
    // We put this in the data section so `get_lcov_report` can read it.
    let mapping_offset = allocate_data_segment(&mut module, &mapping_data)?;
    tracing::info!("Allocated mapping segment at offset {}", mapping_offset);

    // Step 3b: Allocate hits segment (read-write)
    // We allocate this statically so it's available immediately.
    let hits_size = total_probes;
    let hits_data = vec![0u8; hits_size];
    let hits_offset = allocate_data_segment(&mut module, &hits_data)?;
    tracing::info!("Allocated hits segment at offset {}", hits_offset);

    // Step 4: Patch COV_MAP_PTR and COV_MAP_LEN
    let map_ptr_id = find_exported_global(&module, "COV_MAP_PTR")
        .context("COV_MAP_PTR global not found. Build with --features coverage")?;

    // Patch COV_MAP_PTR in memory
    if let Some(addr) = get_global_val(&module, map_ptr_id) {
        module.data.add(
            walrus::DataKind::Active {
                memory: module.memories.iter().next().unwrap().id(),
                offset: ConstExpr::Value(ir::Value::I32(addr as i32)),
            },
            mapping_offset.to_le_bytes().to_vec(),
        );
    }
    // Do NOT patch the global export for MAP_PTR, as it is used as an address by tests/runtime
    // patch_global_ptr(&mut module, map_ptr_id, mapping_offset)?;

    let map_len_id = find_exported_global(&module, "COV_MAP_LEN")
        .context("COV_MAP_LEN global not found. Build with --features coverage")?;

    // Patch COV_MAP_LEN in memory
    if let Some(addr) = get_global_val(&module, map_len_id) {
        module.data.add(
            walrus::DataKind::Active {
                memory: module.memories.iter().next().unwrap().id(),
                offset: ConstExpr::Value(ir::Value::I32(addr as i32)),
            },
            (mapping_data.len() as u32).to_le_bytes().to_vec(),
        );
    }

    let hits_len_id = find_exported_global(&module, "COV_HITS_LEN")
        .context("COV_HITS_LEN global not found. Build with --features coverage")?;

    // Patch COV_HITS_LEN in memory
    if let Some(addr) = get_global_val(&module, hits_len_id) {
        module.data.add(
            walrus::DataKind::Active {
                memory: module.memories.iter().next().unwrap().id(),
                offset: ConstExpr::Value(ir::Value::I32(addr as i32)),
            },
            total_probes.to_le_bytes().to_vec(),
        );
    }
    // Do NOT patch the global export for MAP_LEN
    // patch_global_len(&mut module, map_len_id, mapping_data.len() as u32)?;

    tracing::info!("Patched COV_MAP_PTR and COV_MAP_LEN in memory");

    // Step 5: Instrument functions
    // We need to find the ID of COV_HITS_PTR global
    let cov_hits_ptr_id = find_exported_global(&module, "COV_HITS_PTR")
        .context("COV_HITS_PTR not found. Build with --features coverage")?;

    // CRITICAL FIX: Initialize the static variable in memory AND the global export.
    // 1. Get the address of the static variable from the global's current value (before we patch it)
    if let Some(static_addr) = get_global_val(&module, cov_hits_ptr_id) {
        tracing::info!("COV_HITS_PTR static variable is at address {}", static_addr);

        // 2. Write the hits_offset into the static variable in memory
        // We do this by adding a new data segment that overwrites that location
        module.data.add(
            walrus::DataKind::Active {
                memory: module.memories.iter().next().unwrap().id(),
                offset: ConstExpr::Value(ir::Value::I32(static_addr as i32)),
            },
            hits_offset.to_le_bytes().to_vec(),
        );
        tracing::info!("Initialized COV_HITS_PTR in memory to {}", hits_offset);
    } else {
        tracing::warn!("Could not determine address of COV_HITS_PTR static variable. Runtime initialization might fail.");
    }

    // 3. Patch the global export to point to the hits buffer directly
    // This allows instrument_functions to use `global.get` to find the buffer base address
    patch_global_ptr(&mut module, cov_hits_ptr_id, hits_offset)?;

    instrument_functions(&mut module, &func_to_probes, hits_offset)?;

    // Write output
    module.emit_wasm_file(&output_path)?;
    tracing::info!("Wrote instrumented WASM to {:?}", output_path);

    Ok(())
}

/// Build mapping of instrumentation IDs to source locations.
/// Returns (FuncId -> [ProbeId], ProbeId -> (File, Line, Column))
#[allow(clippy::type_complexity)]
fn build_coverage_mapping(
    wasm_bytes: &[u8],
    module: &Module,
) -> Result<(
    HashMap<walrus::FunctionId, Vec<Option<u32>>>,
    HashMap<u32, (String, u32, u32)>,
)> {
    let mut func_to_probes = HashMap::new();
    let mut probe_to_loc = HashMap::new();

    let parser = wasmparser::Parser::new(0);
    let mut sections = HashMap::new();
    let mut code_section_start = 0;
    let mut function_bodies = Vec::new();

    for payload in parser.parse_all(wasm_bytes) {
        match payload? {
            wasmparser::Payload::CustomSection(reader) => {
                let name = reader.name();
                let data = reader.data().to_vec();
                sections.insert(name.to_string(), data);
            }
            wasmparser::Payload::CodeSectionStart { range, .. } => {
                code_section_start = range.start;
            }
            wasmparser::Payload::CodeSectionEntry(body) => {
                function_bodies.push(body);
            }
            _ => {}
        }
    }

    // Load DWARF
    let loader =
        |id: gimli::SectionId| -> Result<gimli::EndianSlice<'static, RunTimeEndian>, gimli::Error> {
            let name = id.name();
            let data = sections.get(name).map(|v| v.as_slice()).unwrap_or(&[]);
            let leaked: &'static [u8] = Box::leak(data.to_vec().into_boxed_slice());
            Ok(gimli::EndianSlice::new(leaked, RunTimeEndian::Little))
        };

    let dwarf = gimli::Dwarf::load(&loader).ok();
    let dwarf_lookup = dwarf.as_ref().map(DwarfLookup::new);

    let mut probe_id = 0;

    // Build a sorted list of local functions by their entry block ID or name
    // to ensure deterministic ordering.
    let mut local_funcs: Vec<_> = module.funcs.iter_local().collect();
    local_funcs.sort_by_key(|(id, _)| {
        let f = module.funcs.get(*id);
        let raw_name = f.name.as_deref().unwrap_or("unknown");
        let demangled = demangle(raw_name).to_string();

        // Secondary sort key to ensure deterministic order if demangled names collide
        (demangled, id.index())
    });

    let local_funcs_map: HashMap<_, _> = module
        .funcs
        .iter_local()
        .enumerate()
        .map(|(idx, (id, _))| (id, idx))
        .collect();

    for (func_id, local_func) in local_funcs {
        let func = module.funcs.get(func_id);
        let raw_name = func.name.as_deref().unwrap_or("unknown");

        // Filter out known system functions
        let demangled = demangle(raw_name).to_string();
        if demangled.contains("dlmalloc")
            || demangled.contains("alloc")
            || demangled.contains("std::")
            || demangled.contains("core::")
            || demangled.contains("compiler_builtins")
        {
            continue;
        }

        // We still need the original WASM body to get the debug locations
        // Since we can't easily get offsets from Walrus, we use a trick:
        // Match the LocalFunction to its index in the module's sub-list
        // and find the corresponding function_body.
        let local_index = local_funcs_map.get(&func_id).copied();

        if local_index.is_none() || local_index.unwrap() >= function_bodies.len() {
            continue;
        }
        let body = &function_bodies[local_index.unwrap()];
        let code_section_offset = code_section_start;

        let mut path = String::new();
        let mut should_skip_function = false;

        let lookup = dwarf_lookup.as_ref().unwrap();
        let func_offset = body.range().start;
        let addr = (func_offset - code_section_offset) as u64;

        if let Some((p, _l, _c)) = lookup.lookup(addr) {
            path = p;
        }

        if path.is_empty() {
            // Check if it's a math builtin or other lib.rs function
            if demangled.starts_with("gl_") || demangled.starts_with("wasm_") {
                path = "src/lib.rs".to_string();
            } else if demangled.contains("::") {
                // Probable rust function, keep searching or mark as unknown
            }
        }

        if path.contains("library/std") || path.contains("dlmalloc") || path.contains("coverage") {
            should_skip_function = true;
        }

        if path.is_empty() {
            should_skip_function = true;
        }

        if should_skip_function {
            continue;
        }

        // if demangled.contains("coverage") {
        //    continue;
        // }

        let mut probes = Vec::new();

        // 2. Other blocks (RECURSIVE TRAVERSAL)
        collect_probes_recursive(
            local_func,
            local_func.entry_block(),
            &mut probes,
            &mut probe_id,
            &mut probe_to_loc,
            dwarf_lookup.as_ref().unwrap(),
            body,
            code_section_offset,
        )?;

        func_to_probes.insert(func_id, probes);
    }

    Ok((func_to_probes, probe_to_loc))
}

#[allow(clippy::too_many_arguments)]
fn collect_probes_recursive(
    func: &walrus::LocalFunction,
    seq_id: walrus::ir::InstrSeqId,
    probes: &mut Vec<Option<u32>>,
    next_probe_id: &mut u32,
    probe_to_loc: &mut HashMap<u32, (String, u32, u32)>,
    lookup: &DwarfLookup,
    _body: &wasmparser::FunctionBody,
    _code_section_offset: usize,
) -> Result<()> {
    // 1. Current sequence (always assigned an ID/None)
    // We only assign a probe if we have a valid source location.
    // However, we MUST push a ProbeId to the vector to maintain alignment with instrumentation.
    // The convention is: push None if no location, push Some(id) if location.

    let mut current_probe_id = None;

    // TRY TO FIND ACTUAL SOURCE LOCATION
    let mut path = String::new();
    let mut line = 0;
    let mut col = 0;
    let mut found_loc = false;

    let seq = func.block(seq_id);

    // Filter out sequences with no instructions as they might not have valid locations
    if !seq.instrs.is_empty() {
        'find_loc: {
            for (_instr, loc) in seq.instrs.iter() {
                // Attempt to look up location via DWARF
                // In walrus 0.24.4, InstrLocId is just a wrapper around u32
                let raw_id = unsafe { std::mem::transmute::<walrus::ir::InstrLocId, u32>(*loc) };
                if raw_id != u32::MAX {
                    if let Some(file_loc) = lookup.lookup(raw_id as u64) {
                        path = file_loc.0;
                        line = file_loc.1;
                        col = file_loc.2;
                        found_loc = true;
                        break 'find_loc;
                    }
                }
            }
        }
    }

    if found_loc {
        let id = *next_probe_id;
        *next_probe_id += 1;
        current_probe_id = Some(id);
        probe_to_loc.insert(id, (path, line, col));
    }

    probes.push(current_probe_id);

    // 2. Traverse instructions
    let seq = func.block(seq_id);
    let instrs: Vec<_> = seq.instrs.iter().map(|(i, _)| i.clone()).collect();
    for instr in instrs {
        match instr {
            walrus::ir::Instr::Block(b) => {
                collect_probes_recursive(
                    func,
                    b.seq,
                    probes,
                    next_probe_id,
                    probe_to_loc,
                    lookup,
                    _body,
                    _code_section_offset,
                )?;
            }
            walrus::ir::Instr::Loop(l) => {
                collect_probes_recursive(
                    func,
                    l.seq,
                    probes,
                    next_probe_id,
                    probe_to_loc,
                    lookup,
                    _body,
                    _code_section_offset,
                )?;
            }
            walrus::ir::Instr::IfElse(i) => {
                collect_probes_recursive(
                    func,
                    i.consequent,
                    probes,
                    next_probe_id,
                    probe_to_loc,
                    lookup,
                    _body,
                    _code_section_offset,
                )?;
                collect_probes_recursive(
                    func,
                    i.alternative,
                    probes,
                    next_probe_id,
                    probe_to_loc,
                    lookup,
                    _body,
                    _code_section_offset,
                )?;
            }
            _ => {}
        }
    }

    Ok(())
}

struct DwarfLookup {
    rows: Vec<(u64, String, u32, u32)>,
}

impl DwarfLookup {
    fn new(dwarf: &gimli::Dwarf<gimli::EndianSlice<'static, RunTimeEndian>>) -> Self {
        let mut rows = Vec::new();
        let mut units = dwarf.units();
        while let Ok(Some(header)) = units.next() {
            if let Ok(unit) = dwarf.unit(header) {
                if let Some(program) = unit.line_program.clone() {
                    let mut program_rows = program.rows();
                    while let Ok(Some((_, row_ref))) = program_rows.next_row() {
                        let row = *row_ref;
                        let file_idx = row.file_index();
                        if let Some(file) = program_rows.header().file(file_idx) {
                            let path_name = file.path_name();
                            if let Ok(p) = dwarf.attr_string(&unit, path_name) {
                                let mut p_str = p.to_string_lossy().into_owned();

                                // Try to prepend directory
                                let dir_idx = file.directory_index();
                                if let Some(dir_val) = program_rows.header().directory(dir_idx) {
                                    if let Ok(dir_attr) = dwarf.attr_string(&unit, dir_val) {
                                        let d_str = dir_attr.to_string_lossy();
                                        if !d_str.is_empty() {
                                            let d_clean = d_str.replace('\\', "/");
                                            let p_clean = p_str.replace('\\', "/");
                                            // If filename is not absolute, prepend directory
                                            if !p_clean.starts_with('/') && !p_clean.contains(':') {
                                                if d_clean.ends_with('/') {
                                                    p_str = format!("{}{}", d_clean, p_clean);
                                                } else {
                                                    p_str = format!("{}/{}", d_clean, p_clean);
                                                }
                                            }
                                        }
                                    }
                                }

                                if !p_str.is_empty() {
                                    // Normalize slashes
                                    let mut final_path = p_str.replace('\\', "/");

                                    // Attempt to make relative to CWD (project root)
                                    if let Ok(cwd) = std::env::current_dir() {
                                        let cwd_str = cwd.to_string_lossy().replace('\\', "/");
                                        if final_path.starts_with(&cwd_str) {
                                            final_path = final_path[cwd_str.len()..]
                                                .trim_start_matches('/')
                                                .to_string();
                                        }
                                    }

                                    // Filter out external files (absolute paths that are not in CWD)
                                    if std::path::Path::new(&final_path).is_absolute() {
                                        continue;
                                    }

                                    // Filter out library/ paths (rust std lib stuff often appears as relative library/...)
                                    if final_path.starts_with("library/") {
                                        continue;
                                    }

                                    // Filter out files that don't exist locally
                                    // if !std::path::Path::new(&final_path).exists() {
                                    //     continue;
                                    // }

                                    let line = row.line().map(|l| l.get() as u32).unwrap_or(0);
                                    let col = match row.column() {
                                        gimli::ColumnType::LeftEdge => 0,
                                        gimli::ColumnType::Column(c) => c.get() as u32,
                                    };
                                    rows.push((row.address(), final_path, line, col));
                                }
                            }
                        }
                    }
                }
            }
        }
        rows.sort_by_key(|r| r.0);
        tracing::info!("DwarfLookup built with {} rows", rows.len());
        Self { rows }
    }

    fn lookup(&self, addr: u64) -> Option<(String, u32, u32)> {
        match self.rows.binary_search_by_key(&addr, |r| r.0) {
            Ok(idx) => {
                let r = &self.rows[idx];
                if r.2 == 0 {
                    None
                } else {
                    Some((r.1.clone(), r.2, r.3))
                }
            }
            Err(idx) => {
                if idx > 0 {
                    let r = &self.rows[idx - 1];
                    if r.2 == 0 {
                        None
                    } else {
                        Some((r.1.clone(), r.2, r.3))
                    }
                } else {
                    None
                }
            }
        }
    }
}

/// Serialize mapping and create coverage data
fn create_coverage_data(mapping: &HashMap<u32, (String, u32, u32)>) -> Result<(Vec<u8>, usize)> {
    let num_probes = mapping.len();
    let mut entries_data = Vec::with_capacity(num_probes * 64);

    // Sort IDs to ensure the mapping is ordered and we can't miss any in the hits buffer
    let mut ids: Vec<_> = mapping.keys().collect();
    ids.sort();

    for &id in ids {
        let (file, line, col) = &mapping[&id];
        let file_bytes = file.as_bytes();

        // Header: [ID: u32 | LINE: u32 | COL: u32 | PATH_LEN: u32] = 16 bytes
        entries_data.extend_from_slice(&id.to_le_bytes());
        entries_data.extend_from_slice(&line.to_le_bytes());
        entries_data.extend_from_slice(&col.to_le_bytes());
        entries_data.extend_from_slice(&(file_bytes.len() as u32).to_le_bytes());

        // Payload: [PATH_BYTES]
        entries_data.extend_from_slice(file_bytes);
    }

    let mut mapping_data = Vec::with_capacity(entries_data.len() + 8);
    mapping_data.extend_from_slice(&(num_probes as u32).to_le_bytes());
    mapping_data.extend_from_slice(&((8 + entries_data.len()) as u32).to_le_bytes());
    mapping_data.extend_from_slice(&entries_data);

    Ok((mapping_data, num_probes))
}

fn find_exported_global(module: &Module, name: &str) -> Option<walrus::GlobalId> {
    for global in module.globals.iter() {
        if let Some(g_name) = &global.name {
            if g_name == name {
                return Some(global.id());
            }
        }
    }

    for export in module.exports.iter() {
        if export.name == name {
            if let walrus::ExportItem::Global(gid) = export.item {
                return Some(gid);
            }
        }
    }

    None
}

fn get_global_val(module: &Module, global_id: walrus::GlobalId) -> Option<u32> {
    let global = module.globals.get(global_id);
    match &global.kind {
        walrus::GlobalKind::Local(ConstExpr::Value(ir::Value::I32(v))) => Some(*v as u32),
        _ => None,
    }
}

fn update_global_initializer(
    module: &mut Module,
    global_id: walrus::GlobalId,
    value: u32,
) -> Result<()> {
    let global = module.globals.get_mut(global_id);
    global.kind = walrus::GlobalKind::Local(ConstExpr::Value(ir::Value::I32(value as i32)));
    Ok(())
}

fn patch_global_ptr(module: &mut Module, global_id: walrus::GlobalId, value: u32) -> Result<()> {
    update_global_initializer(module, global_id, value)
}

/// Instrument functions with coverage probes
fn instrument_functions(
    module: &mut Module,
    func_to_probes: &HashMap<walrus::FunctionId, Vec<Option<u32>>>,
    hits_offset: u32,
) -> Result<()> {
    let memory_id = module
        .memories
        .iter()
        .next()
        .context("No memory found in WASM module")?
        .id();

    let mut probes_injected = 0;

    let mut func_ids: Vec<_> = func_to_probes.keys().copied().collect();
    func_ids.sort_by_key(|id| {
        let f = module.funcs.get(*id);
        let raw_name = f.name.as_deref().unwrap_or("unknown");
        let demangled = demangle(raw_name).to_string();
        (demangled, id.index())
    });

    for func_id in func_ids {
        let probes = &func_to_probes[&func_id];
        let func = module.funcs.get_mut(func_id);
        let local_func = match &mut func.kind {
            walrus::FunctionKind::Local(lf) => lf,
            _ => continue,
        };

        let mut probe_queue = VecDeque::from(probes.clone());
        let entry_block = local_func.entry_block();

        instrument_instr_seq(
            local_func,
            entry_block,
            &mut probe_queue,
            hits_offset,
            memory_id,
            &mut probes_injected,
        );
    }

    tracing::info!("Injected {} coverage probes", probes_injected);
    Ok(())
}

fn instrument_instr_seq(
    func: &mut walrus::LocalFunction,
    seq_id: walrus::ir::InstrSeqId,
    probes: &mut VecDeque<Option<u32>>,
    hits_offset: u32,
    memory_id: walrus::MemoryId,
    probes_injected: &mut u32,
) {
    let probe_id = probes.pop_front().flatten();

    if let Some(id) = probe_id {
        let builder = func.builder_mut();
        let mut seq = builder.instr_seq(seq_id);

        // Insert probe at start
        let instrs = seq.instrs_mut();

        let mut new_instrs = Vec::with_capacity(instrs.len() + 5);
        new_instrs.push((
            walrus::ir::Instr::Const(walrus::ir::Const {
                value: walrus::ir::Value::I32(hits_offset as i32 + id as i32),
            }),
            Default::default(),
        ));
        new_instrs.push((
            walrus::ir::Instr::Const(walrus::ir::Const {
                value: walrus::ir::Value::I32(1),
            }),
            Default::default(),
        ));
        new_instrs.push((
            walrus::ir::Instr::Store(walrus::ir::Store {
                memory: memory_id,
                kind: walrus::ir::StoreKind::I32_8 { atomic: false },
                arg: walrus::ir::MemArg {
                    offset: 0,
                    align: 1,
                },
            }),
            Default::default(),
        ));

        // Add original instructions
        new_instrs.append(instrs);
        *instrs = new_instrs;

        *probes_injected += 1;
    }

    // Now traverse instructions to find nested blocks
    let instrs: Vec<_> = {
        let builder = func.builder_mut();
        let seq = builder.instr_seq(seq_id);
        seq.instrs().to_vec()
    };

    for (instr, _) in instrs {
        match instr {
            walrus::ir::Instr::Block(b) => {
                instrument_instr_seq(func, b.seq, probes, hits_offset, memory_id, probes_injected);
            }
            walrus::ir::Instr::Loop(l) => {
                instrument_instr_seq(func, l.seq, probes, hits_offset, memory_id, probes_injected);
            }
            walrus::ir::Instr::IfElse(i) => {
                instrument_instr_seq(
                    func,
                    i.consequent,
                    probes,
                    hits_offset,
                    memory_id,
                    probes_injected,
                );
                instrument_instr_seq(
                    func,
                    i.alternative,
                    probes,
                    hits_offset,
                    memory_id,
                    probes_injected,
                );
            }
            _ => {}
        }
    }
}

/// Allocate a new data segment for the map data
fn allocate_data_segment(module: &mut Module, data: &[u8]) -> Result<u32> {
    // Find the first memory
    let memory_id = module
        .memories
        .iter()
        .next()
        .context("No memory found in WASM module")?
        .id();

    // Get current data size by examining existing data segments
    let mut max_offset = 0u32;
    for data_segment in module.data.iter() {
        if let walrus::DataKind::Active { memory: _, offset } = &data_segment.kind {
            let offset_val = match offset {
                ConstExpr::Value(ir::Value::I32(v)) => (*v).max(0) as u32,
                _ => continue,
            };
            let end = offset_val + data_segment.value.len() as u32;
            if end > max_offset {
                max_offset = end;
            }
        }
    }

    // Check __data_end to account for BSS (uninitialized statics)
    if let Some(data_end_id) = find_exported_global(module, "__data_end") {
        if let Some(val) = get_global_val(module, data_end_id) {
            if val > max_offset {
                tracing::info!(
                    "Adjusting max_offset from {} to {} based on __data_end",
                    max_offset,
                    val
                );
                max_offset = val;
            }
        }
    }

    // Align to 16 bytes
    let segment_offset = (max_offset + 15) & !15;

    // Ensure memory is large enough
    let required_bytes = segment_offset + data.len() as u32;
    let required_pages = required_bytes.div_ceil(65536) as u64;

    let memory = module.memories.get_mut(memory_id);

    // Add some padding for the heap (e.g. 16 pages = 1MB)
    let padding_pages = 16;
    let target_pages = required_pages + padding_pages;

    if memory.initial < target_pages {
        tracing::info!(
            "Expanding memory from {} to {} pages (including {} padding)",
            memory.initial,
            target_pages,
            padding_pages
        );
        memory.initial = target_pages;
    }
    if let Some(max) = memory.maximum {
        if max < target_pages {
            memory.maximum = Some(target_pages);
        }
    }

    // Add new data segment
    let offset_expr = ConstExpr::Value(ir::Value::I32(segment_offset as i32));
    module.data.add(
        walrus::DataKind::Active {
            memory: memory_id,
            offset: offset_expr,
        },
        data.to_vec(),
    );

    // Update __heap_base if it exists
    let new_end = segment_offset + data.len() as u32;

    if let Some(heap_base) = find_exported_global(module, "__heap_base") {
        update_global_initializer(module, heap_base, new_end)?;
        tracing::info!("Updated __heap_base to {}", new_end);
    } else {
        tracing::warn!("__heap_base not found, heap corruption likely!");
    }

    // Update __data_end if it exists
    if let Some(data_end) = find_exported_global(module, "__data_end") {
        update_global_initializer(module, data_end, new_end)?;
        tracing::info!("Updated __data_end to {}", new_end);
    }

    Ok(segment_offset)
}
