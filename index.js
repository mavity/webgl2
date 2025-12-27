// @ts-check

import {
  WasmWebGL2RenderingContext,
  ERR_OK,
  ERR_INVALID_HANDLE,
  readErrorMessage
} from './src/webgl2_context.js';
import { GPU, GPUBufferUsage, GPUMapMode, GPUTextureUsage } from './src/webgpu_context.js';

export const debug = {
  getLcovReport,
  resetLcovReport
};

export { ERR_OK, ERR_INVALID_HANDLE, GPUBufferUsage, GPUMapMode, GPUTextureUsage };

/**
 * WebGL2 Prototype: Rust-owned Context, JS thin-forwarder
 * Implements docs/1.1.1-webgl2-prototype.md
 *
 * This module provides:
 * - WasmWebGL2RenderingContext: JS class that forwards all calls to WASM
 * - webGL2(): factory function to create a new context
 *
 * WASM owns all runtime state (textures, framebuffers, contexts).
 * JS is a thin forwarder with no emulation of WebGL behavior.
 *
 * Explicit lifecycle: caller must call destroy() to free resources.
 * All operations return errno (0 = OK). Non-zero errno causes JS to throw
 * with the message from wasm_last_error_ptr/len.
 */


const isNode =
  typeof process !== 'undefined' &&
  process.versions != null &&
  process.versions.node != null;

/** @typedef {number} u32 */

/**
 * Factory function: create a new WebGL2 context.
 *
 * This function:
 * 1. Auto-loads webgl2.wasm (expects it next to index2.js)
 * 2. Instantiates the WASM module with memory
 * 3. Creates a Rust-owned context via wasm_create_context()
 * 4. Returns a WasmWebGL2RenderingContext JS wrapper
 *
 * @param {{
 *  debug?: boolean,
 *  size?: { width: number, height: number },
 * }} [opts] - options
 * @returns {Promise<WasmWebGL2RenderingContext>}
 * @throws {Error} if WASM loading or instantiation fails
 */
export async function webGL2({ debug = process.env.WEBGL2_DEBUG === 'true', size } = {}) {
  // Load WASM binary
  let promise = wasmCache.get(!!debug);
  if (!promise) {
    promise = initWASM({ debug });
    wasmCache.set(!!debug, promise);
    // ensure success is cached but not failure
    promise.catch(() => {
      if (wasmCache.get(!!debug) === promise) {
        wasmCache.delete(!!debug);
      }
    });
  }
  const { ex, instance } = await promise;

  // Initialize coverage if available
  if (ex.wasm_init_coverage && ex.COV_MAP_PTR) {
    const mapPtr = ex.COV_MAP_PTR.value;
    // Read num_entries from the start of the map data
    // mapPtr is aligned to 16 bytes, so we can use Uint32Array
    const mem = new Uint32Array(ex.memory.buffer);
    const numEntries = mem[mapPtr >>> 2];
    ex.wasm_init_coverage(numEntries);
  }

  // Create a context in WASM
  const ctxHandle = ex.wasm_create_context();
  if (ctxHandle === 0) {
    const msg = readErrorMessage(instance);
    throw new Error(`Failed to create context: ${msg}`);
  }

  // Wrap and return
  const gl = new WasmWebGL2RenderingContext(instance, ctxHandle);

  if (size && typeof size.width === 'number' && typeof size.height === 'number') {
    gl.resize(size.width, size.height);
  }

  return gl;
}

/**
 * Factory function: create a new WebGPU instance.
 *
 * @param {{
 *  debug?: boolean,
 * }} [opts] - options
 * @returns {Promise<GPU>}
 */
export async function webGPU({ debug = process.env.WEBGL2_DEBUG === 'true' } = {}) {
  let promise = wasmCache.get(!!debug);
  if (!promise) {
    promise = initWASM({ debug });
    wasmCache.set(!!debug, promise);
    promise.catch(() => {
      if (wasmCache.get(!!debug) === promise) {
        wasmCache.delete(!!debug);
      }
    });
  }
  const { ex, instance } = await promise;
  return new GPU(ex, ex.memory);
}

/**
 * @type {Map<boolean, Promise<{ ex: WebAssembly.Exports, instance: WebAssembly.Instance, module: WebAssembly.Module }>>}
 */
const wasmCache = new Map();

async function initWASM({ debug } = {}) {
  const wasmFile = debug ? 'webgl2.debug.wasm' : 'webgl2.wasm';
  let wasmBuffer;
  if (isNode) {
    // Use dynamic imports so this module can be loaded in the browser too.
    const path = await import('path');
    const fs = await import('fs');
    const { fileURLToPath } = await import('url');
    const wasmPath = path.join(path.dirname(fileURLToPath(import.meta.url)), wasmFile);
    if (!fs.existsSync(wasmPath)) {
      throw new Error(`WASM not found at ${wasmPath}. Run: npm run build:wasm`);
    }
    // readFileSync is available on the imported namespace
    wasmBuffer = fs.readFileSync(wasmPath);
  } else {
    // Browser: fetch the wasm relative to this module
    const resp = await fetch(new URL('./' + wasmFile, import.meta.url));
    if (!resp.ok) {
      throw new Error(`Failed to fetch ${wasmFile}: ${resp.status}`);
    }
    wasmBuffer = await resp.arrayBuffer();
  }

  // Compile WASM module
  const wasmModule = await WebAssembly.compile(wasmBuffer);

  // Instantiate WASM (no imports needed, memory is exported)
  let instance;
  const importObject = {
    env: {
      print: (ptr, len) => {
        const mem = new Uint8Array(instance.exports.memory.buffer);
        const bytes = mem.subarray(ptr, ptr + len);
        console.log(new TextDecoder('utf-8').decode(bytes));
      },
      wasm_execute_shader: (ctx, type, attrPtr, uniformPtr, varyingPtr, privatePtr, texturePtr) => {
        const gl = WasmWebGL2RenderingContext._contexts.get(ctx);
        if (gl) {
          gl._executeShader(type, attrPtr, uniformPtr, varyingPtr, privatePtr, texturePtr);
        }
      }
    }
  };
  instance = await WebAssembly.instantiate(wasmModule, importObject);

  // Verify required exports
  const ex = instance.exports;
  if (typeof ex.wasm_create_context !== 'function') {
    throw new Error('WASM module missing wasm_create_context export');
  }
  if (!(ex.memory instanceof WebAssembly.Memory)) {
    throw new Error('WASM module missing memory export');
  }
  return { ex, instance, module: wasmModule };
}

