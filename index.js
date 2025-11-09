// @ts-check

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
import { WasmWebGL2RenderingContext, ERR_OK, ERR_INVALID_HANDLE, readErrorMessage } from './src/webgl2_context.js';

// WasmWebGL2RenderingContext and related helpers were moved to
// `src/wasm_webgl2_context.js` and are imported above.

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
 * @param {Object} opts - options (unused for now)
 * @returns {Promise<WasmWebGL2RenderingContext>}
 * @throws {Error} if WASM loading or instantiation fails
 */
async function webGL2(opts = {}) {
  // Load WASM binary
  const { ex, instance } = await (wasmInitPromise || initWASM());

  // Create a context in WASM
  const ctxHandle = ex.wasm_create_context();
  if (ctxHandle === 0) {
    const msg = readErrorMessage(instance);
    throw new Error(`Failed to create context: ${msg}`);
  }

  // Wrap and return
  const gl = new WasmWebGL2RenderingContext(instance, ctxHandle);
  return gl;
}

/**
 * @type {(
 *  Promise<{ ex: WebAssembly.Exports, instance: WebAssembly.Instance }> |
 * { ex: WebAssembly.Exports, instance: WebAssembly.Instance } |
 *  undefined
 *  )}
 */
var wasmInitPromise;

async function initWASM() {
  try {
    let wasmBuffer;
    if (isNode) {
      // Use dynamic imports so this module can be loaded in the browser too.
      const path = await import('path');
      const fs = await import('fs');
      const { fileURLToPath } = await import('url');
      const wasmPath = path.join(path.dirname(fileURLToPath(import.meta.url)), 'webgl2.wasm');
      if (!fs.existsSync(wasmPath)) {
        throw new Error(`WASM not found at ${wasmPath}. Run: npm run build:wasm`);
      }
      // readFileSync is available on the imported namespace
      wasmBuffer = fs.readFileSync(wasmPath);
    } else {
      // Browser: fetch the wasm relative to this module
      const resp = await fetch(new URL('./webgl2.wasm', import.meta.url));
      if (!resp.ok) {
        throw new Error(`Failed to fetch webgl2.wasm: ${resp.status}`);
      }
      wasmBuffer = await resp.arrayBuffer();
    }

    // Compile WASM module
    const wasmModule = await WebAssembly.compile(wasmBuffer);

    // Create memory (WASM will import it)
    const memory = new WebAssembly.Memory({ initial: 16, maximum: 256 });

    // Instantiate WASM
    const importObj = { env: { memory } };
    const instance = await WebAssembly.instantiate(wasmModule, importObj);

    // Verify required exports
    const ex = instance.exports;
    if (typeof ex.wasm_create_context !== 'function') {
      throw new Error('WASM module missing wasm_create_context export');
    }
    return wasmInitPromise = { ex, instance };
  } finally {
    // do not cache failures
    wasmInitPromise = undefined;
  }
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


// Exports: ESM-style. Also attach globals in browser for convenience.
export { webGL2, WasmWebGL2RenderingContext, ERR_OK, ERR_INVALID_HANDLE };

if (typeof window !== 'undefined' && window) {
  // also populate globals when running in a browser environment
  try {
    window.webGL2 = webGL2;
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
