//! Coverage module for WASM instrumentation
//!
//! This module provides the runtime support for coverage tracking in WASM.

#[no_mangle]
pub static mut COV_HITS_PTR: *mut u8 = std::ptr::null_mut();

#[no_mangle]
pub static mut COV_MAP_PTR: *const u8 = std::ptr::null();

#[no_mangle]
pub static mut COV_MAP_LEN: usize = 0;

/// Initialize coverage hits buffer.
/// This must be called before any instrumented code runs.
#[no_mangle]
pub extern "C" fn wasm_init_coverage(size: usize) {
    unsafe {
        if !COV_HITS_PTR.is_null() {
            return;
        }
        // Allocate buffer using Rust's allocator
        let mut buf = vec![0; size];
        let ptr = buf.as_mut_ptr();
        std::mem::forget(buf); // Leak it so it lives forever
        COV_HITS_PTR = ptr;
        COV_HITS_LEN = size;
    }
}

#[no_mangle]
pub static mut COV_HITS_LEN: usize = 0;

/// Reset coverage hits to zero.
#[no_mangle]
pub extern "C" fn wasm_reset_coverage() {
    unsafe {
        if COV_HITS_PTR.is_null() || COV_HITS_LEN == 0 {
            return;
        }

        // Zero the entire hits buffer based on its allocated length.
        std::ptr::write_bytes(COV_HITS_PTR, 0, COV_HITS_LEN);
        // Also clear the cached report
        if let Ok(mut report) = LCOV_REPORT.lock() {
            *report = None;
        }
    }
}

use std::sync::Mutex;

// Use a static mutex to store the report
static LCOV_REPORT: Mutex<Option<String>> = Mutex::new(None);

/// Get LCOV report from coverage data.
/// Returns a pointer to a UTF-8 encoded LCOV string.
/// The string is stored in a static variable to avoid memory leaks.
#[no_mangle]
pub extern "C" fn wasm_get_lcov_report_ptr() -> *const u8 {
    unsafe {
        if COV_MAP_PTR.is_null() || COV_HITS_PTR.is_null() {
            return std::ptr::null();
        }

        // Mapping data is in COV_MAP_PTR with length COV_MAP_LEN
        let mapping_data = std::slice::from_raw_parts(COV_MAP_PTR, COV_MAP_LEN);

        // Hits data is in COV_HITS_PTR.
        // We need to know the size. It should match the number of entries in mapping.
        // But for now, let's assume the mapping header tells us enough?
        // The mapping header has `num_entries`.
        if mapping_data.len() < 8 {
            return std::ptr::null();
        }

        let num_entries = u32::from_le_bytes([
            mapping_data[0],
            mapping_data[1],
            mapping_data[2],
            mapping_data[3],
        ]) as usize;
        let hit_data = std::slice::from_raw_parts(COV_HITS_PTR, num_entries);

        // Generate LCOV report
        let lcov = generate_lcov_report(mapping_data, hit_data);

        // Store in static variable
        let mut report = LCOV_REPORT.lock().unwrap();
        *report = Some(lcov);

        // Return pointer to the stored string
        report.as_ref().unwrap().as_ptr()
    }
}

/// Get the length of the LCOV report.
#[no_mangle]
pub extern "C" fn wasm_get_lcov_report_len() -> usize {
    let report = LCOV_REPORT.lock().unwrap();
    report.as_ref().map(|s| s.len()).unwrap_or(0)
}

/// Generate LCOV formatted report from mapping and hit data
fn generate_lcov_report(mapping_data: &[u8], hit_data: &[u8]) -> String {
    use std::collections::HashMap;

    let mut report = String::new();
    let mut file_coverage: HashMap<String, Vec<(u32, u32, bool)>> = HashMap::new();

    // Parse mapping entries
    // Header: [ num_entries (4 bytes) | mapping_size (4 bytes) ]
    // Entries start at offset 8
    let mut offset = 8;
    let mut id = 0u32;

    while offset < mapping_data.len() {
        if offset + 10 > mapping_data.len() {
            break;
        }

        // Read id (4 bytes)
        let _entry_id = u32::from_le_bytes([
            mapping_data[offset],
            mapping_data[offset + 1],
            mapping_data[offset + 2],
            mapping_data[offset + 3],
        ]);
        offset += 4;

        // Read line number (4 bytes)
        let line = u32::from_le_bytes([
            mapping_data[offset],
            mapping_data[offset + 1],
            mapping_data[offset + 2],
            mapping_data[offset + 3],
        ]);
        offset += 4;

        // Read column number (4 bytes)
        let col = u32::from_le_bytes([
            mapping_data[offset],
            mapping_data[offset + 1],
            mapping_data[offset + 2],
            mapping_data[offset + 3],
        ]);
        offset += 4;

        // Read file_len (4 bytes)
        let file_len = u32::from_le_bytes([
            mapping_data[offset],
            mapping_data[offset + 1],
            mapping_data[offset + 2],
            mapping_data[offset + 3],
        ]) as usize;
        offset += 4;

        if offset + file_len > mapping_data.len() {
            break;
        }

        // Read file string
        let file_bytes = &mapping_data[offset..offset + file_len];
        let file = String::from_utf8_lossy(file_bytes).to_string();
        offset += file_len;

        // Check hit status
        let hit = if (id as usize) < hit_data.len() {
            hit_data[id as usize] > 0
        } else {
            false
        };

        file_coverage
            .entry(file)
            .or_default()
            .push((line, col, hit));

        id += 1;
    }

    // Format LCOV
    for (file, entries) in file_coverage {
        report.push_str(&format!("SF:{}\n", file));

        // Group by line for DA records
        let mut line_hits: HashMap<u32, u32> = HashMap::new();
        for (line, _col, hit) in &entries {
            if *hit {
                *line_hits.entry(*line).or_default() += 1;
            } else {
                line_hits.entry(*line).or_default();
            }
        }

        for (line, hits) in line_hits {
            report.push_str(&format!("DA:{},{}\n", line, hits));
        }

        // Output branch data (BRDA) using column as branch ID
        // BRDA:<line>,<block>,<branch>,<hits>
        // We'll use column as a proxy for branch ID to disambiguate multiple branches on same line
        for (line, col, hit) in entries {
            report.push_str(&format!(
                "BRDA:{},0,{},{}\n",
                line,
                col,
                if hit { "1" } else { "-" }
            ));
        }

        report.push_str("end_of_record\n");
    }

    report
}
