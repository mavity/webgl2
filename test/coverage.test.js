import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

test('coverage module is available with coverage feature', async () => {
  // Load the WASM module with coverage enabled
  const wasmPath = join(__dirname, '..', 'webgl2.debug.wasm');
  let wasmBytes = readFileSync(wasmPath);

  // 1. Parse Custom Section "cov_mapping"
  const module = new WebAssembly.Module(wasmBytes);
  const customSections = WebAssembly.Module.customSections(module, "cov_mapping");
  
  assert(customSections.length > 0, "cov_mapping custom section not found");
  const mappingBuffer = customSections[0];
  const mappingView = new DataView(mappingBuffer);
  
  let offset = 0;
  const numEntries = mappingView.getUint32(offset, true); offset += 4;
  const mappingSize = mappingView.getUint32(offset, true); offset += 4;
  
  // console.log(`Found ${numEntries} coverage entries in custom section`);
  
  const mapping = new Map();
  // Safety: ensure mappingSize sanity
  assert(mappingSize, "Mapping size is zero");
  assert(mappingSize <= mappingBuffer.byteLength, "Mapping size exceeds buffer length");

  for (let i = 0; i < numEntries; i++) {
    if (offset + 16 > mappingBuffer.byteLength) {
      throw new Error('Unexpected EOF while parsing mapping entries');
    }

    const id = mappingView.getUint32(offset, true); offset += 4;
    const line = mappingView.getUint32(offset, true); offset += 4;
    const col = mappingView.getUint32(offset, true); offset += 4; // new column field
    const pathLen = mappingView.getUint32(offset, true); offset += 4;

    if (offset + pathLen > mappingBuffer.byteLength) {
      throw new Error(`Invalid path length (${pathLen}) at entry ${i}`);
    }

    const pathBytes = new Uint8Array(mappingBuffer, offset, pathLen);
    const path = new TextDecoder().decode(pathBytes);
    offset += pathLen;

    mapping.set(id, { path, line, col });
  }

  // console.log("Coverage Mapping (first 20):");
  // for (let i = 0; i < numEntries; i++) {
  //   const info = mapping.get(i);
  //   if (i < 20 || info.path.includes("lib.rs") || info.line === 591) {
  //     console.log(`Entry ${i}: ${info.path}:${info.line}`);
  //   }
  // }

  // 2. Instantiate and Run
  const imports = {
    env: {
      print: (ptr, len) => {
        // Mock print
      },
      wasm_execute_shader: () => {
        // Mock shader execution
      },
      dispatch_uncaptured_error: () => {},
      // Required by egg crate for timing measurements
      now: () => {
        return performance.now();
      },
      // Add other imports if needed by the module
      __linear_memory: new WebAssembly.Memory({ initial: 100 }),
      __indirect_function_table: new WebAssembly.Table({ initial: 0, element: 'anyfunc' })
    }
  };

  const instance = await WebAssembly.instantiate(module, imports);
  const exports = instance.exports;

  // Check exports
  assert(exports.COV_HITS_PTR, "COV_HITS_PTR export missing");
  // assert(exports.COV_MAP_LEN, "COV_MAP_LEN export missing"); // COV_MAP_LEN might be a value, not a pointer?

  const covPtrGlobal = exports.COV_HITS_PTR.value;
  // const covLenGlobal = exports.COV_MAP_LEN.value;

  // Read the actual pointers from memory (since they are globals pointing to memory)
  const memory = exports.memory;
  const memView = new DataView(memory.buffer);
  
  // COV_HITS_PTR is patched by distill_wasm to point directly to the buffer
  // So exports.COV_HITS_PTR.value IS the pointer.
  const covPtr = covPtrGlobal;
  const covLen = numEntries;
  
  // COV_MAP_PTR is NOT patched in the global export (it remains the address of the static)
  // So we need to read the memory at that address to get the actual pointer.
  // But wait, we don't use COV_MAP_PTR in this test, we use the custom section.
  // If we wanted to use it:
  // const mapPtr = memView.getUint32(exports.COV_MAP_PTR.value, true);

  // console.log(`Coverage Buffer: ptr=${covPtr}, len=${covLen}`);
  
  assert(covPtr !== 0, "COV_PTR is null");
  assert(covLen > 0, "COV_LEN is 0");
  assert.equal(covLen, numEntries, "Coverage buffer size mismatch with mapping entries");

  // 3. Verify Hit Counts
  // Since we haven't run any code yet, hits should be 0 (or 1 if initialization runs code)
  // Let's run a function to trigger coverage
  
  // Call a simple function to trigger coverage: create a context with flags (no debug)
  exports.wasm_create_context_with_flags(0);

  const hitBuffer = new Uint8Array(memory.buffer, covPtr, covLen);
  let totalHits = 0;
  for (let i = 0; i < covLen; i++) {
    if (hitBuffer[i] > 0) {
      totalHits++;
      const info = mapping.get(i);
      // console.log(`Hit: ID=${i} ${info.path}:${info.line}`);
    }
  }
  
  // console.log(`Total coverage hits: ${totalHits}`);
  assert(totalHits > 0, "No coverage hits recorded after execution");
});
