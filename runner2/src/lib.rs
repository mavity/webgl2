use base64::Engine;
use addr2line::Context;
use gimli::Dwarf;
use wasmparser;
use core::arch::wasm32;

extern "C" {
    fn eval(js_ptr: u32, scratch_ptr: u32);
}

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
        let snippet_s = r#"(function() {{
    const cp = require('child_process');
    const p  = require('path');
    // Inline worker script: parse args and run wasm with inspector coverage collection.
    const inline = `const fs = require('fs');
const path = require('path');
const inspector = require('inspector');
function postSync(session, method, params) {{
    const SAB = new SharedArrayBuffer(4);
    const ia = new Int32Array(SAB);
    ia[0] = 0;
    let result = null;
    let error = null;
    session.post(method, params, (err, res) => {{ if (err) error = err; else result = res; ia[0] = 1; Atomics.notify(ia, 0, 1); }});
    while (Atomics.load(ia, 0) === 0) Atomics.wait(ia, 0, 0, 1000);
    if (error) throw error; return result;
}

function runWorkerSync(opts) {{
    const target = opts.target_wasm;
    const out = opts.out || './coverage';
    if (!target || !fs.existsSync(target)) {{ return {{ exitCode: 2, error: `target wasm not found: ${target}` }}; }}
    fs.mkdirSync(out, {{ recursive: true }});
    const wasmBuf = fs.readFileSync(target);
    const module = new WebAssembly.Module(wasmBuf);
    const imports = {{ env: {{}} }};
    const instance = new WebAssembly.Instance(module, imports);
    const session = new inspector.Session();
    session.connect();
    try {{ postSync(session, 'Profiler.enable', {{}}); postSync(session, 'Profiler.startPreciseCoverage', {{ callCount: true, detailed: true }}); }} catch (e) {{ console.warn('[WorkerInline] inspector error:', e && e.message ? e.message : e); }}
    let exitCode = 0;
    try {{
        if (instance.exports.__run_tests) {{ const rc = instance.exports.__run_tests(); exitCode = Number(rc) || 0; }}
        else if (instance.exports._start) {{ try {{ instance.exports._start(); exitCode = 0; }} catch(e) {{ exitCode = 1; }} }}
        else if (instance.exports.main) {{ try {{ instance.exports.main(); exitCode = 0; }} catch(e) {{ exitCode = 1; }} }}
        else {{ exitCode = 1; }}
    }} catch (err) {{ console.error('[WorkerInline] Error running tests:', err && err.message ? err.message : err); exitCode = 1; }}
    let coverage = null;
    try {{ coverage = postSync(session, 'Profiler.takePreciseCoverage', {{}}); postSync(session, 'Profiler.stopPreciseCoverage', {{}}); session.disconnect(); }} catch(e) {{ console.warn('[WorkerInline] Failed to capture coverage:', e && e.message ? e.message : e); }}
    try {{ fs.writeFileSync(path.join(out, 'v8-coverage.json'), JSON.stringify(coverage, null, 2), 'utf8'); }} catch(e) {{ console.warn('[WorkerInline] write coverage error:', e && e.message ? e.message : e); }}
    return {{ exitCode: exitCode, coverageFile: path.join(out, 'v8-coverage.json'), coverage: coverage }};
};

// CLI argument parsing wrapper for inline '-e' script
const rawArgs = process.argv.slice(1); // slice(1) because with -e Node sets argv[1] as '-e'
let opts = {{ target_wasm: null, out: null, harness: null }};
for (let i = 0; i < rawArgs.length; i++) {{
    const a = rawArgs[i];
    if (a === '--target-wasm' && i + 1 < rawArgs.length) {{ opts.target_wasm = rawArgs[++i]; }}
    else if (a === '--out' && i + 1 < rawArgs.length) {{ opts.out = rawArgs[++i]; }}
    else if (a === '--harness' && i + 1 < rawArgs.length) {{ opts.harness = rawArgs[++i]; }}
}}
const result = runWorkerSync(opts);
console.log(JSON.stringify(result));`;

    const res = cp.spawnSync('node', ['-e', inline, '--target-wasm', '{TWASM}', '--out', '{OD}', '--harness', 'libtest'], {{ encoding: 'utf8' }});
    return {{
        exitCode: res.status,
        stdout: (res.stdout && res.stdout.toString()) || '',
        stderr: (res.stderr && res.stderr.toString()) || '',
        coverageFile: p.join('{OD}', 'v8-coverage.json')
    }};
    }} )()"#
        .replace("{TWASM}", t_wasm)
        .replace("{OD}", od);

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
