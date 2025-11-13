use base64::Engine;
use addr2line::Context;
use gimli::Dwarf;
use wasmparser;
use core::arch::wasm32;

extern "C" {
    fn eval(js_ptr: u32, scratch_ptr: u32);
}

// Embed the satellite script into the binary. The runtime will still only need
// `runner.js` and `runner.wasm` — no separate `runner_satellite.js` at runtime.
const RUNNER_SATELLITE_JS: &str = include_str!("runner_satellite.js");

// Safe helper: return the total memory size in bytes
fn wasm_memory_len_bytes() -> usize {
    (wasm32::memory_size(0) as usize) * 65536
}

unsafe fn read_cstr(ptr: u32) -> Option<String> {
    if ptr == 0 { return None; }
    let start = ptr as usize;
    let mem_len = wasm_memory_len_bytes();
    if start >= mem_len { return None; }
    // Create slice from start to end of memory and scan for NUL
    let slice = core::slice::from_raw_parts(start as *const u8, mem_len - start);
    if let Some(pos) = slice.iter().position(|&b| b == 0) {
        let bytes = &slice[..pos];
        match String::from_utf8(bytes.to_vec()) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    } else {
        None
    }
}

// Keep write_cstr available for legacy callers, but don't use it for eval data.
// write_cstr removed: eval uses heap buffers. Keep a safe wrapper if needed later.

// Safe high-level helper: read a NUL-terminated string from memory as String (no unsafe outside)
fn safe_read_cstr(ptr: u32) -> Option<String> {
    unsafe { read_cstr(ptr) }
}

// Parse argv_ptr into Vec<String> by reading NUL-terminated args from memory.
fn parse_argv_from_ptr(argv_ptr: u32) -> Vec<String> {
    if argv_ptr == 0 { return Vec::new(); }
    let start = argv_ptr as usize;
    let mem_len = wasm_memory_len_bytes();
    if start >= mem_len { return Vec::new(); }
    let slice = unsafe { core::slice::from_raw_parts(start as *const u8, mem_len - start) };
    let mut args = Vec::new();
    let mut i = 0usize;
    while i < slice.len() {
        if slice[i] == 0 { i += 1; continue; }
        let pos = match slice[i..].iter().position(|&b| b == 0) {
            Some(p) => i + p,
            None => slice.len(),
        };
        let part = &slice[i..pos];
        let s = String::from_utf8_lossy(part).into_owned();
        args.push(s);
        i = pos + 1;
    }
    args
}

// Evaluate JS by allocating code and scratch on the module heap and calling eval(code_ptr, scratch_ptr).
const MAX_SCRATCH: usize = 16 * 1024 * 1024; // 16 MiB
fn eval_js_on_heap(js: &str, mut scratch_size: usize) -> Option<String> {
    if scratch_size == 0 { scratch_size = 1024; }
    loop {
        // Prepare code buffer
        let mut code_buf = js.as_bytes().to_vec();
        if code_buf.last().copied() != Some(0) { code_buf.push(0); }
        // Scratch buffer
        let mut scratch = vec![0u8; scratch_size];
        let code_ptr = code_buf.as_ptr() as u32;
        let scratch_ptr = scratch.as_mut_ptr() as u32;
        unsafe { eval(code_ptr, scratch_ptr); }
        if let Some(res) = safe_read_cstr(scratch_ptr) {
            if res.len() >= scratch_size.saturating_sub(1) && scratch_size < MAX_SCRATCH {
                scratch_size = (scratch_size * 2).min(MAX_SCRATCH);
                continue;
            }
            return Some(res);
        } else {
            if scratch_size < MAX_SCRATCH {
                scratch_size = (scratch_size * 2).min(MAX_SCRATCH);
                continue;
            }
            return None;
        }
    }
}

