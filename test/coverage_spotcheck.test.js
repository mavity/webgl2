import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';


test('coverage: createTexture triggers coverage in texture-related code', async () => {
  const gl = await webGL2({ debug: true });
  
  // Check if coverage is available
  const instance = gl._instance;
  assert(instance, 'WASM instance not found on context');

  assert(instance.exports.COV_HITS_PTR, 'COV_HITS_PTR export not found on instance');
    
  // Execute the API that should trigger coverage
  const texture = gl.createTexture();
    
  // Get coverage report
  const fs = await import('fs');
  const path = await import('path');
  const { fileURLToPath } = await import('url');
  const wasmPath = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'webgl2.debug.wasm');
  const wasmBytes = fs.readFileSync(wasmPath);
  const module = new WebAssembly.Module(wasmBytes);

  const hits = getCoverageData(gl._instance, module);
  
  const uniquePaths = [...new Set(hits.map(h => h.path))];
  console.log('Unique paths hit:', uniquePaths);
    
  assert(hits && hits.length > 0, 'Coverage hits should be recorded');
    
  // Debug: print some hits
  // console.log('Sample hits:', hits.slice(0, 10).map(h => `${h.path.split(/[\\/]/).pop()}:${h.line}:${h.col}`));

  // Check if we hit something in webgl2_context
  const textureHits = hits.filter(h => h.path.includes('webgl2_context'));
  assert(textureHits.length > 0, 'Should hit code in webgl2_context');
});

function getCoverageData(instance, module) {
  // 1. Parse Custom Section "cov_mapping"
  const customSections = WebAssembly.Module.customSections(module, "cov_mapping");
  if (customSections.length === 0) return null;

  const mappingBuffer = customSections[0];
  const mappingView = new DataView(mappingBuffer);

  let offset = 0;
  const numEntries = mappingView.getUint32(offset, true); offset += 4;
  const mappingSize = mappingView.getUint32(offset, true); offset += 4;

  const mapping = new Map();
  for (let i = 0; i < numEntries; i++) {
    const id = mappingView.getUint32(offset, true); offset += 4;
    const line = mappingView.getUint32(offset, true); offset += 4;
    const col = mappingView.getUint32(offset, true); offset += 4; // New column field
    const pathLen = mappingView.getUint32(offset, true); offset += 4;

    const pathBytes = new Uint8Array(mappingBuffer, offset, pathLen);
    const path = new TextDecoder().decode(pathBytes);
    offset += pathLen;

    mapping.set(id, { path, line, col });
  }

  // 2. Read Hits
  const exports = instance.exports;
  if (!exports.COV_HITS_PTR) return null;

  const covPtr = exports.COV_HITS_PTR.value;
  const covLen = numEntries;

  const memory = exports.memory;
  if (covPtr === 0 || covLen === 0) return null;

  const hitBuffer = new Uint8Array(memory.buffer, covPtr, covLen);

  const hits = [];
  for (let i = 0; i < covLen; i++) {
    if (hitBuffer[i] > 0) {
      const info = mapping.get(i);
      if (info) {
        console.log(`Hit [${i}]: ${info.path}:${info.line}`);
        hits.push({ ...info, count: hitBuffer[i] });
      }
    }
  }

  return hits;
}