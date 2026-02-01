// @ts-check

import {
  WasmWebGL2RenderingContext,
  ERR_OK,
  ERR_INVALID_HANDLE,
  readErrorMessage,
  getShaderModule,
  getShaderWat,
  getShaderGlsl,
  decompileWasmToGlsl
} from './src/webgl2_context.js';
import { GPU, GPUBufferUsage, GPUMapMode, GPUTextureUsage, GPUShaderStage } from './src/webgpu_context.js';

export const debug = {
  getLcovReport,
  resetLcovReport
};

export { ERR_OK, ERR_INVALID_HANDLE, GPUBufferUsage, GPUMapMode, GPUTextureUsage, GPUShaderStage, getShaderModule, getShaderWat, getShaderGlsl, decompileWasmToGlsl };

/**
 * Simple allocator for function table indices.
 * Tracks which slots are in use to enable reuse.
 */
class TableAllocator {
  constructor() {
    // Rust uses many slots for its indirect function table (dyn calls, etc).
    // We must avoid collision by starting allocations after that region.
    // Increased from 2000 to 5000 for safety in larger modules.
    this.nextIndex = 5000;
    this.freeList = [];
  }

  allocate() {
    if (this.freeList.length > 0) {
      return this.freeList.pop();
    }
    return this.nextIndex++;
  }

  free(index) {
    this.freeList.push(index);
  }
}

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
 * 3. Creates a Rust-owned context via wasm_create_context_with_flags(flags)
 * 4. Returns a WasmWebGL2RenderingContext JS wrapper
 *
 * @param {{
 *  debug?: boolean | 'shaders' | 'rust' | 'all',
 *  size?: { width: number, height: number },
 * }} [opts] - options
 * @returns {Promise<WasmWebGL2RenderingContext>}
 * @throws {Error} if WASM loading or instantiation fails
 */
export async function webGL2({ debug = (typeof process !== 'undefined' ? process?.env || {} : typeof window !== 'undefined' ? window : globalThis).WEBGL2_DEBUG === 'true', size } = {}) {
  // Determine if we need the debug WASM binary (Rust symbols)
  const useDebugWasm = debug === true || debug === 'rust' || debug === 'all';

  // Load WASM binary
  let promise = wasmCache.get(useDebugWasm);
  if (!promise) {
    promise = initWASM({ debug: useDebugWasm });
    wasmCache.set(useDebugWasm, promise);
    // ensure success is cached but not failure
    promise.catch(() => {
      if (wasmCache.get(useDebugWasm) === promise) {
        wasmCache.delete(useDebugWasm);
      }
    });
  }
  const { ex, instance, sharedTable, tableAllocator, turboGlobals } = await promise;

  // Initialize coverage if available
  if (ex.wasm_init_coverage && ex.COV_MAP_PTR) {
    const mapPtr = ex.COV_MAP_PTR.value;
    // Read num_entries from the start of the map data
    // mapPtr is aligned to 16 bytes, so we can use Uint32Array
    const mem = new Uint32Array(ex.memory.buffer);
    const numEntries = mem[mapPtr >>> 2];
    ex.wasm_init_coverage(numEntries);
  }

  // Determine debug flags for creation
  const debugShaders = debug === true || debug === 'shaders' || debug === 'all';
  const debugRust = debug === true || debug === 'rust' || debug === 'all';
  const flags = (debugShaders ? 1 : 0); // only shader debug encoded in flags

  // Default size to 640x480 if not provided
  const width = size?.width ?? 640;
  const height = size?.height ?? 480;

  // Create a context in WASM using the flags-aware API (mandatory)
  const ctxHandle = ex.wasm_create_context_with_flags(flags, width, height);

  if (ctxHandle === 0) {
    const msg = readErrorMessage(instance);
    throw new Error(`Failed to create context: ${msg}`);
  }

  // Wrap and return, pass debug booleans to the JS wrapper
  const gl = new WasmWebGL2RenderingContext({
    instance,
    ctxHandle,
    width,
    height,
    debugShaders: !!debugShaders,
    sharedTable,
    tableAllocator,
    turboGlobals
  });

  if (size && typeof size.width === 'number' && typeof size.height === 'number') {
    gl.resize(size.width, size.height);
    gl.viewport(0, 0, size.width, size.height);
  }

  return gl;
}

