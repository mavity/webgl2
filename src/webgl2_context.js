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
 * @implements {WebGL2RenderingContext}
 */
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

  // --- Stubs for unimplemented WebGL2 methods (forwarding API surface) ---
  // These are intentionally not implemented in the prototype. They allow
  // callers to detect missing functionality early with a uniform error.

  createShader(type) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  shaderSource(shader, source) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  compileShader(shader) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  deleteShader(shader) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  createProgram() { this._assertNotDestroyed(); throw new Error('not implemented'); }
  attachShader(program, shader) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  detachShader(program, shader) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  linkProgram(program) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  deleteProgram(program) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  useProgram(program) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  getShaderParameter(shader, pname) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  getProgramParameter(program, pname) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  getShaderInfoLog(shader) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  getProgramInfoLog(program) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  getAttribLocation(program, name) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  bindAttribLocation(program, index, name) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  enableVertexAttribArray(index) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  disableVertexAttribArray(index) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  vertexAttribPointer(index, size, type, normalized, stride, offset) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  vertexAttribDivisor(index, divisor) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  createBuffer() { this._assertNotDestroyed(); throw new Error('not implemented'); }
  bindBuffer(target, buffer) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  deleteBuffer(buffer) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  bufferData(target, data, usage) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  bufferSubData(target, offset, data) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  copyBufferSubData(readTarget, writeTarget, readOffset, writeOffset, size) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  getBufferParameter(target, pname) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  isBuffer(buffer) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  drawArrays(mode, first, count) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  drawElements(mode, count, type, offset) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  drawArraysInstanced(mode, first, count, instanceCount) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  drawElementsInstanced(mode, count, type, offset, instanceCount) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  drawRangeElements(mode, start, end, count, type, offset) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  drawBuffers(buffers) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  createVertexArray() { this._assertNotDestroyed(); throw new Error('not implemented'); }
  bindVertexArray(vao) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  deleteVertexArray(vao) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  isVertexArray(vao) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  createTransformFeedback() { this._assertNotDestroyed(); throw new Error('not implemented'); }
  bindTransformFeedback(target, tf) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  beginTransformFeedback(primitiveMode) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  pauseTransformFeedback() { this._assertNotDestroyed(); throw new Error('not implemented'); }
  resumeTransformFeedback() { this._assertNotDestroyed(); throw new Error('not implemented'); }
  endTransformFeedback() { this._assertNotDestroyed(); throw new Error('not implemented'); }
  transformFeedbackVaryings(program, varyings, bufferMode) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  getTransformFeedbackVarying(program, index) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  createQuery() { this._assertNotDestroyed(); throw new Error('not implemented'); }
  deleteQuery(q) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  beginQuery(target, id) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  endQuery(target) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  getQueryParameter(query, pname) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  fenceSync(condition, flags) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  clientWaitSync(sync, flags, timeout) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  waitSync(sync, flags, timeout) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  deleteSync(sync) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  getSyncParameter(sync, pname) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  createSampler() { this._assertNotDestroyed(); throw new Error('not implemented'); }
  deleteSampler(s) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  bindSampler(unit, sampler) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  samplerParameteri(sampler, pname, param) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  samplerParameterf(sampler, pname, param) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  activeTexture(texture) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  texParameteri(target, pname, param) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  generateMipmap(target) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  copyTexImage2D(target, level, internalformat, x, y, width, height, border) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  copyTexSubImage2D(target, level, xoffset, yoffset, x, y, width, height) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  texImage3D(target, level, internalformat, width, height, depth, border, format, type, pixels) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  createRenderbuffer() { this._assertNotDestroyed(); throw new Error('not implemented'); }
  bindRenderbuffer(target, rb) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  deleteRenderbuffer(rb) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  renderbufferStorage(target, internalformat, width, height) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  framebufferRenderbuffer(target, attachment, renderbuffertarget, renderbuffer) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  checkFramebufferStatus(target) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  blitFramebuffer(srcX0, srcY0, srcX1, srcY1, dstX0, dstY0, dstX1, dstY1, mask, filter) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  readBuffer(src) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  pixelStorei(pname, param) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  getExtension(name) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  getSupportedExtensions() { this._assertNotDestroyed(); throw new Error('not implemented'); }

  getUniformLocation(program, name) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  uniform1f(loc, x) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  uniform2f(loc, x, y) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  uniform3f(loc, x, y, z) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  uniform4f(loc, x, y, z, w) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  uniform1i(loc, x) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  uniformMatrix4fv(loc, transpose, value) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  getActiveUniform(program, index) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  getActiveAttrib(program, index) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  getParameter(pname) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  getError() { this._assertNotDestroyed(); throw new Error('not implemented'); }
  finish() { this._assertNotDestroyed(); throw new Error('not implemented'); }
  flush() { this._assertNotDestroyed(); throw new Error('not implemented'); }

  isTexture(tex) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  isFramebuffer(fb) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  isProgram(p) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  isShader(s) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  isEnabled(cap) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  viewport(x, y, width, height) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  scissor(x, y, width, height) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  clear(mask) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  clearColor(r, g, b, a) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  clearDepth(depth) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  depthFunc(func) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  depthMask(flag) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  colorMask(r, g, b, a) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  polygonOffset(factor, units) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  sampleCoverage(value, invert) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  stencilFunc(func, ref, mask) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  stencilOp(fail, zfail, zpass) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  stencilMask(mask) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  blendFunc(sfactor, dfactor) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  blendFuncSeparate(sfactorRGB, dfactorRGB, sfactorAlpha, dfactorAlpha) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  blendEquation(mode) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  blendEquationSeparate(modeRGB, modeAlpha) { this._assertNotDestroyed(); throw new Error('not implemented'); }
}

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