// helper functions
fn find_flag_value(args: &[String], flag: &str) -> Option<String> {
    for i in 0..args.len() {
        if args[i] == flag && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

// replace older index-based parsing with scan-based parsing
#[no_mangle]
pub extern "C" fn run(argv_ptr: u32) -> i32 {
    // parse argv into heap-allocated Vec<String> and never use argv_ptr again
    let mut args = parse_argv_from_ptr(argv_ptr);

    // previous code used to assume args[0] / args[1] were already stripped;
    // now we just scan safely for our flags:

    // parse flags irrespective of position
    let target_wasm = find_flag_value(&args, "--target-wasm");
    let out_dir = find_flag_value(&args, "--out").unwrap_or_else(|| "coverage".to_string());

    // if not provided, compute defaults relative to argv_ptr to avoid memory overlap
    // eval will use heap-allocated buffers; code_ptr/scratch_ptr defaults should not be used for eval.
    let _scratch_ptr = find_flag_value(&args, "--scratch-ptr")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0u32);

    let _code_ptr = find_flag_value(&args, "--code-ptr")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0u32);

    let verbose = has_flag(&args, "--verbose");

    // Compose a JS snippet that will be executed by the host's eval() in runner.js scope.
    // The JS code will spawn a worker process to run the tests, and return a JSON string describing results.
    let t_wasm = target_wasm.as_deref().unwrap_or_default();
    let od = out_dir.as_str();
        // Inline worker script (no external worker.js dependency). We spawn node -e '<script>'
        // with args for CLI options. This allows the runner to work with only `runner.js` and `runner.wasm`.
                let snippet_s = r#"(function() {
        const cp = require('child_process');
        const p  = require('path');

        // Child Node script run as a satellite process.
        // This script uses the Inspector protocol to capture V8 precise coverage while it runs
        // the supplied WASM module (libtest harness or equivalent). It writes coverage JSON
        // to the provided output path and exits with the child's exit code.
        const inline_script = {SAT_JS};
        const res = cp.spawnSync('node', ['-', '--target-wasm', '{TWASM}', '--out', '{OD}', '--harness', 'libtest'], { encoding: 'utf8', input: inline_script, maxBuffer: 50 * 1024 * 1024 });
        return {
                exitCode: res.status,
                stdout: (res.stdout && res.stdout.toString()) || '',
                stderr: (res.stderr && res.stderr.toString()) || '',
                coverageFile: p.join('{OD}', 'v8-coverage.json')
        };
} )()"#
        .replace("{TWASM}", t_wasm)
        .replace("{OD}", od)
        .replace("{SAT_JS}", serde_json::to_string(RUNNER_SATELLITE_JS).unwrap().as_str());

    let result_json = eval_js_on_heap(&snippet_s, 8 * 1024);
    if let Some(res) = result_json {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&res) {
            // Result v may contain coverage directly or inside `result` (from our wrapper in JS eval)
            let mut coverage_val = None;
            if let Some(cov) = v.get("coverage") { coverage_val = Some(cov.clone()); }
            else if let Some(rr) = v.get("result") { if let Some(cf) = rr.get("coverage") { coverage_val = Some(cf.clone()); } }
            else if let Some(rr) = v.get("result") { if let Some(cf) = rr.get("coverageFile") { coverage_val = Some(serde_json::Value::String(cf.as_str().unwrap_or_default().to_string())); } }
            // coverage_val might be an actual coverage JSON, or a file path to coverage JSON
            let mut coverage_json_opt: Option<serde_json::Value> = None;
            if let Some(cov) = coverage_val {
                match cov {
                    serde_json::Value::String(path) => {
                        // Read the coverage file via JS eval using heap buffers
                        let read_cov_cmd = format!("(function(){{ const fs=require('fs'); const s=fs.readFileSync('{}','utf8'); return JSON.stringify({{ content: s }}); }})()", path);
                        if let Some(read_cov_res) = eval_js_on_heap(&read_cov_cmd, 64 * 1024) {
                            if let Ok(read_cov_val) = serde_json::from_str::<serde_json::Value>(&read_cov_res) {
                                if let Some(content) = read_cov_val.get("content") {
                                    if let Some(content_s) = content.as_str() {
                                        if let Ok(parsed_cov) = serde_json::from_str::<serde_json::Value>(content_s) {
                                            coverage_json_opt = Some(parsed_cov);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    serde_json::Value::Object(_) => {
                        coverage_json_opt = Some(cov.clone());
                    }
                    _ => {}
                }
            }
            if let Some(coverage_json) = coverage_json_opt {
                // Collect offsets
                let mut offsets: Vec<u64> = Vec::new();
                if let Some(result_arr) = coverage_json.get("result").and_then(|r| r.as_array()) {
                    for script in result_arr {
                        if let Some(url) = script.get("url").and_then(|u| u.as_str()) {
                            if url.starts_with("wasm://") || url.starts_with("file://") {
                                if let Some(funcs) = script.get("functions").and_then(|f| f.as_array()) {
                                    for func in funcs {
                                        if let Some(ranges) = func.get("ranges").and_then(|r| r.as_array()) {
                                            for range in ranges {
                                                if let Some(count) = range.get("count") {
                                                    if count.as_i64().unwrap_or(0) > 0 {
                                                        if let Some(start) = range.get("startOffset") {
                                                            offsets.push(start.as_u64().unwrap_or(0));
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
                }

                // Read wasm bytes via host by executing a JS snippet that reads and base64-encodes the file.
                let read_cmd_s = format!("(function(){{ const fs=require('fs'); const data=fs.readFileSync('{}'); return {{ content: Buffer.from(data).toString('base64') }}; }})()", t_wasm);
                let wasm_read = eval_js_on_heap(&read_cmd_s, 64 * 1024);
                if let Some(wr) = wasm_read {
                    if let Ok(wrv) = serde_json::from_str::<serde_json::Value>(&wr) {
                        if let Some(content_b64) = wrv.get("content").and_then(|c| c.as_str()) {
                            if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(content_b64) {
                                if let Ok(lcov) = symbolicate_offsets(&decoded, &offsets) {
                                    let write_cmd_s = format!("(function(){{ const fs=require('fs'); const p=require('path'); fs.mkdirSync(p.dirname('{}'), {{ recursive: true }}); fs.writeFileSync('{}', `{}`,'utf8'); return {{ ok: true }}; }})()", format!("{}/coverage.lcov", out_dir), format!("{}/coverage.lcov", out_dir), lcov.replace('`', "\\`") );
                                    let _ = eval_js_on_heap(&write_cmd_s, 8 * 1024);
                                }
                            }
                        }
                    }
                }

            }
            // Detect exit code either directly on v or nested under result
            if let Some(exit) = v.get("exitCode").or_else(|| v.get("result").and_then(|r| r.get("exitCode"))) {
                if let Some(exit_code) = exit.as_i64() {
                    return exit_code as i32;
                }
            }
        }
    }
    0
}

fn symbolicate_offsets(wasm_bytes: &[u8], offsets: &Vec<u64>) -> Result<String, String> {
    // Try to extract DWARF debug sections from the WASM binary using wasmparser
    let mut dwarf_sections: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();
    for payload in wasmparser::Parser::new(0).parse_all(wasm_bytes) {
        match payload {
            Ok(wasmparser::Payload::CustomSection(reader)) => {
                let name = reader.name().to_string();
                if name.starts_with(".debug_") {
                    dwarf_sections.insert(name, reader.data().to_vec());
                }
            }
            _ => {}
        }
    }

    if dwarf_sections.is_empty() {
        // No DWARF information available; produce minimal LCOV entries mapping offsets to unknown files
        let mut lcov = String::new();
        for off in offsets {
            lcov.push_str("SF:unknown\n");
            lcov.push_str(&format!("DA:{},1\n", off));
            lcov.push_str("end_of_record\n");
        }
        return Ok(lcov);
    }

    // Leak the section data to get &'static slices for gimli endians
    let mut leaked_sections: std::collections::HashMap<String, &'static [u8]> = std::collections::HashMap::new();
    for (k, v) in dwarf_sections.into_iter() {
        let leaked = Box::leak(v.into_boxed_slice());
        leaked_sections.insert(k, leaked);
    }

    type DwarfReader = gimli::EndianSlice<'static, gimli::RunTimeEndian>;
    let mut load_section = |id: gimli::SectionId| -> Result<DwarfReader, gimli::read::Error> {
        let data = leaked_sections.get(id.name()).map(|x| *x).unwrap_or(&[]);
        Ok(gimli::EndianSlice::new(data, gimli::RunTimeEndian::Little))
    };
    let dwarf = match Dwarf::load(&mut load_section) {
        Ok(d) => d,
        Err(e) => return Err(format!("gimli dwarf load error: {:?}", e)),
    };
    let ctx = match Context::from_dwarf(dwarf) {
        Ok(c) => c,
        Err(e) => return Err(format!("addr2line context error: {}", e)),
    };

    let mut lcov = String::new();
    // LCOV header
    for off in offsets {
        // Map offset to location
        match ctx.find_location(*off) {
            Ok(Some(loc)) => {
                let file = loc.file.as_deref().unwrap_or("unknown");
                let line = loc.line.unwrap_or(0);
                // Append a simple LCOV entry: file coverage line
                lcov.push_str(&format!("SF:{}\n", file));
                lcov.push_str(&format!("DA:{},1\n", line));
                lcov.push_str("end_of_record\n");
            }
            Ok(None) => {
                // Unknown
            }
            Err(e) => {
                return Err(format!("location lookup error: {}", e));
            }
        }
    }
    Ok(lcov)
}

// Conventional Rust unit tests for normal development
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_flag_value() {
        let args = vec!["prog".to_string(), "--target-wasm".to_string(), "foo.wasm".to_string(), "--out".to_string(), "outdir".to_string()];
        assert_eq!(find_flag_value(&args, "--target-wasm"), Some("foo.wasm".to_string()));
        assert_eq!(find_flag_value(&args, "--out"), Some("outdir".to_string()));
        assert_eq!(find_flag_value(&args, "--notthere"), None);
    }

    #[test]
    fn test_has_flag() {
        let args = vec!["a".to_string(), "--verbose".to_string(), "b".to_string()];
        assert!(has_flag(&args, "--verbose"));
        assert!(!has_flag(&args, "--noway"));
    }

    #[test]
    fn test_symbolicate_offsets_empty() {
        let res = symbolicate_offsets(&[], &vec![10, 20]);
        assert!(res.is_ok());
        let lcov = res.unwrap();
        assert!(lcov.contains("SF:unknown"));
        assert!(lcov.contains("DA:10,1") || lcov.contains("DA:20,1"));
    }

    // Trial end-to-end test that will compile in wasm test build run by the runner.
    #[test]
    fn runner_wasm_test_example() {
        // Simple assertion to provide a detectable test in wasm built test harness.
        assert_eq!(2 + 2, 4);
    }
}