/**
 * Reads an error message from WASM memory and returns it.
 * @param {WebAssembly.Instance} instance
 * @returns {string}
 */
function _readErrorMessage(instance) {
  const ex = instance.exports;
  if (!ex || typeof ex.wasm_last_error_ptr !== 'function' || typeof ex.wasm_last_error_len !== 'function') {
    return '(no error message available)';
  }
  const ptr = ex.wasm_last_error_ptr();
  const len = ex.wasm_last_error_len();
  if (ptr === 0 || len === 0) {
    return '';
  }
  const mem = new Uint8Array(ex.memory.buffer);
  const bytes = mem.subarray(ptr, ptr + len);
  return new TextDecoder('utf-8').decode(bytes);
}

/**
 * Get LCOV coverage report from a context or device.
 * @param {any} glOrGpu
 * @returns {string}
 */
function getLcovReport(glOrGpu) {
  if (!glOrGpu) return '';

  let ex;
  if (glOrGpu._instance && glOrGpu._instance.exports) {
    ex = glOrGpu._instance.exports;
  } else if (glOrGpu.wasm) {
    ex = glOrGpu.wasm;
  } else if (glOrGpu._instance) {
    ex = glOrGpu._instance;
  }

  if (ex && typeof ex.wasm_get_lcov_report_ptr === 'function' && typeof ex.wasm_get_lcov_report_len === 'function') {
    const ptr = ex.wasm_get_lcov_report_ptr();
    const len = ex.wasm_get_lcov_report_len();
    if (ptr === 0 || len === 0) return '';
    const mem = new Uint8Array(ex.memory.buffer);
    const bytes = mem.subarray(ptr, ptr + len);
    return new TextDecoder('utf-8').decode(bytes);
  }
  return '';
}

/**
 * Reset LCOV coverage counters.
 * @param {any} glOrGpu
 */
export function resetLcovReport(glOrGpu) {
  if (!glOrGpu) return;

  let ex;
  if (glOrGpu._instance && glOrGpu._instance.exports) {
    ex = glOrGpu._instance.exports;
  } else if (glOrGpu.wasm) {
    ex = glOrGpu.wasm;
  } else if (glOrGpu._instance) {
    ex = glOrGpu._instance;
  }

  if (ex && typeof ex.wasm_reset_coverage === 'function') {
    ex.wasm_reset_coverage();
  }
}

/**
 * Checks a WASM return code (errno).
 * If non-zero, reads the error message and throws.
 * @param {number} code
 * @param {WebAssembly.Instance} instance
 * @throws {Error} if code !== 0
 */
function _checkErr(code, instance) {
  if (code === ERR_OK) return;
  const msg = _readErrorMessage(instance);
  throw new Error(`WASM error ${code}: ${msg}`);
}

if (typeof window !== 'undefined' && window) {
  // also populate globals when running in a browser environment
  try {
    window.webGL2 = webGL2;
    window.webGPU = webGPU;
    window.getLcovReport = getLcovReport;
    window.resetLcovReport = resetLcovReport;
    window.WasmWebGL2RenderingContext = WasmWebGL2RenderingContext;
  } catch (e) {
    // ignore if window is not writable
  }
}

async function nodeDemo() {
  console.log('Running index2.js demo...');
  const gl = await webGL2();
  console.log(`✓ Context created (handle will be managed by destroy())`);

  // 1x1 texture with CornflowerBlue (100, 149, 237, 255)
  const tex = gl.createTexture();
  console.log(`✓ Texture created (handle: ${tex})`);

  gl.bindTexture(0, tex);
  const pixel = new Uint8Array([100, 149, 237, 255]);
  gl.texImage2D(0, 0, 0, 1, 1, 0, 0, 0, pixel);
  console.log(`✓ Texture uploaded`);

  const fb = gl.createFramebuffer();
  console.log(`✓ Framebuffer created (handle: ${fb})`);

  gl.bindFramebuffer(0, fb);
  gl.framebufferTexture2D(0, 0, 0, tex, 0);
  console.log(`✓ Texture attached to framebuffer`);

  const out = new Uint8Array(4);
  gl.readPixels(0, 0, 1, 1, 0, 0, out);
  console.log(
    `✓ Pixel read: r=${out[0]}, g=${out[1]}, b=${out[2]}, a=${out[3]}`
  );

  if (out[0] === 100 && out[1] === 149 && out[2] === 237 && out[3] === 255) {
    console.log('✓ Pixel matches expected CornflowerBlue!');
  } else {
    console.error('✗ Pixel mismatch!');
    process.exit(1);
  }

  gl.destroy();
  console.log('✓ Context destroyed');
  console.log('\n✓ Demo passed!');
  process.exit(0);
}

// CLI demo: run when executed directly in Node
if (isNode) {
  (async () => {
    const { fileURLToPath } = await import('url');
    const path = (await import('path'));
    if (fileURLToPath(import.meta.url) === process.argv[1]) {
      nodeDemo();
    }
  })();
}
