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

// Errno constants (must match src/webgl2_context.rs)
const ERR_OK = 0;
const ERR_INVALID_HANDLE = 1;
const ERR_OOM = 2;
const ERR_INVALID_ARGS = 3;
const ERR_NOT_IMPLEMENTED = 4;
const ERR_GL = 5;
const ERR_INTERNAL = 6;

/**
 * WasmWebGL2RenderingContext
 *
 * A WebGL2-like context that forwards all operations to WASM.
 * Constructor is internal only; use webGL2() factory to create.
 *
 * Public methods:
 * - destroy(): explicitly free all WASM resources
 * - createTexture(), bindTexture(), texImage2D()
 * - createFramebuffer(), bindFramebuffer(), framebufferTexture2D()
 * - readPixels()
 * - (more methods as needed)
 *
 * All operations validate !_destroyed before proceeding.
 */
class WasmWebGL2RenderingContext {
  /**
   * Internal constructor. Use webGL2() factory instead.
   * @param {WebAssembly.Instance} instance
   * @param {u32} ctxHandle
   */
  constructor(instance, ctxHandle) {
    this._instance = instance;
    this._ctxHandle = ctxHandle;
    this._destroyed = false;
  }

  /**
   * Destroy the context and free all WASM resources.
   * After calling destroy(), all methods will throw.
   */
  destroy() {
    if (this._destroyed) return;
    const ex = this._instance.exports;
    if (ex && typeof ex.wasm_ctx_destroy_context === 'function') {
      // Note: current design has wasm_destroy_context (module level), not per-ctx
      // We'll use the module-level destroy for now
      if (typeof ex.wasm_destroy_context === 'function') {
        const code = ex.wasm_destroy_context(this._ctxHandle);
        _checkErr(code, this._instance);
      }
    }
    this._destroyed = true;
  }

  /**
   * Validate that the context is not destroyed.
   * @throws {Error} if destroyed
   */
  _assertNotDestroyed() {
    if (this._destroyed) {
      throw new Error('context has been destroyed');
    }
  }

