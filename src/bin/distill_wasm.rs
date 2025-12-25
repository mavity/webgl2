//! WebGL2 WASM Coverage Distiller
//!
//! This tool instruments WASM binaries with coverage tracking.
//! It uses DWARF debug info to map instrumentation points to source lines.

use anyhow::{Context, Result};
use clap::Parser;
use gimli::{RunTimeEndian, SectionId};
use rustc_demangle::demangle;
use std::collections::{HashMap, HashSet};
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
    let (func_to_probe, probe_to_loc) = build_coverage_mapping(&wasm_bytes, &module)?;
    tracing::info!("Found {} instrumentation points", probe_to_loc.len());

    // Step 2: Serialize mapping
    let (mapping_data, _hit_size) = create_coverage_data(&probe_to_loc)?;

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
    let hits_size = probe_to_loc.len();
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

    instrument_functions(&mut module, &func_to_probe, hits_offset)?;

    // Write output
    module.emit_wasm_file(&output_path)?;
    tracing::info!("Wrote instrumented WASM to {:?}", output_path);

    Ok(())
}

/// Build mapping of instrumentation IDs to source locations.
/// Returns (FuncId -> ProbeId, ProbeId -> (File, Line))
fn build_coverage_mapping(
    wasm_bytes: &[u8],
    module: &Module,
) -> Result<(
    HashMap<walrus::FunctionId, u32>,
    HashMap<u32, (String, u32)>,
)> {
    let mut func_to_probe = HashMap::new();
    let mut probe_to_loc = HashMap::new();

    let parser = wasmparser::Parser::new(0);
    let mut sections = HashMap::new();
    let mut code_section_start = 0;
    let mut function_offsets = Vec::new();

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
                function_offsets.push(body.range().start);
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

    let mut probe_id = 0;

    for (i, (id, _)) in module.funcs.iter_local().enumerate() {
        let func = module.funcs.get(id);
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

        let mut path = "unknown.rs".to_string();
        let mut line = 1;
        let mut found_dwarf = false;
        let mut should_skip_function = false;

        if let Some(ref dwarf) = dwarf {
            if i < function_offsets.len() {
                let func_offset = function_offsets[i];
                let addr = (func_offset - code_section_start) as u64;

                let mut units = dwarf.units();
                'dwarf_loop: while let Ok(Some(header)) = units.next() {
                    if let Ok(unit) = dwarf.unit(header) {
                        if let Some(program) = unit.line_program.clone() {
                            let mut rows = program.rows();
                            while let Ok(Some((_, row_ref))) = rows.next_row() {
                                if row_ref.address() >= addr {
                                    let row = row_ref.clone();
                                    let file_idx = row.file_index();
                                    if let Some(file) = rows.header().file(file_idx) {
                                        let path_name = file.path_name();
                                        if let Ok(p) = dwarf.attr_string(&unit, path_name) {
                                            let p_str = p.to_string_lossy();
                                            if !p_str.is_empty() {
                                                let p_owned = p_str.into_owned();
                                                // Filter by path - BLOCKLIST
                                                if p_owned.contains("library/std")
                                                    || p_owned.contains("dlmalloc")
                                                    || p_owned.contains("coverage.rs")
                                                {
                                                    should_skip_function = true;
                                                    break 'dwarf_loop;
                                                }

                                                path = p_owned;
                                                if let Some(l) = row.line() {
                                                    line = l.get() as u32;
                                                }
                                                found_dwarf = true;
                                                break 'dwarf_loop;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if should_skip_function {
            continue;
        }

        // If we didn't find DWARF, or if we did but it's not clearly excluded yet,
        // apply strict filtering.

        // ALLOWLIST CHECK
        let is_whitelisted = if found_dwarf {
            // If we have a path, it MUST look like our project code.
            // We assume our code is in the current directory structure.
            // Matches:
            // - "src/..."
            // - ".../webgl2/src/..."
            // - "...\\webgl2\\src\\..."
            let p = path.replace('\\', "/");
            (p.contains("webgl2") || p.starts_with("src/") || p.contains("lib.rs"))
                && !p.contains("/registry/")
                && !p.contains("/.cargo/")
        } else {
            // If no DWARF, rely on symbol name
            demangled.contains("webgl2")
        };

        if !is_whitelisted {
            continue;
        }

        // Double check exclusion (redundant but safe)
        if path.contains("coverage.rs") || demangled.contains("coverage") {
            continue;
        }

        tracing::debug!("Instrumenting: {} ({})", demangled, path);

        func_to_probe.insert(id, probe_id);
        probe_to_loc.insert(probe_id, (path, line));
        probe_id += 1;
    }

    Ok((func_to_probe, probe_to_loc))
}

fn build_function_coverage_map_heuristic(module: &Module) -> HashMap<u32, (String, u32)> {
    let mut map = HashMap::new();
    let mut probe_id = 0;

    for (id, _) in module.funcs.iter_local() {
        let func = module.funcs.get(id);
        let name = func.name.as_deref().unwrap_or("unknown");

        let mut path = "unmapped_function.rs".to_string();
        let mut line = 1;

        if name != "unknown" {
            let clean_name = if name.starts_with("_ZN") { name } else { name };

            if clean_name.contains("webgl2") {
                let parts: Vec<&str> = clean_name.split("::").collect();
                for part in parts {
                    if part.starts_with("webgl2_") {
                        path = format!("src/{}.rs", part);
                        line = 100;
                        break;
                    }
                }
            }
        }

        map.insert(probe_id, (path, line));
        probe_id += 1;
    }

    map
}

/// Serialize mapping and create coverage data
fn create_coverage_data(mapping: &HashMap<u32, (String, u32)>) -> Result<(Vec<u8>, usize)> {
    // Serialize mapping entries
    let mut ids: Vec<u32> = mapping.keys().copied().collect();
    ids.sort();

    let num_entries = ids.len() as u32;

    let mut entries_data = Vec::new();
    for id in ids {
        let (file, line) = mapping.get(&id).unwrap();

        entries_data.extend_from_slice(&id.to_le_bytes());
        entries_data.extend_from_slice(&line.to_le_bytes());

        let file_bytes = file.as_bytes();
        entries_data.extend_from_slice(&(file_bytes.len() as u32).to_le_bytes());
        entries_data.extend_from_slice(file_bytes);
    }

    let mapping_size = 8 + entries_data.len(); // 8 bytes for header
    let mut mapping_data = Vec::with_capacity(mapping_size);

    mapping_data.extend_from_slice(&num_entries.to_le_bytes());
    mapping_data.extend_from_slice(&(mapping_size as u32).to_le_bytes());
    mapping_data.extend_from_slice(&entries_data);

    let hit_size = mapping.len();

    Ok((mapping_data, hit_size))
}

/// Allocate hits segment in WASM memory
fn allocate_hits_segment(module: &mut Module, size: usize) -> Result<u32> {
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

    // Align to 16 bytes
    let segment_offset = (max_offset + 15) & !15;

    // Ensure memory is large enough
    let required_bytes = segment_offset + size as u32;
    let required_pages = ((required_bytes + 65535) / 65536) as u64;

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

    // Add new data segment (zeros)
    let offset_expr = ConstExpr::Value(ir::Value::I32(segment_offset as i32));
    module.data.add(
        walrus::DataKind::Active {
            memory: memory_id,
            offset: offset_expr,
        },
        vec![0u8; size],
    );

    // Update __heap_base if it exists
    let new_end = segment_offset + size as u32;

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

fn patch_global_len(module: &mut Module, global_id: walrus::GlobalId, value: u32) -> Result<()> {
    update_global_initializer(module, global_id, value)
}

/// Instrument functions with coverage probes
fn instrument_functions(
    module: &mut Module,
    func_to_probe: &HashMap<walrus::FunctionId, u32>,
    hits_offset: u32,
) -> Result<()> {
    let memory_id = module
        .memories
        .iter()
        .next()
        .context("No memory found in WASM module")?
        .id();

    let mut probes_injected = 0;

    for (func_id, probe_id) in func_to_probe {
        let func = module.funcs.get_mut(*func_id);
        let local_func = match &mut func.kind {
            walrus::FunctionKind::Local(lf) => lf,
            _ => continue,
        };

        let entry_block = local_func.entry_block();
        let builder = local_func.builder_mut();
        let mut seq = builder.instr_seq(entry_block);

        // Insert probe at start
        // i32.const $hits_offset
        seq.i32_const(hits_offset as i32);
        // i32.const $offset
        seq.i32_const(*probe_id as i32);
        // i32.add
        seq.binop(walrus::ir::BinaryOp::I32Add);
        // i32.const 1
        seq.i32_const(1);
        // i32.store8
        seq.store(
            memory_id,
            walrus::ir::StoreKind::I32_8 { atomic: false },
            walrus::ir::MemArg {
                offset: 0,
                align: 1,
            },
        );

        // Move the 5 injected instructions to the beginning of the block
        // The block originally contained [Body...]. We appended [Probe...].
        // We want [Probe..., Body...].
        // So we rotate right by 5.
        seq.instrs_mut().rotate_right(5);

        probes_injected += 1;
    }

    tracing::info!("Injected {} coverage probes", probes_injected);
    Ok(())
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
    let required_pages = ((required_bytes + 65535) / 65536) as u64;

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
