// Thin forwarding WasmWebGL2RenderingContext and helpers
// This module contains the class and small helpers that operate on the
// WebAssembly instance. It is intentionally minimal: JS forwards calls to
// WASM and reads last-error strings when needed.

/** @typedef {number} u32 */

// Errno constants (must match src/webgl2_context.rs)
export const ERR_OK = 0;
export const ERR_INVALID_HANDLE = 1;
export const ERR_OOM = 2;
export const ERR_INVALID_ARGS = 3;
export const ERR_NOT_IMPLEMENTED = 4;
export const ERR_GL = 5;
export const ERR_INTERNAL = 6;

/**
 * Read an error message from WASM memory and return it as string.
 * Exported so callers outside this module can report errors.
 * @param {WebAssembly.Instance} instance
 * @returns {string}
 */
export function readErrorMessage(instance) {
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

function _checkErr(code, instance) {
  if (code === ERR_OK) return;
  const msg = readErrorMessage(instance);
  throw new Error(`WASM error ${code}: ${msg}`);
}

export class WasmWebGL2RenderingContext {
  /**
   * @param {WebAssembly.Instance} instance
   * @param {u32} ctxHandle
   */
  constructor(instance, ctxHandle) {
    this._instance = instance;
    this._ctxHandle = ctxHandle;
    this._destroyed = false;
  }

  destroy() {
    if (this._destroyed) return;
    const ex = this._instance.exports;
    if (ex && typeof ex.wasm_ctx_destroy_context === 'function') {
      if (typeof ex.wasm_destroy_context === 'function') {
        const code = ex.wasm_destroy_context(this._ctxHandle);
        _checkErr(code, this._instance);
      }
    }
    this._destroyed = true;
  }

  _assertNotDestroyed() {
    if (this._destroyed) throw new Error('context has been destroyed');
  }

  createTexture() {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_create_texture !== 'function') {
      throw new Error('wasm_ctx_create_texture not found');
    }
    const handle = ex.wasm_ctx_create_texture(this._ctxHandle);
    if (handle === 0) {
      const msg = readErrorMessage(this._instance);
      throw new Error(`Failed to create texture: ${msg}`);
    }
    return handle;
  }

  deleteTexture(tex) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_delete_texture !== 'function') {
      throw new Error('wasm_ctx_delete_texture not found');
    }
    const code = ex.wasm_ctx_delete_texture(this._ctxHandle, tex);
    _checkErr(code, this._instance);
  }

  bindTexture(target, tex) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_bind_texture !== 'function') {
      throw new Error('wasm_ctx_bind_texture not found');
    }
    const code = ex.wasm_ctx_bind_texture(this._ctxHandle, target >>> 0, tex >>> 0);
    _checkErr(code, this._instance);
  }

  texImage2D(target, level, internalFormat, width, height, border, format, type_, pixels) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_tex_image_2d !== 'function') {
      throw new Error('wasm_ctx_tex_image_2d not found');
    }

    let data = pixels;
    if (!data) data = new Uint8Array(width * height * 4);
    else if (!(data instanceof Uint8Array)) data = new Uint8Array(data);

    const len = data.length;
    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) throw new Error('Failed to allocate memory for pixel data');

    try {
      const mem = new Uint8Array(ex.memory.buffer);
      mem.set(data, ptr);

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
      ex.wasm_free(ptr);
    }
  }

  createFramebuffer() {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_create_framebuffer !== 'function') {
      throw new Error('wasm_ctx_create_framebuffer not found');
    }
    const handle = ex.wasm_ctx_create_framebuffer(this._ctxHandle);
    if (handle === 0) {
      const msg = readErrorMessage(this._instance);
      throw new Error(`Failed to create framebuffer: ${msg}`);
    }
    return handle;
  }

  deleteFramebuffer(fb) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_delete_framebuffer !== 'function') {
      throw new Error('wasm_ctx_delete_framebuffer not found');
    }
    const code = ex.wasm_ctx_delete_framebuffer(this._ctxHandle, fb);
    _checkErr(code, this._instance);
  }

  bindFramebuffer(target, fb) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_bind_framebuffer !== 'function') {
      throw new Error('wasm_ctx_bind_framebuffer not found');
    }
    const code = ex.wasm_ctx_bind_framebuffer(this._ctxHandle, target >>> 0, fb >>> 0);
    _checkErr(code, this._instance);
  }

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

    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) throw new Error('Failed to allocate memory for readPixels output');

    try {
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

      const mem = new Uint8Array(ex.memory.buffer);
      const src = mem.subarray(ptr, ptr + len);
      out.set(src);
    } finally {
      ex.wasm_free(ptr);
    }
  }
}