  /**
   * Create a new texture.
   * @returns {u32} texture handle
   * @throws {Error} on failure
   */
  createTexture() {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_create_texture !== 'function') {
      throw new Error('wasm_ctx_create_texture not found');
    }
    const handle = ex.wasm_ctx_create_texture(this._ctxHandle);
    if (handle === 0) {
      const msg = _readErrorMessage(this._instance);
      throw new Error(`Failed to create texture: ${msg}`);
    }
    return handle;
  }

  /**
   * Delete a texture.
   * @param {u32} tex
   * @throws {Error} on failure
   */
  deleteTexture(tex) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_delete_texture !== 'function') {
      throw new Error('wasm_ctx_delete_texture not found');
    }
    const code = ex.wasm_ctx_delete_texture(this._ctxHandle, tex);
    _checkErr(code, this._instance);
  }

  /**
   * Bind a texture.
   * @param {u32} target (ignored; texture target is implicit)
   * @param {u32} tex (0 to unbind)
   * @throws {Error} on failure
   */
  bindTexture(target, tex) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_bind_texture !== 'function') {
      throw new Error('wasm_ctx_bind_texture not found');
    }
    const code = ex.wasm_ctx_bind_texture(this._ctxHandle, target >>> 0, tex >>> 0);
    _checkErr(code, this._instance);
  }

  /**
   * Upload pixel data to a texture.
   * @param {u32} target (ignored)
   * @param {u32} level (ignored for now)
   * @param {u32} internalFormat (ignored)
   * @param {u32} width
   * @param {u32} height
   * @param {u32} border (ignored)
   * @param {u32} format (ignored; assumes RGBA)
   * @param {u32} type_ (ignored; assumes UNSIGNED_BYTE)
   * @param {Uint8Array} pixels (RGBA u8 data, or null)
   * @throws {Error} on failure
   */
  texImage2D(target, level, internalFormat, width, height, border, format, type_, pixels) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_tex_image_2d !== 'function') {
      throw new Error('wasm_ctx_tex_image_2d not found');
    }

    // Normalize pixels to Uint8Array
    let data = pixels;
    if (!data) {
      data = new Uint8Array(width * height * 4);
    } else if (!(data instanceof Uint8Array)) {
      data = new Uint8Array(data);
    }

    // Allocate buffer in WASM and upload
    const len = data.length;
    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) {
      throw new Error('Failed to allocate memory for pixel data');
    }

    try {
      // Write pixels into WASM memory (create fresh view)
      const mem = new Uint8Array(ex.memory.buffer);
      mem.set(data, ptr);

      // Call WASM to upload
      const code = ex.wasm_ctx_tex_image_2d(
        this._ctxHandle,
        target >>> 0,
        level >>> 0,
        internalFormat >>> 0,
        width >>> 0,
        height >>> 0,
        border >>> 0,
        format >>> 0,
        type_ >>> 0,
        ptr >>> 0,
        len >>> 0
      );
      _checkErr(code, this._instance);
    } finally {
      // Always free the temporary buffer
      const freeCode = ex.wasm_free(ptr);
      // We ignore free errors for now
    }
  }

  /**
   * Create a new framebuffer.
   * @returns {u32} framebuffer handle
   * @throws {Error} on failure
   */
  createFramebuffer() {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_create_framebuffer !== 'function') {
      throw new Error('wasm_ctx_create_framebuffer not found');
    }
    const handle = ex.wasm_ctx_create_framebuffer(this._ctxHandle);
    if (handle === 0) {
      const msg = _readErrorMessage(this._instance);
      throw new Error(`Failed to create framebuffer: ${msg}`);
    }
    return handle;
  }

  /**
   * Delete a framebuffer.
   * @param {u32} fb
   * @throws {Error} on failure
   */
  deleteFramebuffer(fb) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_delete_framebuffer !== 'function') {
      throw new Error('wasm_ctx_delete_framebuffer not found');
    }
    const code = ex.wasm_ctx_delete_framebuffer(this._ctxHandle, fb);
    _checkErr(code, this._instance);
  }

  /**
   * Bind a framebuffer.
   * @param {u32} target (ignored)
   * @param {u32} fb (0 to unbind)
   * @throws {Error} on failure
   */
  bindFramebuffer(target, fb) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_bind_framebuffer !== 'function') {
      throw new Error('wasm_ctx_bind_framebuffer not found');
    }
    const code = ex.wasm_ctx_bind_framebuffer(this._ctxHandle, target >>> 0, fb >>> 0);
    _checkErr(code, this._instance);
  }

  /**
   * Attach a texture to the bound framebuffer.
   * @param {u32} target (ignored)
   * @param {u32} attachment (ignored; assumes COLOR_ATTACHMENT0)
   * @param {u32} textarget (ignored)
   * @param {u32} texture
   * @param {i32} level (ignored)
   * @throws {Error} on failure
   */
  framebufferTexture2D(target, attachment, textarget, texture, level) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_framebuffer_texture2d !== 'function') {
      throw new Error('wasm_ctx_framebuffer_texture2d not found');
    }
    const code = ex.wasm_ctx_framebuffer_texture2d(
      this._ctxHandle,
      target >>> 0,
      attachment >>> 0,
      textarget >>> 0,
      texture >>> 0,
      level >>> 0
    );
    _checkErr(code, this._instance);
  }

  /**
   * Read pixels from the bound framebuffer into out buffer.
   * @param {i32} x
   * @param {i32} y
   * @param {u32} width
   * @param {u32} height
   * @param {u32} format (ignored; assumes RGBA)
   * @param {u32} type_ (ignored; assumes UNSIGNED_BYTE)
   * @param {Uint8Array} out (output buffer, must be w*h*4 bytes)
   * @throws {Error} on failure
   */
  readPixels(x, y, width, height, format, type_, out) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_read_pixels !== 'function') {
      throw new Error('wasm_ctx_read_pixels not found');
    }

    const len = width * height * 4;
    if (!out || out.length < len) {
      throw new Error(`output buffer too small (need ${len}, have ${out ? out.length : 0})`);
    }

    // Allocate buffer in WASM for readback
    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) {
      throw new Error('Failed to allocate memory for readPixels output');
    }

    try {
      // Call WASM to read pixels into the temporary buffer
      const code = ex.wasm_ctx_read_pixels(
        this._ctxHandle,
        x >>> 0,
        y >>> 0,
        width >>> 0,
        height >>> 0,
        format >>> 0,
        type_ >>> 0,
        ptr >>> 0,
        len >>> 0
      );
      _checkErr(code, this._instance);

      // Copy from WASM memory to output buffer (create fresh view)
      const mem = new Uint8Array(ex.memory.buffer);
      const src = mem.subarray(ptr, ptr + len);
      out.set(src);
    } finally {
      // Always free the temporary buffer
      ex.wasm_free(ptr);
    }
  }
}

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

  // Create a context in WASM
  const ctxHandle = ex.wasm_create_context();
  if (ctxHandle === 0) {
    const msg = _readErrorMessage(instance);
    throw new Error(`Failed to create context: ${msg}`);
  }

  // Wrap and return
  const gl = new WasmWebGL2RenderingContext(instance, ctxHandle);
  return gl;
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