/**
 * Factory function: create a new WebGPU instance.
 *
 * @param {{
 *  debug?: boolean | 'shaders' | 'rust' | 'all',
 * }} [opts] - options
 * @returns {Promise<GPU>}
 */
export async function webGPU({ debug = (typeof process !== 'undefined' ? process?.env || {} : typeof window !== 'undefined' ? window : globalThis).WEBGL2_DEBUG === 'true' } = {}) {
  const useDebugWasm = debug === true || debug === 'rust' || debug === 'all';
  let promise = wasmCache.get(useDebugWasm);
  if (!promise) {
    promise = initWASM({ debug: useDebugWasm });
    wasmCache.set(useDebugWasm, promise);
    promise.catch(() => {
      if (wasmCache.get(useDebugWasm) === promise) {
        wasmCache.delete(useDebugWasm);
      }
    });
  }

  // Resolve the WASM initialization and return a WebGPU wrapper (GPU).
  // Tests expect an object with `requestAdapter()`; return a `GPU` instance
  // backed by the WASM exports and memory.
  const { ex } = await promise;
  return new GPU(ex, ex.memory);
}

/**
 * @type {Map<boolean, ReturnType<typeof initWASM>>}
 */
const wasmCache = new Map();

/**
 * @param {{ debug?: boolean }} [options]
 */
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

  /**
   * Instantiate WASM (no imports needed, memory is exported)
   * @type {WebAssembly.Instance}
   */
  let instance;

  // Create shared function table for direct shader calls
  const sharedTable = new WebAssembly.Table({
    initial: 8192,
    maximum: 65536,
    element: "anyfunc"
  });
  const tableAllocator = new TableAllocator();

  // Create shared mutable globals that will be used by both the main module and
  // transient shader modules. These are WebAssembly.Global objects with mutable
  // i32 value so shaders can update pointer state directly.
  const turboGlobals = {
    ACTIVE_ATTR_PTR: new WebAssembly.Global({ value: 'i32', mutable: true }, 0),
    ACTIVE_UNIFORM_PTR: new WebAssembly.Global({ value: 'i32', mutable: true }, 0),
    ACTIVE_VARYING_PTR: new WebAssembly.Global({ value: 'i32', mutable: true }, 0),
    ACTIVE_PRIVATE_PTR: new WebAssembly.Global({ value: 'i32', mutable: true }, 0),
    ACTIVE_TEXTURE_PTR: new WebAssembly.Global({ value: 'i32', mutable: true }, 0),
    ACTIVE_FRAME_SP: new WebAssembly.Global({ value: 'i32', mutable: true }, 0),
  };

  const importObject = {
    env: {
      __indirect_function_table: sharedTable,  // Exact name LLVM expects
      memory: new WebAssembly.Memory({ initial: 100 }),
      ACTIVE_ATTR_PTR: turboGlobals.ACTIVE_ATTR_PTR,
      ACTIVE_UNIFORM_PTR: turboGlobals.ACTIVE_UNIFORM_PTR,
      ACTIVE_VARYING_PTR: turboGlobals.ACTIVE_VARYING_PTR,
      ACTIVE_PRIVATE_PTR: turboGlobals.ACTIVE_PRIVATE_PTR,
      ACTIVE_TEXTURE_PTR: turboGlobals.ACTIVE_TEXTURE_PTR,
      ACTIVE_FRAME_SP: turboGlobals.ACTIVE_FRAME_SP,
      print: (ptr, len) => {
        const mem = new Uint8Array(instance.exports.memory.buffer);
        const bytes = mem.subarray(ptr, ptr + len);
        console.log(new TextDecoder('utf-8').decode(bytes));
      },
      wasm_register_shader: (ptr, len) => {
        const mem = new Uint8Array(instance.exports.memory.buffer);
        const bytes = mem.slice(ptr, ptr + len);
        const shaderModule = new WebAssembly.Module(bytes);
        const index = tableAllocator.allocate();
        
        const env = {
          memory: instance.exports.memory,
          __indirect_function_table: sharedTable,
          ACTIVE_ATTR_PTR: turboGlobals.ACTIVE_ATTR_PTR,
          ACTIVE_UNIFORM_PTR: turboGlobals.ACTIVE_UNIFORM_PTR,
          ACTIVE_VARYING_PTR: turboGlobals.ACTIVE_VARYING_PTR,
          ACTIVE_PRIVATE_PTR: turboGlobals.ACTIVE_PRIVATE_PTR,
          ACTIVE_TEXTURE_PTR: turboGlobals.ACTIVE_TEXTURE_PTR,
          ACTIVE_FRAME_SP: turboGlobals.ACTIVE_FRAME_SP,
        };
        
        // Copy math functions
        const mathFuncs = [
          'gl_cos', 'gl_sin', 'gl_tan', 'gl_acos', 'gl_asin', 'gl_atan', 'gl_atan2',
          'gl_exp', 'gl_exp2', 'gl_log', 'gl_log2', 'gl_pow', 'gl_floor', 'gl_ceil',
          'gl_fract', 'gl_mod', 'gl_min', 'gl_max', 'gl_abs', 'gl_sign', 'gl_sqrt',
          'gl_inversesqrt', 'gl_sinh', 'gl_cosh', 'gl_tanh', 'gl_asinh', 'gl_acosh', 'gl_atanh'
        ];
        for (const name of mathFuncs) {
          if (instance.exports[name]) {
            env[name] = instance.exports[name];
          }
        }

        const shaderInstance = new WebAssembly.Instance(shaderModule, { env });
        if (shaderInstance.exports.main) {
          sharedTable.set(index, shaderInstance.exports.main);
        }
        return index;
      },
      wasm_release_shader_index: (idx) => {
        tableAllocator.free(idx);
      },
      wasm_sync_turbo_globals: (attr, uniform, varying, private_, texture, frame_sp) => {
        try {
          turboGlobals.ACTIVE_ATTR_PTR.value = attr >>> 0;
          turboGlobals.ACTIVE_UNIFORM_PTR.value = uniform >>> 0;
          turboGlobals.ACTIVE_VARYING_PTR.value = varying >>> 0;
          turboGlobals.ACTIVE_PRIVATE_PTR.value = private_ >>> 0;
          turboGlobals.ACTIVE_TEXTURE_PTR.value = texture >>> 0;
          turboGlobals.ACTIVE_FRAME_SP.value = frame_sp >>> 0;
        } catch (e) {
          // Defensive: if the globals are immutable or not set, at least avoid crashing
          console.warn('wasm_sync_turbo_globals failed to set globals', e);
        }
      },
      dispatch_uncaptured_error: (ptr, len) => {
        const mem = new Uint8Array(instance.exports.memory.buffer);
        const bytes = mem.subarray(ptr, ptr + len);
        const msg = new TextDecoder('utf-8').decode(bytes);
        if (typeof GPU !== 'undefined' && typeof GPU.dispatchUncapturedError === 'function') {
          GPU.dispatchUncapturedError(msg);
        } else {
          console.error("GPU.dispatchUncapturedError not available", msg);
        }
      },
      // Required by egg crate for timing measurements
      now: () => {
        return performance.now();
      }
    },
    math: {
      sin: Math.sin,
      cos: Math.cos,
      tan: Math.tan,
      asin: Math.asin,
      acos: Math.acos,
      atan: Math.atan,
      atan2: Math.atan2,
      exp: Math.exp,
      exp2: (x) => Math.pow(2, x),
      log: Math.log,
      log2: Math.log2,
      pow: Math.pow
    }
  };
  instance = await WebAssembly.instantiate(wasmModule, importObject);

  // Verify required exports
  const ex = instance.exports;
  if (typeof ex.wasm_create_context_with_flags !== 'function') {
    throw new Error('WASM module missing wasm_create_context_with_flags export');
  }
  if (!(ex.memory instanceof WebAssembly.Memory)) {
    throw new Error('WASM module missing memory export');
  }
  return { ex, instance, module: wasmModule, sharedTable, tableAllocator, turboGlobals };
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
