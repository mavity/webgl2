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

import { WasmWebGLTexture } from './webgl2_texture.js';
import { WasmWebGLShader, WasmWebGLProgram, WasmWebGLBuffer, WasmWebGLRenderbuffer } from './webgl2_resources.js';

/**
 * @implements {WebGL2RenderingContext}
 */
export class WasmWebGL2RenderingContext {
  // Constants
  FRAGMENT_SHADER = 0x8B30;
  VERTEX_SHADER = 0x8B31;
  TRIANGLES = 0x0004;
  TRIANGLE_STRIP = 0x0005;
  COLOR_BUFFER_BIT = 0x00004000;
  DEPTH_BUFFER_BIT = 0x00000100;
  DEPTH_TEST = 0x0B71;
  STENCIL_TEST = 0x0B90;
  SCISSOR_TEST = 0x0C11;
  STENCIL_BUFFER_BIT = 0x00000400;
  COMPILE_STATUS = 0x8B81;
  LINK_STATUS = 0x8B82;
  DELETE_STATUS = 0x8B80;
  VALIDATE_STATUS = 0x8B83;
  ARRAY_BUFFER = 0x8892;
  ELEMENT_ARRAY_BUFFER = 0x8893;
  COPY_READ_BUFFER = 0x8F36;
  COPY_WRITE_BUFFER = 0x8F37;
  PIXEL_PACK_BUFFER = 0x88EB;
  PIXEL_UNPACK_BUFFER = 0x88EC;
  UNIFORM_BUFFER = 0x8A11;
  TRANSFORM_FEEDBACK_BUFFER = 0x8C8E;
  STATIC_DRAW = 0x88E4;
  BYTE = 0x1400;
  UNSIGNED_BYTE = 0x1401;
  SHORT = 0x1402;
  UNSIGNED_SHORT = 0x1403;
  INT = 0x1404;
  UNSIGNED_INT = 0x1405;
  FLOAT = 0x1406;
  FLOAT_VEC2 = 0x8B50;
  FLOAT_VEC3 = 0x8B51;
  FLOAT_VEC4 = 0x8B52;
  INT_VEC2 = 0x8B53;
  INT_VEC3 = 0x8B54;
  INT_VEC4 = 0x8B55;
  BOOL = 0x8B56;
  BOOL_VEC2 = 0x8B57;
  BOOL_VEC3 = 0x8B58;
  BOOL_VEC4 = 0x8B59;
  FLOAT_MAT2 = 0x8B5A;
  FLOAT_MAT3 = 0x8B5B;
  FLOAT_MAT4 = 0x8B5C;
  SAMPLER_2D = 0x8B5E;
  SAMPLER_3D = 0x8B5F;
  SAMPLER_CUBE = 0x8B60;
  ACTIVE_UNIFORMS = 0x8B86;
  ACTIVE_ATTRIBUTES = 0x8B89;
  VIEWPORT = 0x0BA2;
  COLOR_CLEAR_VALUE = 0x0C22;
  COLOR_WRITEMASK = 0x0C23;
  DEPTH_WRITEMASK = 0x0B72;
  STENCIL_WRITEMASK = 0x0B98;
  STENCIL_BACK_WRITEMASK = 0x8CA5;

  DEPTH_FUNC = 0x0B74;
  STENCIL_FUNC = 0x0B92;
  STENCIL_VALUE_MASK = 0x0B93;
  STENCIL_REF = 0x0B97;
  STENCIL_BACK_FUNC = 0x8800;
  STENCIL_BACK_VALUE_MASK = 0x8CA4;
  STENCIL_BACK_REF = 0x8CA3;
  STENCIL_FAIL = 0x0B94;
  STENCIL_PASS_DEPTH_FAIL = 0x0B95;
  STENCIL_PASS_DEPTH_PASS = 0x0B96;
  STENCIL_BACK_FAIL = 0x8801;
  STENCIL_BACK_PASS_DEPTH_FAIL = 0x8802;
  STENCIL_BACK_PASS_DEPTH_PASS = 0x8803;

  BUFFER_SIZE = 0x8764;
  MAX_VERTEX_ATTRIBS = 0x8869;
  NO_ERROR = 0;
  INVALID_ENUM = 0x0500;
  INVALID_VALUE = 0x0501;
  INVALID_OPERATION = 0x0502;
  OUT_OF_MEMORY = 0x0505;

  ZERO = 0;
  ONE = 1;

  CURRENT_VERTEX_ATTRIB = 0x8626;
  VERTEX_ATTRIB_ARRAY_ENABLED = 0x8622;
  VERTEX_ATTRIB_ARRAY_SIZE = 0x8623;
  VERTEX_ATTRIB_ARRAY_STRIDE = 0x8624;
  VERTEX_ATTRIB_ARRAY_TYPE = 0x8625;
  VERTEX_ATTRIB_ARRAY_NORMALIZED = 0x886A;
  VERTEX_ATTRIB_ARRAY_POINTER = 0x8645;
  VERTEX_ATTRIB_ARRAY_BUFFER_BINDING = 0x889F;
  VERTEX_ATTRIB_ARRAY_DIVISOR = 0x88FE;
  VERTEX_ATTRIB_ARRAY_INTEGER = 0x88FD;

  RENDERBUFFER = 0x8D41;
  FRAMEBUFFER = 0x8D40;
  READ_FRAMEBUFFER = 0x8CA8;
  DRAW_FRAMEBUFFER = 0x8CA9;
  DEPTH_COMPONENT16 = 0x81A5;
  DEPTH_STENCIL = 0x84F9;
  RGBA4 = 0x8056;
  RGB565 = 0x8D62;
  RGB5_A1 = 0x8057;
  RGBA8 = 0x8058;
  STENCIL_INDEX8 = 0x8D48;
  COLOR_ATTACHMENT0 = 0x8CE0;
  DEPTH_ATTACHMENT = 0x8D00;
  STENCIL_ATTACHMENT = 0x8D20;
  DEPTH_STENCIL_ATTACHMENT = 0x821A;

  LESS = 0x0201;
  EQUAL = 0x0202;
  LEQUAL = 0x0203;
  GREATER = 0x0204;
  NOTEQUAL = 0x0205;
  GEQUAL = 0x0206;
  ALWAYS = 0x0207;
  NEVER = 0x0200;

  KEEP = 0x1E00;
  REPLACE = 0x1E01;
  INCR = 0x1E02;
  DECR = 0x1E03;
  INVERT = 0x150A;
  INCR_WRAP = 0x8507;
  DECR_WRAP = 0x8508;

  FRONT = 0x0404;
  BACK = 0x0405;
  FRONT_AND_BACK = 0x0408;

  TEXTURE_2D = 0x0DE1;
  TEXTURE_3D = 0x806F;
  TEXTURE_2D_ARRAY = 0x8C1A;
  TEXTURE_WRAP_S = 0x2802;
  TEXTURE_WRAP_T = 0x2803;
  TEXTURE_WRAP_R = 0x8072;
  TEXTURE_MAG_FILTER = 0x2800;
  TEXTURE_MIN_FILTER = 0x2801;
  RGBA = 0x1908;
  RED = 0x1903;
  RG = 0x8227;
  UNSIGNED_BYTE = 0x1401;
  FLOAT = 0x1406;
  NEAREST = 0x2600;
  LINEAR = 0x2601;
  NEAREST_MIPMAP_NEAREST = 0x2700;
  LINEAR_MIPMAP_NEAREST = 0x2701;
  NEAREST_MIPMAP_LINEAR = 0x2702;
  LINEAR_MIPMAP_LINEAR = 0x2703;
  REPEAT = 0x2901;
  CLAMP_TO_EDGE = 0x812F;
  MIRRORED_REPEAT = 0x8370;

  /**
   * @param {WebAssembly.Instance} instance
   * @param {u32} ctxHandle
   * @param {number} width
   * @param {number} height
   * @param {boolean} [debugShaders]
   * @param {any} [sharedTable]
   * @param {any} [tableAllocator]
   * @param {any} [scratchLayout]
   */
  constructor({ instance, ctxHandle, width, height, debugShaders = false, sharedTable = null, tableAllocator = null, scratchLayout = null }) {
    this._instance = instance;
    this._ctxHandle = ctxHandle;
    this._destroyed = false;
    /** @type {import('./webgl2_resources.js').WasmWebGLProgram | null} */
    this._currentProgram = null;
    // Explicit booleans for clarity
    this._debugShaders = !!debugShaders;
    this._drawingBufferWidth = width;
    this._drawingBufferHeight = height;
    this._sharedTable = sharedTable;
    this._tableAllocator = tableAllocator;
    this._scratchLayout = scratchLayout;

    WasmWebGL2RenderingContext._contexts.set(this._ctxHandle, this);
  }

  get drawingBufferWidth() {
    return this._drawingBufferWidth;
  }

  get drawingBufferHeight() {
    return this._drawingBufferHeight;
  }

  resize(width, height) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_resize !== 'function') {
      throw new Error('wasm_ctx_resize not found');
    }
    const code = ex.wasm_ctx_resize(this._ctxHandle, width, height);
    _checkErr(code, this._instance);
    this._drawingBufferWidth = width;
    this._drawingBufferHeight = height;
  }

  // Set the viewport for rendering
  viewport(x, y, width, height) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_viewport !== 'function') {
      throw new Error('wasm_ctx_viewport not found');
    }
    const code = ex.wasm_ctx_viewport(this._ctxHandle, x | 0, y | 0, width >>> 0, height >>> 0);
    _checkErr(code, this._instance);
  }

  /** @type {Map<number, WasmWebGL2RenderingContext>} */
  static _contexts = new Map();

  _executeShader(type, tableIdx, attrPtr, uniformPtr, varyingPtr, privatePtr, texturePtr) {
    if (!this._currentProgram) {
      return;
    }

    if (tableIdx > 0 && this._sharedTable) {
      const func = this._sharedTable.get(tableIdx);
      if (func) {
        func(this._ctxHandle, type, tableIdx, attrPtr, uniformPtr, varyingPtr, privatePtr, texturePtr);
        return;
      }
    }

    const shaderInstance = type === this.VERTEX_SHADER ? this._currentProgram._vsInstance : this._currentProgram._fsInstance;
    if (shaderInstance && shaderInstance.exports.main) {
      // @ts-ignore
      shaderInstance.exports.main(this._ctxHandle, type, tableIdx, attrPtr, uniformPtr, varyingPtr, privatePtr, texturePtr);
    }
  }

  destroy() {
    if (this._destroyed) return;
    WasmWebGL2RenderingContext._contexts.delete(this._ctxHandle);
    const ex = this._instance.exports;
    if (ex && typeof ex.wasm_destroy_context === 'function') {
      const code = ex.wasm_destroy_context(this._ctxHandle);
      _checkErr(code, this._instance);
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
    // Return a thin wrapper object representing the texture.
    return new WasmWebGLTexture(this, handle);
  }

  deleteTexture(tex) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_delete_texture !== 'function') {
      throw new Error('wasm_ctx_delete_texture not found');
    }
    const handle = tex && typeof tex === 'object' && typeof tex._handle === 'number' ? tex._handle : (tex >>> 0);
    const code = ex.wasm_ctx_delete_texture(this._ctxHandle, handle);
    _checkErr(code, this._instance);
    // If a wrapper object was passed, mark it as deleted.
    if (tex && typeof tex === 'object') {
      try { tex._handle = 0; tex._deleted = true; } catch (e) { /* ignore */ }
    }
  }

  bindTexture(target, tex) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_bind_texture !== 'function') {
      throw new Error('wasm_ctx_bind_texture not found');
    }
    const handle = tex && typeof tex === 'object' && typeof tex._handle === 'number' ? tex._handle : (tex >>> 0);
    const code = ex.wasm_ctx_bind_texture(this._ctxHandle, target >>> 0, handle);
    _checkErr(code, this._instance);
    // Record bound texture in JS so we can map units to texture data for texel fetch
    this._boundTexture = handle;
    this._textureUnits = this._textureUnits || [];
    const unit = this._activeTextureUnit || 0;
    this._textureUnits[unit] = handle;
  }

  texImage2D(target, level, internalFormat, width, height, border, format, type_, pixels) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_tex_image_2d !== 'function') {
      throw new Error('wasm_ctx_tex_image_2d not found');
    }

    let data = pixels;
    if (!data) {
      data = new Uint8Array(width * height * 4);
    } else if (ArrayBuffer.isView(data)) {
      data = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
    } else if (data instanceof ArrayBuffer) {
      data = new Uint8Array(data);
    }

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

      // Mirror texture data in JS for fast texel fetches by shader imports
      this._textureData = this._textureData || new Map();
      const handle = this._boundTexture || 0;
      // Re-fetch memory in case it grew during the call (detaching the old buffer)
      const currentMem = new Uint8Array(ex.memory.buffer);
      const copy = new Uint8Array(currentMem.slice(ptr, ptr + len));
      this._textureData.set(handle, { width: width >>> 0, height: height >>> 0, data: copy });
    } finally {
      ex.wasm_free(ptr);
    }
  }

  texImage3D(target, level, internalFormat, width, height, depth, border, format, type_, pixels) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_tex_image_3d !== 'function') {
      throw new Error('wasm_ctx_tex_image_3d not found');
    }

    let data = pixels;
    if (!data) {
      data = new Uint8Array(width * height * depth * 4);
    } else if (ArrayBuffer.isView(data)) {
      data = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
    } else if (data instanceof ArrayBuffer) {
      data = new Uint8Array(data);
    }

    const len = data.length;
    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) throw new Error('Failed to allocate memory for pixel data');

    try {
      const mem = new Uint8Array(ex.memory.buffer);
      mem.set(data, ptr);

      const code = ex.wasm_ctx_tex_image_3d(
        this._ctxHandle,
        target >>> 0,
        level >>> 0,
        internalFormat >>> 0,
        width >>> 0,
        height >>> 0,
        depth >>> 0,
        border >>> 0,
        format >>> 0,
        type_ >>> 0,
        ptr >>> 0,
        len >>> 0
      );
      _checkErr(code, this._instance);

      // TODO: why texel fetches back in JS?????
      // Mirror texture data in JS for fast texel fetches by shader imports
      this._textureData = this._textureData || new Map();
      const handle = this._boundTexture || 0;
      // Re-fetch memory in case it grew during the call (detaching the old buffer)
      const currentMem = new Uint8Array(ex.memory.buffer);
      const copy = new Uint8Array(currentMem.slice(ptr, ptr + len));
      this._textureData.set(handle, { width: width >>> 0, height: height >>> 0, depth: depth >>> 0, data: copy });
    } finally {
      ex.wasm_free(ptr);
    }
  }

  copyTexImage2D(target, level, internalFormat, x, y, width, height, border) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_copy_tex_image_2d !== 'function') {
      throw new Error('wasm_ctx_copy_tex_image_2d not found');
    }
    const code = ex.wasm_ctx_copy_tex_image_2d(
      this._ctxHandle,
      target >>> 0,
      level >>> 0,
      internalFormat >>> 0,
      x | 0,
      y | 0,
      width >>> 0,
      height >>> 0,
      border >>> 0
    );
    _checkErr(code, this._instance);
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
    const texHandle = texture && typeof texture === 'object' && typeof texture._handle === 'number' ? texture._handle : (texture >>> 0);
    const code = ex.wasm_ctx_framebuffer_texture2d(
      this._ctxHandle,
      target >>> 0,
      attachment >>> 0,
      textarget >>> 0,
      texHandle,
      level >>> 0
    );
    _checkErr(code, this._instance);
  }

  createRenderbuffer() {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_create_renderbuffer !== 'function') {
      throw new Error('wasm_ctx_create_renderbuffer not found');
    }
    const handle = ex.wasm_ctx_create_renderbuffer(this._ctxHandle);
    if (handle === 0) {
      const msg = readErrorMessage(this._instance);
      throw new Error(`Failed to create renderbuffer: ${msg}`);
    }
    return new WasmWebGLRenderbuffer(this, handle);
  }

  bindRenderbuffer(target, renderbuffer) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_bind_renderbuffer !== 'function') {
      throw new Error('wasm_ctx_bind_renderbuffer not found');
    }
    const rbHandle = renderbuffer && typeof renderbuffer === 'object' && typeof renderbuffer._handle === 'number' ? renderbuffer._handle : (renderbuffer >>> 0);
    const code = ex.wasm_ctx_bind_renderbuffer(this._ctxHandle, target >>> 0, rbHandle);
    _checkErr(code, this._instance);
  }

  deleteRenderbuffer(renderbuffer) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_delete_renderbuffer !== 'function') {
      throw new Error('wasm_ctx_delete_renderbuffer not found');
    }
    const rbHandle = renderbuffer && typeof renderbuffer === 'object' && typeof renderbuffer._handle === 'number' ? renderbuffer._handle : (renderbuffer >>> 0);
    const code = ex.wasm_ctx_delete_renderbuffer(this._ctxHandle, rbHandle);
    _checkErr(code, this._instance);
  }

  isRenderbuffer(rb) {
    this._assertNotDestroyed();
    if (!rb || typeof rb !== 'object' || !(rb instanceof WasmWebGLRenderbuffer)) return false;
    if (rb._ctx !== this) return false;
    const ex = this._instance.exports;
    return ex.wasm_ctx_is_renderbuffer(this._ctxHandle, rb._handle) !== 0;
  }

  renderbufferStorage(target, internalFormat, width, height) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_renderbuffer_storage !== 'function') {
      throw new Error('wasm_ctx_renderbuffer_storage not found');
    }
    const code = ex.wasm_ctx_renderbuffer_storage(this._ctxHandle, target >>> 0, internalFormat >>> 0, width | 0, height | 0);
    _checkErr(code, this._instance);
  }

  framebufferRenderbuffer(target, attachment, renderbuffertarget, renderbuffer) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_framebuffer_renderbuffer !== 'function') {
      throw new Error('wasm_ctx_framebuffer_renderbuffer not found');
    }
    const rbHandle = renderbuffer && typeof renderbuffer === 'object' && typeof renderbuffer._handle === 'number' ? renderbuffer._handle : (renderbuffer >>> 0);
    const code = ex.wasm_ctx_framebuffer_renderbuffer(
      this._ctxHandle,
      target >>> 0,
      attachment >>> 0,
      renderbuffertarget >>> 0,
      rbHandle
    );
    _checkErr(code, this._instance);
  }

  readPixels(x, y, width, height, format, type_, out) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_read_pixels !== 'function') {
      throw new Error('wasm_ctx_read_pixels not found');
    }

    let bpp = 4;
    if (type_ === 0x1406) { // GL_FLOAT
      if (format === 0x1908) bpp = 16;      // GL_RGBA
      else if (format === 0x8227) bpp = 8;  // GL_RG
      else if (format === 0x1903) bpp = 4;  // GL_RED
    } else if (type_ === 0x1401) { // GL_UNSIGNED_BYTE
      if (format === 0x1908) bpp = 4;
    }

    const len = width * height * bpp;
    if (!out || out.byteLength < len) {
      throw new Error(`output buffer too small (need ${len} bytes, have ${out ? out.byteLength : 0})`);
    }

    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) throw new Error('Failed to allocate memory for readPixels output');

    try {
      const code = ex.wasm_ctx_read_pixels(
        this._ctxHandle,
        x | 0,
        y | 0,
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
      const out_bytes = new Uint8Array(out.buffer, out.byteOffset, len);
      out_bytes.set(src);
    } finally {
      ex.wasm_free(ptr);
    }
  }

  // --- Stubs for unimplemented WebGL2 methods (forwarding API surface) ---
  // These are intentionally not implemented in the prototype. They allow
  // callers to detect missing functionality early with a uniform error.

  createShader(type) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_create_shader !== 'function') {
      throw new Error('wasm_ctx_create_shader not found');
    }
    const handle = ex.wasm_ctx_create_shader(this._ctxHandle, type >>> 0);
    if (handle === 0) {
      const msg = readErrorMessage(this._instance);
      throw new Error(`Failed to create shader: ${msg}`);
    }
    return new WasmWebGLShader(this, handle);
  }

  shaderSource(shader, source) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_shader_source !== 'function') {
      throw new Error('wasm_ctx_shader_source not found');
    }

    const shaderHandle = shader && typeof shader === 'object' && typeof shader._handle === 'number' ? shader._handle : (shader >>> 0);
    const sourceStr = String(source);
    const bytes = new TextEncoder().encode(sourceStr);
    const len = bytes.length;
    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) throw new Error('Failed to allocate memory for shaderSource');

    try {
      const mem = new Uint8Array(ex.memory.buffer);
      mem.set(bytes, ptr);
      const code = ex.wasm_ctx_shader_source(this._ctxHandle, shaderHandle, ptr, len);
      _checkErr(code, this._instance);
    } finally {
      ex.wasm_free(ptr);
    }
  }

  compileShader(shader) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_compile_shader !== 'function') {
      throw new Error('wasm_ctx_compile_shader not found');
    }
    const shaderHandle = shader && typeof shader === 'object' && typeof shader._handle === 'number' ? shader._handle : (shader >>> 0);
    const code = ex.wasm_ctx_compile_shader(this._ctxHandle, shaderHandle);
    _checkErr(code, this._instance);
  }

  deleteShader(shader) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_delete_shader !== 'function') {
      throw new Error('wasm_ctx_delete_shader not found');
    }
    const shaderHandle = shader && typeof shader === 'object' && typeof shader._handle === 'number' ? shader._handle : (shader >>> 0);
    const code = ex.wasm_ctx_delete_shader(this._ctxHandle, shaderHandle);
    _checkErr(code, this._instance);
    if (shader && typeof shader === 'object') {
      try { shader._handle = 0; shader._deleted = true; } catch (e) { /* ignore */ }
    }
  }

  createProgram() {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_create_program !== 'function') {
      throw new Error('wasm_ctx_create_program not found');
    }
    const handle = ex.wasm_ctx_create_program(this._ctxHandle);
    if (handle === 0) {
      const msg = readErrorMessage(this._instance);
      throw new Error(`Failed to create program: ${msg}`);
    }
    return new WasmWebGLProgram(this, handle);
  }

  attachShader(program, shader) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_attach_shader !== 'function') {
      throw new Error('wasm_ctx_attach_shader not found');
    }
    const programHandle = program && typeof program === 'object' && typeof program._handle === 'number' ? program._handle : (program >>> 0);
    const shaderHandle = shader && typeof shader === 'object' && typeof shader._handle === 'number' ? shader._handle : (shader >>> 0);
    const code = ex.wasm_ctx_attach_shader(this._ctxHandle, programHandle, shaderHandle);
    _checkErr(code, this._instance);
  }

  detachShader(program, shader) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  getActiveUniform(program, index) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_get_active_uniform !== 'function') {
      throw new Error('wasm_ctx_get_active_uniform not found');
    }
    const programHandle = program && typeof program === 'object' && typeof program._handle === 'number' ? program._handle : (program >>> 0);

    // Allocate buffers: size(4) + type(4) + name(256)
    const sizePtr = ex.wasm_alloc(4);
    const typePtr = ex.wasm_alloc(4);
    const nameMaxLen = 256;
    const namePtr = ex.wasm_alloc(nameMaxLen);

    try {
      const nameLen = ex.wasm_ctx_get_active_uniform(
        this._ctxHandle,
        programHandle,
        index >>> 0,
        sizePtr,
        typePtr,
        namePtr,
        nameMaxLen
      );

      if (nameLen === 0) {
        return null;
      }

      const mem32SizeIdx = sizePtr >>> 2;
      const mem32TypeIdx = typePtr >>> 2;

      const size = new Int32Array(ex.memory.buffer)[mem32SizeIdx];
      const type_ = new Uint32Array(ex.memory.buffer)[mem32TypeIdx];

      const nameBytes = new Uint8Array(ex.memory.buffer, namePtr, nameLen);
      const name = new TextDecoder().decode(nameBytes);

      return { name, size, type: type_ };

    } finally {
      ex.wasm_free(sizePtr);
      ex.wasm_free(typePtr);
      ex.wasm_free(namePtr);
    }
  }

  getActiveAttrib(program, index) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_get_active_attrib !== 'function') {
      throw new Error('wasm_ctx_get_active_attrib not found');
    }
    const programHandle = program && typeof program === 'object' && typeof program._handle === 'number' ? program._handle : (program >>> 0);

    // Allocate buffers: size(4) + type(4) + name(256)
    const sizePtr = ex.wasm_alloc(4);
    const typePtr = ex.wasm_alloc(4);
    const nameMaxLen = 256;
    const namePtr = ex.wasm_alloc(nameMaxLen);

    try {
      const nameLen = ex.wasm_ctx_get_active_attrib(
        this._ctxHandle,
        programHandle,
        index >>> 0,
        sizePtr,
        typePtr,
        namePtr,
        nameMaxLen
      );

      if (nameLen === 0) {
        return null;
      }

      const mem32SizeIdx = sizePtr >>> 2;
      const mem32TypeIdx = typePtr >>> 2;

      const size = new Int32Array(ex.memory.buffer)[mem32SizeIdx];
      const type_ = new Uint32Array(ex.memory.buffer)[mem32TypeIdx];

      const nameBytes = new Uint8Array(ex.memory.buffer, namePtr, nameLen);
      const name = new TextDecoder().decode(nameBytes);

      return { name, size, type: type_ };

    } finally {
      ex.wasm_free(sizePtr);
      ex.wasm_free(typePtr);
      ex.wasm_free(namePtr);
    }
  }

  linkProgram(program) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_link_program !== 'function') {
      throw new Error('wasm_ctx_link_program not found');
    }
    const programHandle = program && typeof program === 'object' && typeof program._handle === 'number' ? program._handle : (program >>> 0);
    const code = ex.wasm_ctx_link_program(this._ctxHandle, programHandle);
    _checkErr(code, this._instance);

    // After linking, we need to instantiate the WASM modules on the host.
    if (program && typeof program === 'object') {
      const linkStatus = this.getProgramParameter(program, this.LINK_STATUS);
      if (linkStatus) {
        this._instantiateProgramShaders(program);
      }
    }
  }

  _instantiateProgramShaders(program) {
    const vsWasm = this.getProgramWasm(program, this.VERTEX_SHADER);
    const fsWasm = this.getProgramWasm(program, this.FRAGMENT_SHADER);

    if (!vsWasm || !fsWasm) {
      return;
    }

    // Allocate table slots for both shaders
    const vsIdx = this._tableAllocator ? this._tableAllocator.allocate() : null;
    const fsIdx = this._tableAllocator ? this._tableAllocator.allocate() : null;

    const createDebugEnv = (type, instanceRef) => {
      if (!this._debugShaders) return {};

      const stubCode = this.getProgramDebugStub(program, type);
      if (!stubCode) return {};

      // // Add sourceURL for debugging
      // const debugName = `shader_stub_program_${program._handle}_${type === this.VERTEX_SHADER ? 'vs' : 'fs'}.js`;
      // const codeWithUrl = stubCode + `\n//# sourceURL=${debugName}`;

      let stubFuncs;
      try {
        // Eval the stub array
        stubFuncs = (0, eval)(stubCode);
      } catch (e) {
        console.error("Failed to eval debug stub:", e);
        return {};
      }

      return {
        debug_step: (line, funcIdx, resultPtr) => {
          if (line === 999999) {
            return;
          }
          const func = stubFuncs[line - 1];
          if (func) {
            const ctx = {
              go: () => {
                // Trampoline logic would go here
                // For now we rely on WASM calling the function after debug_step returns
              }
            };
            try {
              func.call(ctx);
            } catch (e) {
              console.error("Error in debug stub:", e);
            }
          }
        }
      };
    };

    let vsModule;
    vsModule = new WebAssembly.Module(vsWasm);
    const vsInstanceRef = { current: null };
    const vsDebugEnv = createDebugEnv(this.VERTEX_SHADER, vsInstanceRef);

    const env = {
      memory: this._instance.exports.memory,
      __indirect_function_table: this._sharedTable,
      ...vsDebugEnv
    };

    // Add math builtins from renderer (skipping host)
    const mathFuncs = [
      'gl_sin', 'gl_cos', 'gl_tan', 'gl_asin', 'gl_acos', 'gl_atan', 'gl_atan2',
      'gl_exp', 'gl_exp2', 'gl_log', 'gl_log2', 'gl_pow',
      'gl_sinh', 'gl_cosh', 'gl_tanh', 'gl_asinh', 'gl_acosh', 'gl_atanh'
    ];
    for (const name of mathFuncs) {
      if (this._instance.exports[name]) {
        env[name] = this._instance.exports[name];
      }
    }

    program._vsInstance = new WebAssembly.Instance(vsModule, {
      env
    });
    vsInstanceRef.current = program._vsInstance;

    // Register in table
    if (this._sharedTable && vsIdx !== null && program._vsInstance.exports.main) {
      this._sharedTable.set(vsIdx, program._vsInstance.exports.main);
      program._vsTableIndex = vsIdx;
    }

    let fsModule;
    fsModule = new WebAssembly.Module(fsWasm);
    const fsInstanceRef = { current: null };
    const fsDebugEnv = createDebugEnv(this.FRAGMENT_SHADER, fsInstanceRef);

    const fsEnv = {
      memory: this._instance.exports.memory,
      __indirect_function_table: this._sharedTable,
      ...fsDebugEnv
    };

    for (const name of mathFuncs) {
      if (this._instance.exports[name]) {
        fsEnv[name] = this._instance.exports[name];
      }
    }

    program._fsInstance = new WebAssembly.Instance(fsModule, {
      env: fsEnv
    });
    fsInstanceRef.current = program._fsInstance;

    // Register in table
    if (this._sharedTable && fsIdx !== null && program._fsInstance.exports.main) {
      this._sharedTable.set(fsIdx, program._fsInstance.exports.main);
      program._fsTableIndex = fsIdx;
    }

    // Notify Rust of table indices (requires Phase 4)
    if (vsIdx !== null && fsIdx !== null) {
      const ex = this._instance.exports;
      if (ex.wasm_ctx_register_shader_indices) {
        ex.wasm_ctx_register_shader_indices(
          this._ctxHandle,
          program._handle,
          vsIdx,
          fsIdx
        );
      }
    }
  }

  getProgramDebugStub(program, shaderType) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_get_program_debug_stub !== 'function') {
      return null;
    }
    const programHandle = program && typeof program === 'object' && typeof program._handle === 'number' ? program._handle : (program >>> 0);
    const len = ex.wasm_ctx_get_program_debug_stub(this._ctxHandle, programHandle, shaderType, 0, 0);
    if (len === 0) return null;

    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) return null;

    try {
      const actualLen = ex.wasm_ctx_get_program_debug_stub(this._ctxHandle, programHandle, shaderType, ptr, len);
      const mem = new Uint8Array(ex.memory.buffer);
      const bytes = mem.subarray(ptr, ptr + actualLen);
      return new TextDecoder().decode(bytes);
    } finally {
      ex.wasm_free(ptr);
    }
  }

  getProgramDebugStub(program, shaderType) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_get_program_debug_stub !== 'function') {
      return null;
    }
    const programHandle = program && typeof program === 'object' && typeof program._handle === 'number' ? program._handle : (program >>> 0);
    const len = ex.wasm_ctx_get_program_debug_stub(this._ctxHandle, programHandle, shaderType, 0, 0);
    if (len === 0) return null;

    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) return null;

    try {
      const actualLen = ex.wasm_ctx_get_program_debug_stub(this._ctxHandle, programHandle, shaderType, ptr, len);
      const mem = new Uint8Array(ex.memory.buffer);
      const bytes = mem.subarray(ptr, ptr + actualLen);
      return new TextDecoder().decode(bytes);
    } finally {
      ex.wasm_free(ptr);
    }
  }

  deleteProgram(program) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_delete_program !== 'function') {
      throw new Error('wasm_ctx_delete_program not found');
    }
    const programHandle = program && typeof program === 'object' && typeof program._handle === 'number' ? program._handle : (program >>> 0);

    // Free table indices
    if (program && typeof program === 'object') {
      if (program._vsTableIndex !== undefined && this._tableAllocator) {
        this._tableAllocator.free(program._vsTableIndex);
      }
      if (program._fsTableIndex !== undefined && this._tableAllocator) {
        this._tableAllocator.free(program._fsTableIndex);
      }
    }

    const code = ex.wasm_ctx_delete_program(this._ctxHandle, programHandle);
    _checkErr(code, this._instance);
    if (program && typeof program === 'object') {
      try { program._handle = 0; program._deleted = true; } catch (e) { /* ignore */ }
    }
  }

  useProgram(program) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_use_program !== 'function') {
      throw new Error('wasm_ctx_use_program not found');
    }
    const programHandle = program && typeof program === 'object' && typeof program._handle === 'number' ? program._handle : (program >>> 0);
    const code = ex.wasm_ctx_use_program(this._ctxHandle, programHandle);
    _checkErr(code, this._instance);
    this._currentProgram = program;
  }

  getShaderParameter(shader, pname) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_get_shader_parameter !== 'function') {
      throw new Error('wasm_ctx_get_shader_parameter not found');
    }
    const shaderHandle = shader && typeof shader === 'object' && typeof shader._handle === 'number' ? shader._handle : (shader >>> 0);
    const val = ex.wasm_ctx_get_shader_parameter(this._ctxHandle, shaderHandle, pname >>> 0);

    // WebGL returns boolean for status parameters
    if (pname === 0x8B81 /* COMPILE_STATUS */ || pname === 0x8B80 /* DELETE_STATUS */) {
      return !!val;
    }
    return val;
  }

  getProgramParameter(program, pname) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_get_program_parameter !== 'function') {
      throw new Error('wasm_ctx_get_program_parameter not found');
    }
    const programHandle = program && typeof program === 'object' && typeof program._handle === 'number' ? program._handle : (program >>> 0);
    const val = ex.wasm_ctx_get_program_parameter(this._ctxHandle, programHandle, pname >>> 0);

    // WebGL returns boolean for status parameters
    if (pname === 0x8B82 /* LINK_STATUS */ || pname === 0x8B80 /* DELETE_STATUS */ || pname === 0x8B83 /* VALIDATE_STATUS */) {
      return !!val;
    }
    return val;
  }

  getShaderInfoLog(shader) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_get_shader_info_log !== 'function') {
      throw new Error('wasm_ctx_get_shader_info_log not found');
    }
    const shaderHandle = shader && typeof shader === 'object' && typeof shader._handle === 'number' ? shader._handle : (shader >>> 0);

    const maxLen = 1024;
    const ptr = ex.wasm_alloc(maxLen);
    if (ptr === 0) throw new Error('Failed to allocate memory for getShaderInfoLog');

    try {
      const len = ex.wasm_ctx_get_shader_info_log(this._ctxHandle, shaderHandle, ptr, maxLen);
      const mem = new Uint8Array(ex.memory.buffer);
      const bytes = mem.subarray(ptr, ptr + len);
      return new TextDecoder().decode(bytes);
    } finally {
      ex.wasm_free(ptr);
    }
  }

  getProgramInfoLog(program) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_get_program_info_log !== 'function') {
      throw new Error('wasm_ctx_get_program_info_log not found');
    }
    const programHandle = program && typeof program === 'object' && typeof program._handle === 'number' ? program._handle : (program >>> 0);

    const maxLen = 1024;
    const ptr = ex.wasm_alloc(maxLen);
    if (ptr === 0) throw new Error('Failed to allocate memory for getProgramInfoLog');

    try {
      const len = ex.wasm_ctx_get_program_info_log(this._ctxHandle, programHandle, ptr, maxLen);
      const mem = new Uint8Array(ex.memory.buffer);
      const bytes = mem.subarray(ptr, ptr + len);
      return new TextDecoder().decode(bytes);
    } finally {
      ex.wasm_free(ptr);
    }
  }

  getProgramWasm(program, shaderType) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_get_program_wasm_len !== 'function') {
      throw new Error('wasm_ctx_get_program_wasm_len not found');
    }
    const programHandle = program && typeof program === 'object' && typeof program._handle === 'number' ? program._handle : (program >>> 0);
    const len = ex.wasm_ctx_get_program_wasm_len(this._ctxHandle, programHandle, shaderType);
    if (len === 0) return null;

    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) throw new Error('Failed to allocate memory for getProgramWasm');

    try {
      const actualLen = ex.wasm_ctx_get_program_wasm(this._ctxHandle, programHandle, shaderType, ptr, len);
      const mem = new Uint8Array(ex.memory.buffer);
      return new Uint8Array(mem.buffer, ptr, actualLen).slice();
    } finally {
      ex.wasm_free(ptr);
    }
  }

  getAttribLocation(program, name) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_get_attrib_location !== 'function') {
      throw new Error('wasm_ctx_get_attrib_location not found');
    }
    const programHandle = program && typeof program === 'object' && typeof program._handle === 'number' ? program._handle : (program >>> 0);
    const nameStr = String(name);
    const bytes = new TextEncoder().encode(nameStr);
    const len = bytes.length;
    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) throw new Error('Failed to allocate memory for getAttribLocation');

    try {
      const mem = new Uint8Array(ex.memory.buffer);
      mem.set(bytes, ptr);
      return ex.wasm_ctx_get_attrib_location(this._ctxHandle, programHandle, ptr, len);
    } finally {
      ex.wasm_free(ptr);
    }
  }

  bindAttribLocation(program, index, name) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_bind_attrib_location !== 'function') {
      throw new Error('wasm_ctx_bind_attrib_location not found');
    }
    const programHandle = program && typeof program === 'object' && typeof program._handle === 'number' ? program._handle : (program >>> 0);
    const nameStr = String(name);
    const bytes = new TextEncoder().encode(nameStr);
    const len = bytes.length;
    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) throw new Error('Failed to allocate memory for bindAttribLocation');

    try {
      const mem = new Uint8Array(ex.memory.buffer);
      mem.set(bytes, ptr);
      const code = ex.wasm_ctx_bind_attrib_location(this._ctxHandle, programHandle, index >>> 0, ptr, len);
      _checkErr(code, this._instance);
    } finally {
      ex.wasm_free(ptr);
    }
  }

  enableVertexAttribArray(index) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_enable_vertex_attrib_array !== 'function') {
      throw new Error('wasm_ctx_enable_vertex_attrib_array not found');
    }
    const code = ex.wasm_ctx_enable_vertex_attrib_array(this._ctxHandle, index >>> 0);
    _checkErr(code, this._instance);
  }

  disableVertexAttribArray(index) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_disable_vertex_attrib_array !== 'function') {
      throw new Error('wasm_ctx_disable_vertex_attrib_array not found');
    }
    const code = ex.wasm_ctx_disable_vertex_attrib_array(this._ctxHandle, index >>> 0);
    _checkErr(code, this._instance);
  }

  vertexAttribPointer(index, size, type, normalized, stride, offset) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_vertex_attrib_pointer !== 'function') {
      throw new Error('wasm_ctx_vertex_attrib_pointer not found');
    }
    const code = ex.wasm_ctx_vertex_attrib_pointer(
      this._ctxHandle,
      index >>> 0,
      size >>> 0,
      type >>> 0,
      normalized ? 1 : 0,
      stride >>> 0,
      offset >>> 0
    );
    if (code === 5) return; // ERR_GL
    _checkErr(code, this._instance);
  }

  vertexAttribIPointer(index, size, type, stride, offset) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_vertex_attrib_ipointer !== 'function') {
      throw new Error('wasm_ctx_vertex_attrib_ipointer not found');
    }
    const code = ex.wasm_ctx_vertex_attrib_ipointer(
      this._ctxHandle,
      index >>> 0,
      size >>> 0,
      type >>> 0,
      stride >>> 0,
      offset >>> 0
    );
    if (code === 5) return; // ERR_GL
    _checkErr(code, this._instance);
  }

  vertexAttrib1f(index, v0) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_vertex_attrib1f !== 'function') {
      throw new Error('wasm_ctx_vertex_attrib1f not found');
    }
    const code = ex.wasm_ctx_vertex_attrib1f(this._ctxHandle, index >>> 0, +v0);
    if (code === 5) return; // ERR_GL
    _checkErr(code, this._instance);
  }
  vertexAttrib2f(index, v0, v1) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_vertex_attrib2f !== 'function') {
      throw new Error('wasm_ctx_vertex_attrib2f not found');
    }
    const code = ex.wasm_ctx_vertex_attrib2f(this._ctxHandle, index >>> 0, +v0, +v1);
    if (code === 5) return; // ERR_GL
    _checkErr(code, this._instance);
  }
  vertexAttrib3f(index, v0, v1, v2) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_vertex_attrib3f !== 'function') {
      throw new Error('wasm_ctx_vertex_attrib3f not found');
    }
    const code = ex.wasm_ctx_vertex_attrib3f(this._ctxHandle, index >>> 0, +v0, +v1, +v2);
    if (code === 5) return; // ERR_GL
    _checkErr(code, this._instance);
  }
  vertexAttrib4f(index, v0, v1, v2, v3) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_vertex_attrib4f !== 'function') {
      throw new Error('wasm_ctx_vertex_attrib4f not found');
    }
    const code = ex.wasm_ctx_vertex_attrib4f(this._ctxHandle, index >>> 0, +v0, +v1, +v2, +v3);
    if (code === 5) return; // ERR_GL
    _checkErr(code, this._instance);
  }

  vertexAttrib1fv(index, v) {
    if (v && v.length >= 1) {
      this.vertexAttrib1f(index, v[0]);
    } else {
      this._setError(0x0501);
    }
  }
  vertexAttrib2fv(index, v) {
    if (v && v.length >= 2) {
      this.vertexAttrib2f(index, v[0], v[1]);
    } else {
      this._setError(0x0501);
    }
  }
  vertexAttrib3fv(index, v) {
    if (v && v.length >= 3) {
      this.vertexAttrib3f(index, v[0], v[1], v[2]);
    } else {
      this._setError(0x0501);
    }
  }
  vertexAttrib4fv(index, v) {
    if (v && v.length >= 4) {
      this.vertexAttrib4f(index, v[0], v[1], v[2], v[3]);
    } else {
      this._setError(0x0501);
    }
  }

  vertexAttribI4i(index, v0, v1, v2, v3) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_vertex_attrib_i4i !== 'function') {
      throw new Error('wasm_ctx_vertex_attrib_i4i not found');
    }
    const code = ex.wasm_ctx_vertex_attrib_i4i(this._ctxHandle, index >>> 0, v0 | 0, v1 | 0, v2 | 0, v3 | 0);
    if (code === 5) return; // ERR_GL
    _checkErr(code, this._instance);
  }

  vertexAttribI4ui(index, v0, v1, v2, v3) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_vertex_attrib_i4ui !== 'function') {
      throw new Error('wasm_ctx_vertex_attrib_i4ui not found');
    }
    const code = ex.wasm_ctx_vertex_attrib_i4ui(this._ctxHandle, index >>> 0, v0 >>> 0, v1 >>> 0, v2 >>> 0, v3 >>> 0);
    if (code === 5) return; // ERR_GL
    _checkErr(code, this._instance);
  }

  vertexAttribI4iv(index, v) {
    if (v && v.length >= 4) {
      this.vertexAttribI4i(index, v[0], v[1], v[2], v[3]);
    } else {
      this._setError(0x0501);
    }
  }

  vertexAttribI4uiv(index, v) {
    if (v && v.length >= 4) {
      this.vertexAttribI4ui(index, v[0], v[1], v[2], v[3]);
    } else {
      this._setError(0x0501);
    }
  }

  vertexAttribDivisor(index, divisor) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_vertex_attrib_divisor !== 'function') {
      throw new Error('wasm_ctx_vertex_attrib_divisor not found');
    }
    const code = ex.wasm_ctx_vertex_attrib_divisor(this._ctxHandle, index >>> 0, divisor >>> 0);
    _checkErr(code, this._instance);
  }

  createBuffer() {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_create_buffer !== 'function') {
      throw new Error('wasm_ctx_create_buffer not found');
    }
    const handle = ex.wasm_ctx_create_buffer(this._ctxHandle);
    if (handle === 0) {
      const msg = readErrorMessage(this._instance);
      throw new Error(`Failed to create buffer: ${msg}`);
    }
    return new WasmWebGLBuffer(this, handle);
  }

  bindBuffer(target, buffer) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_bind_buffer !== 'function') {
      throw new Error('wasm_ctx_bind_buffer not found');
    }
    const handle = buffer && typeof buffer === 'object' && typeof buffer._handle === 'number' ? buffer._handle : (buffer >>> 0);
    const code = ex.wasm_ctx_bind_buffer(this._ctxHandle, target >>> 0, handle);
    _checkErr(code, this._instance);
  }

  deleteBuffer(buffer) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_delete_buffer !== 'function') {
      throw new Error('wasm_ctx_delete_buffer not found');
    }
    const handle = buffer && typeof buffer === 'object' && typeof buffer._handle === 'number' ? buffer._handle : (buffer >>> 0);
    const code = ex.wasm_ctx_delete_buffer(this._ctxHandle, handle);
    _checkErr(code, this._instance); if (buffer && typeof buffer === 'object') {
      try { buffer._handle = 0; buffer._deleted = true; } catch (e) { /* ignore */ }
    }
  }

  bufferData(target, data, usage) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_buffer_data !== 'function') {
      throw new Error('wasm_ctx_buffer_data not found');
    }

    let bytes;
    if (data instanceof ArrayBuffer) {
      bytes = new Uint8Array(data);
    } else if (ArrayBuffer.isView(data)) {
      bytes = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
    } else if (typeof data === 'number') {
      bytes = new Uint8Array(data);
    } else {
      throw new Error('Invalid data type for bufferData');
    }

    const len = bytes.length;
    if (len === 0) {
      const code = ex.wasm_ctx_buffer_data(this._ctxHandle, target >>> 0, 0, 0, usage >>> 0);
      _checkErr(code, this._instance);
      return;
    }

    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) throw new Error('Failed to allocate memory for bufferData');

    try {
      const mem = new Uint8Array(ex.memory.buffer);
      mem.set(bytes, ptr);
      const code = ex.wasm_ctx_buffer_data(this._ctxHandle, target >>> 0, ptr, len, usage >>> 0);
      _checkErr(code, this._instance);
    } finally {
      ex.wasm_free(ptr);
    }
  }

  bufferSubData(target, offset, data) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_buffer_sub_data !== 'function') {
      throw new Error('wasm_ctx_buffer_sub_data not found');
    }

    let bytes;
    if (data instanceof Uint8Array) bytes = data;
    else if (ArrayBuffer.isView(data)) bytes = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
    else if (data instanceof ArrayBuffer) bytes = new Uint8Array(data);
    else bytes = new Uint8Array(data); // Fallback for arrays

    const len = bytes.length;
    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) throw new Error('Failed to allocate memory for bufferSubData');

    try {
      const mem = new Uint8Array(ex.memory.buffer);
      mem.set(bytes, ptr);
      const code = ex.wasm_ctx_buffer_sub_data(this._ctxHandle, target >>> 0, offset >>> 0, ptr, len);
      _checkErr(code, this._instance);
    } finally {
      ex.wasm_free(ptr);
    }
  }
  copyBufferSubData(readTarget, writeTarget, readOffset, writeOffset, size) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_copy_buffer_sub_data !== 'function') {
      throw new Error('wasm_ctx_copy_buffer_sub_data not found');
    }
    const code = ex.wasm_ctx_copy_buffer_sub_data(
      this._ctxHandle,
      readTarget >>> 0,
      writeTarget >>> 0,
      readOffset >>> 0,
      writeOffset >>> 0,
      size >>> 0
    );
    _checkErr(code, this._instance);
  }
  getBufferParameter(target, pname) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_get_buffer_parameter !== 'function') {
      throw new Error('wasm_ctx_get_buffer_parameter not found');
    }
    const val = ex.wasm_ctx_get_buffer_parameter(this._ctxHandle, target >>> 0, pname >>> 0);
    if (val < 0) {
      const msg = readErrorMessage(this._instance);
      throw new Error(`getBufferParameter failed: ${msg}`);
    }
    return val;
  }
  isBuffer(buffer) {
    this._assertNotDestroyed();
    if (!buffer || typeof buffer !== 'object' || !(buffer instanceof WasmWebGLBuffer)) return false;
    if (buffer._ctx !== this) return false;
    const ex = this._instance.exports;
    return ex.wasm_ctx_is_buffer(this._ctxHandle, buffer._handle) !== 0;
  }

  drawArrays(mode, first, count) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_draw_arrays !== 'function') {
      throw new Error('wasm_ctx_draw_arrays not found');
    }
    const code = ex.wasm_ctx_draw_arrays(this._ctxHandle, mode >>> 0, first >>> 0, count >>> 0);
    _checkErr(code, this._instance);
  }

  drawElements(mode, count, type, offset) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_draw_elements !== 'function') {
      throw new Error('wasm_ctx_draw_elements not found');
    }
    const code = ex.wasm_ctx_draw_elements(this._ctxHandle, mode >>> 0, count >>> 0, type >>> 0, offset >>> 0);
    _checkErr(code, this._instance);
  }
  drawArraysInstanced(mode, first, count, instanceCount) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_draw_arrays_instanced !== 'function') {
      throw new Error('wasm_ctx_draw_arrays_instanced not found');
    }
    const code = ex.wasm_ctx_draw_arrays_instanced(this._ctxHandle, mode >>> 0, first | 0, count | 0, instanceCount | 0);
    _checkErr(code, this._instance);
  }
  drawElementsInstanced(mode, count, type, offset, instanceCount) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_draw_elements_instanced !== 'function') {
      throw new Error('wasm_ctx_draw_elements_instanced not found');
    }
    const code = ex.wasm_ctx_draw_elements_instanced(this._ctxHandle, mode >>> 0, count | 0, type >>> 0, offset >>> 0, instanceCount | 0);
    _checkErr(code, this._instance);
  }
  drawRangeElements(mode, start, end, count, type, offset) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  drawBuffers(buffers) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  createVertexArray() {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_create_vertex_array !== 'function') {
      throw new Error('wasm_ctx_create_vertex_array not found');
    }
    const handle = ex.wasm_ctx_create_vertex_array(this._ctxHandle);
    if (handle === 0) return null;
    return { _handle: handle, _type: 'WebGLVertexArrayObject' };
  }

  bindVertexArray(vao) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_bind_vertex_array !== 'function') {
      throw new Error('wasm_ctx_bind_vertex_array not found');
    }
    const handle = vao && typeof vao === 'object' && typeof vao._handle === 'number' ? vao._handle : (vao ? (vao >>> 0) : 0);
    const code = ex.wasm_ctx_bind_vertex_array(this._ctxHandle, handle);
    _checkErr(code, this._instance);
  }

  deleteVertexArray(vao) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_delete_vertex_array !== 'function') {
      throw new Error('wasm_ctx_delete_vertex_array not found');
    }
    const handle = vao && typeof vao === 'object' && typeof vao._handle === 'number' ? vao._handle : (vao >>> 0);
    const code = ex.wasm_ctx_delete_vertex_array(this._ctxHandle, handle);
    _checkErr(code, this._instance);
    if (vao && typeof vao === 'object') {
      try { vao._handle = 0; vao._deleted = true; } catch (e) { /* ignore */ }
    }
  }

  isVertexArray(vao) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_is_vertex_array !== 'function') {
      throw new Error('wasm_ctx_is_vertex_array not found');
    }
    const handle = vao && typeof vao === 'object' && typeof vao._handle === 'number' ? vao._handle : (vao >>> 0);
    const res = ex.wasm_ctx_is_vertex_array(this._ctxHandle, handle);
    return res !== 0;
  }

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

  activeTexture(texture) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_active_texture !== 'function') {
      throw new Error('wasm_ctx_active_texture not found');
    }
    const code = ex.wasm_ctx_active_texture(this._ctxHandle, texture >>> 0);
    _checkErr(code, this._instance);
    // Track active texture unit in JS wrapper (GL_TEXTURE0 = 0x84C0)
    this._activeTextureUnit = (texture >>> 0) - 0x84C0;
    this._textureUnits = this._textureUnits || [];
  }
  texParameteri(target, pname, param) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_tex_parameter_i !== 'function') {
      throw new Error('wasm_ctx_tex_parameter_i not found');
    }
    const code = ex.wasm_ctx_tex_parameter_i(this._ctxHandle, target >>> 0, pname >>> 0, param | 0);
    _checkErr(code, this._instance);
  }
  generateMipmap(target) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_generate_mipmap !== 'function') {
      throw new Error('wasm_ctx_generate_mipmap not found');
    }
    const code = ex.wasm_ctx_generate_mipmap(this._ctxHandle, target >>> 0);
    _checkErr(code, this._instance);
  }

  copyTexImage2D(target, level, internalformat, x, y, width, height, border) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_copy_tex_image_2d !== 'function') {
      throw new Error('wasm_ctx_copy_tex_image_2d not found');
    }
    const code = ex.wasm_ctx_copy_tex_image_2d(
      this._ctxHandle,
      target >>> 0,
      level | 0,
      internalformat >>> 0,
      x | 0,
      y | 0,
      width | 0,
      height | 0,
      border | 0
    );
    _checkErr(code, this._instance);
  }
  copyTexSubImage2D(target, level, xoffset, yoffset, x, y, width, height) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  texSubImage2D(target, level, xoffset, yoffset, width, height, format, type_, pixels) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_tex_sub_image_2d !== 'function') {
      throw new Error('wasm_ctx_tex_sub_image_2d not found');
    }

    let data = pixels;
    if (!data) return; // No-op if no data provided
    if (!(data instanceof Uint8Array)) {
      if (ArrayBuffer.isView(data)) {
        data = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
      } else {
        data = new Uint8Array(data);
      }
    }

    const len = data.length;
    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) throw new Error('Failed to allocate memory for sub-pixel data');

    try {
      const mem = new Uint8Array(ex.memory.buffer);
      mem.set(data, ptr);

      const code = ex.wasm_ctx_tex_sub_image_2d(
        this._ctxHandle,
        target >>> 0,
        level >>> 0,
        xoffset | 0,
        yoffset | 0,
        width >>> 0,
        height >>> 0,
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

  checkFramebufferStatus(target) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  blitFramebuffer(srcX0, srcY0, srcX1, srcY1, dstX0, dstY0, dstX1, dstY1, mask, filter) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_blit_framebuffer !== 'function') {
      throw new Error('wasm_ctx_blit_framebuffer not found');
    }
    const code = ex.wasm_ctx_blit_framebuffer(
      this._ctxHandle,
      srcX0 | 0, srcY0 | 0, srcX1 | 0, srcY1 | 0,
      dstX0 | 0, dstY0 | 0, dstX1 | 0, dstY1 | 0,
      mask >>> 0, filter >>> 0
    );
    _checkErr(code, this._instance);
  }
  readBuffer(src) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  pixelStorei(pname, param) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  getExtension(name) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  getSupportedExtensions() { this._assertNotDestroyed(); throw new Error('not implemented'); }

  getUniformLocation(program, name) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_get_uniform_location !== 'function') {
      throw new Error('wasm_ctx_get_uniform_location not found');
    }
    const programHandle = program && typeof program === 'object' && typeof program._handle === 'number' ? program._handle : (program >>> 0);
    const nameStr = String(name);
    const bytes = new TextEncoder().encode(nameStr);
    const len = bytes.length;
    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) throw new Error('Failed to allocate memory for getUniformLocation');

    try {
      const mem = new Uint8Array(ex.memory.buffer);
      mem.set(bytes, ptr);
      const loc = ex.wasm_ctx_get_uniform_location(this._ctxHandle, programHandle, ptr, len);
      return loc === -1 ? null : loc;
    } finally {
      ex.wasm_free(ptr);
    }
  }

  uniform1f(loc, x) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_uniform1f !== 'function') {
      throw new Error('wasm_ctx_uniform1f not found');
    }
    const locHandle = loc === null ? -1 : (typeof loc === 'number' ? loc : (loc._handle >>> 0));
    const code = ex.wasm_ctx_uniform1f(this._ctxHandle, locHandle, +x);
    _checkErr(code, this._instance);
  }

  uniform2f(loc, x, y) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_uniform2f !== 'function') {
      throw new Error('wasm_ctx_uniform2f not found');
    }
    const locHandle = loc === null ? -1 : (typeof loc === 'number' ? loc : (loc._handle >>> 0));
    const code = ex.wasm_ctx_uniform2f(this._ctxHandle, locHandle, +x, +y);
    _checkErr(code, this._instance);
  }

  uniform3f(loc, x, y, z) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_uniform3f !== 'function') {
      throw new Error('wasm_ctx_uniform3f not found');
    }
    const locHandle = loc === null ? -1 : (typeof loc === 'number' ? loc : (loc._handle >>> 0));
    const code = ex.wasm_ctx_uniform3f(this._ctxHandle, locHandle, +x, +y, +z);
    _checkErr(code, this._instance);
  }

  uniform4f(loc, x, y, z, w) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_uniform4f !== 'function') {
      throw new Error('wasm_ctx_uniform4f not found');
    }
    const locHandle = loc === null ? -1 : (typeof loc === 'number' ? loc : (loc._handle >>> 0));
    const code = ex.wasm_ctx_uniform4f(this._ctxHandle, locHandle, +x, +y, +z, +w);
    _checkErr(code, this._instance);
  }

  uniform1i(loc, x) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_uniform1i !== 'function') {
      throw new Error('wasm_ctx_uniform1i not found');
    }
    const locHandle = loc === null ? -1 : (typeof loc === 'number' ? loc : (loc._handle >>> 0));
    const code = ex.wasm_ctx_uniform1i(this._ctxHandle, locHandle, x | 0);
    _checkErr(code, this._instance);
  }

  uniformMatrix4fv(loc, transpose, value) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_uniform_matrix_4fv !== 'function') {
      throw new Error('wasm_ctx_uniform_matrix_4fv not found');
    }
    const locHandle = loc === null ? -1 : (typeof loc === 'number' ? loc : (loc._handle >>> 0));

    let bytes;
    if (value instanceof Float32Array) {
      bytes = new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
    } else {
      bytes = new Uint8Array(new Float32Array(value).buffer);
    }

    const len = bytes.length;
    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) throw new Error('Failed to allocate memory for uniformMatrix4fv');

    try {
      const mem = new Uint8Array(ex.memory.buffer);
      mem.set(bytes, ptr);
      const count = len / 4;
      const code = ex.wasm_ctx_uniform_matrix_4fv(this._ctxHandle, locHandle, transpose ? 1 : 0, ptr, count);
      _checkErr(code, this._instance);
    } finally {
      ex.wasm_free(ptr);
    }
  }

  getVertexAttrib(index, pname) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_get_vertex_attrib !== 'function') {
      throw new Error('wasm_ctx_get_vertex_attrib not found');
    }

    // Allocate memory for result.
    // Most params return 1 int (4 bytes).
    // CURRENT_VERTEX_ATTRIB returns 4 values (16 bytes) + type (4 bytes) = 20 bytes.
    const len = 20;
    const ptr = ex.wasm_alloc(len);
    if (ptr === 0) throw new Error('Failed to allocate memory for getVertexAttrib');

    try {
      const code = ex.wasm_ctx_get_vertex_attrib(this._ctxHandle, index >>> 0, pname >>> 0, ptr, len);
      if (code === 5) { // ERR_GL
        return undefined;
      }
      _checkErr(code, this._instance);

      const mem = new Int32Array(ex.memory.buffer, ptr, 5);
      const memU = new Uint32Array(ex.memory.buffer, ptr, 5);
      const memF = new Float32Array(ex.memory.buffer, ptr, 5);

      if (pname === 0x8626 /* CURRENT_VERTEX_ATTRIB */) {
        // Check type at index 4
        const type = memU[4];
        if (type === 0x1404 /* INT */) {
          return new Int32Array([mem[0], mem[1], mem[2], mem[3]]);
        } else if (type === 0x1405 /* UNSIGNED_INT */) {
          return new Uint32Array([memU[0], memU[1], memU[2], memU[3]]);
        } else {
          // Default to float
          return new Float32Array([memF[0], memF[1], memF[2], memF[3]]);
        }
      }

      // Other params
      if (pname === 0x8622 /* ENABLED */ ||
        pname === 0x886A /* NORMALIZED */ ||
        pname === 0x88FD /* INTEGER */) {
        return mem[0] !== 0;
      }

      if (pname === 0x889F /* BUFFER_BINDING */) {
        const handle = memU[0];
        if (handle === 0) return null;
        return new WasmWebGLBuffer(this, handle);
      }

      return mem[0];
    } finally {
      ex.wasm_free(ptr, len);
    }
  }


  getParameter(pname) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_get_parameter_v !== 'function') {
      throw new Error('wasm_ctx_get_parameter_v not found');
    }

    if (pname === 0x0BA2 /* VIEWPORT */) {
      const ptr = ex.wasm_alloc(16);
      try {
        const code = ex.wasm_ctx_get_parameter_v(this._ctxHandle, pname, ptr, 16);
        _checkErr(code, this._instance);
        const mem = new Int32Array(ex.memory.buffer, ptr, 4);
        return new Int32Array(mem);
      } finally {
        ex.wasm_free(ptr, 16);
      }
    }

    if (pname === 0x0C22 /* COLOR_CLEAR_VALUE */) {
      const ptr = ex.wasm_alloc(16);
      try {
        const code = ex.wasm_ctx_get_parameter_v(this._ctxHandle, pname, ptr, 16);
        _checkErr(code, this._instance);
        const mem = new Float32Array(ex.memory.buffer, ptr, 4);
        return new Float32Array(mem);
      } finally {
        ex.wasm_free(ptr, 16);
      }
    }

    if (pname === 0x0C23 /* COLOR_WRITEMASK */) {
      const ptr = ex.wasm_alloc(4);
      try {
        const code = ex.wasm_ctx_get_parameter_v(this._ctxHandle, pname, ptr, 4);
        _checkErr(code, this._instance);
        const mem = new Uint8Array(ex.memory.buffer, ptr, 4);
        return [mem[0] !== 0, mem[1] !== 0, mem[2] !== 0, mem[3] !== 0];
      } finally {
        ex.wasm_free(ptr, 4);
      }
    }

    if (pname === 0x0B72 /* DEPTH_WRITEMASK */) {
      const ptr = ex.wasm_alloc(4);
      try {
        const code = ex.wasm_ctx_get_parameter_v(this._ctxHandle, pname, ptr, 4);
        _checkErr(code, this._instance);
        const mem = new Uint8Array(ex.memory.buffer, ptr, 1);
        return mem[0] !== 0;
      } finally {
        ex.wasm_free(ptr, 4);
      }
    }

    if (pname === 0x0B98 /* STENCIL_WRITEMASK */ || pname === 0x8CA5 /* STENCIL_BACK_WRITEMASK */) {
      const ptr = ex.wasm_alloc(4);
      try {
        const code = ex.wasm_ctx_get_parameter_v(this._ctxHandle, pname, ptr, 4);
        _checkErr(code, this._instance);
        const mem = new Int32Array(ex.memory.buffer, ptr, 1);
        return mem[0];
      } finally {
        ex.wasm_free(ptr, 4);
      }
    }

    const singleIntParams = [
      0x0B74, // DEPTH_FUNC
      0x0B92, // STENCIL_FUNC
      0x0B93, // STENCIL_VALUE_MASK
      0x0B97, // STENCIL_REF
      0x8800, // STENCIL_BACK_FUNC
      0x8CA4, // STENCIL_BACK_VALUE_MASK
      0x8CA3, // STENCIL_BACK_REF
      0x0B94, // STENCIL_FAIL
      0x0B95, // STENCIL_PASS_DEPTH_FAIL
      0x0B96, // STENCIL_PASS_DEPTH_PASS
      0x8801, // STENCIL_BACK_FAIL
      0x8802, // STENCIL_BACK_PASS_DEPTH_FAIL
      0x8803, // STENCIL_BACK_PASS_DEPTH_PASS
    ];

    if (singleIntParams.includes(pname)) {
      const ptr = ex.wasm_alloc(4);
      try {
        const code = ex.wasm_ctx_get_parameter_v(this._ctxHandle, pname, ptr, 4);
        _checkErr(code, this._instance);
        const mem = new Int32Array(ex.memory.buffer, ptr, 1);
        return mem[0];
      } finally {
        ex.wasm_free(ptr, 4);
      }
    }

    if (pname === 0x8869 /* MAX_VERTEX_ATTRIBS */) {
      return 16;
    }

    throw new Error(`getParameter for ${pname} not implemented`);
  }
  getError() {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_get_error !== 'function') {
      throw new Error('wasm_ctx_get_error not found');
    }
    return ex.wasm_ctx_get_error(this._ctxHandle);
  }

  _setError(error) {
    const ex = this._instance.exports;
    if (ex && typeof ex.wasm_ctx_set_gl_error === 'function') {
      ex.wasm_ctx_set_gl_error(this._ctxHandle, error);
    }
  }

  finish() { this._assertNotDestroyed(); throw new Error('not implemented'); }
  flush() { this._assertNotDestroyed(); throw new Error('not implemented'); }

  isTexture(tex) {
    this._assertNotDestroyed();
    if (!tex || typeof tex !== 'object' || !(tex instanceof WasmWebGLTexture)) return false;
    if (tex._ctx !== this) return false;
    const ex = this._instance.exports;
    return ex.wasm_ctx_is_texture(this._ctxHandle, tex._handle) !== 0;
  }
  isFramebuffer(fb) {
    this._assertNotDestroyed();
    // Framebuffer is currently implemented as a number handle
    if (typeof fb !== 'number') return false;
    const ex = this._instance.exports;
    return ex.wasm_ctx_is_framebuffer(this._ctxHandle, fb) !== 0;
  }
  isProgram(p) {
    this._assertNotDestroyed();
    if (!p || typeof p !== 'object' || !(p instanceof WasmWebGLProgram)) return false;
    if (p._ctx !== this) return false;
    const ex = this._instance.exports;
    return ex.wasm_ctx_is_program(this._ctxHandle, p._handle) !== 0;
  }
  isShader(s) {
    this._assertNotDestroyed();
    if (!s || typeof s !== 'object' || !(s instanceof WasmWebGLShader)) return false;
    if (s._ctx !== this) return false;
    const ex = this._instance.exports;
    return ex.wasm_ctx_is_shader(this._ctxHandle, s._handle) !== 0;
  }
  enable(cap) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_enable !== 'function') {
      throw new Error('wasm_ctx_enable not found');
    }
    const code = ex.wasm_ctx_enable(this._ctxHandle, cap >>> 0);
    _checkErr(code, this._instance);
  }
  disable(cap) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_disable !== 'function') {
      throw new Error('wasm_ctx_disable not found');
    }
    const code = ex.wasm_ctx_disable(this._ctxHandle, cap >>> 0);
    _checkErr(code, this._instance);
  }

  blendFunc(sfactor, dfactor) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_blend_func !== 'function') {
      throw new Error('wasm_ctx_blend_func not found');
    }
    const code = ex.wasm_ctx_blend_func(this._ctxHandle, sfactor >>> 0, dfactor >>> 0);
    _checkErr(code, this._instance);
  }

  blendFuncSeparate(srcRGB, dstRGB, srcAlpha, dstAlpha) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_blend_func_separate !== 'function') {
      throw new Error('wasm_ctx_blend_func_separate not found');
    }
    const code = ex.wasm_ctx_blend_func_separate(
      this._ctxHandle,
      srcRGB >>> 0,
      dstRGB >>> 0,
      srcAlpha >>> 0,
      dstAlpha >>> 0
    );
    _checkErr(code, this._instance);
  }

  blendEquation(mode) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_blend_equation !== 'function') {
      throw new Error('wasm_ctx_blend_equation not found');
    }
    const code = ex.wasm_ctx_blend_equation(this._ctxHandle, mode >>> 0);
    _checkErr(code, this._instance);
  }

  blendEquationSeparate(modeRGB, modeAlpha) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_blend_equation_separate !== 'function') {
      throw new Error('wasm_ctx_blend_equation_separate not found');
    }
    const code = ex.wasm_ctx_blend_equation_separate(
      this._ctxHandle,
      modeRGB >>> 0,
      modeAlpha >>> 0
    );
    _checkErr(code, this._instance);
  }

  blendColor(red, green, blue, alpha) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_blend_color !== 'function') {
      throw new Error('wasm_ctx_blend_color not found');
    }
    const code = ex.wasm_ctx_blend_color(
      this._ctxHandle,
      +red,
      +green,
      +blue,
      +alpha
    );
    _checkErr(code, this._instance);
  }

  isEnabled(cap) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  viewport(x, y, width, height) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_viewport !== 'function') {
      throw new Error('wasm_ctx_viewport not found');
    }
    const code = ex.wasm_ctx_viewport(this._ctxHandle, x >>> 0, y >>> 0, width >>> 0, height >>> 0);
    _checkErr(code, this._instance);
  }
  scissor(x, y, width, height) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_scissor !== 'function') {
      throw new Error('wasm_ctx_scissor not found');
    }
    const code = ex.wasm_ctx_scissor(this._ctxHandle, x | 0, y | 0, width >>> 0, height >>> 0);
    _checkErr(code, this._instance);
  }
  clear(mask) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_clear !== 'function') {
      throw new Error('wasm_ctx_clear not found');
    }
    const code = ex.wasm_ctx_clear(this._ctxHandle, mask >>> 0);
    _checkErr(code, this._instance);
  }
  clearColor(r, g, b, a) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_clear_color !== 'function') {
      throw new Error('wasm_ctx_clear_color not found');
    }
    const code = ex.wasm_ctx_clear_color(this._ctxHandle, +r, +g, +b, +a);
    _checkErr(code, this._instance);
  }
  clearDepth(depth) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  depthFunc(func) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_depth_func !== 'function') {
      throw new Error('wasm_ctx_depth_func not found');
    }
    const code = ex.wasm_ctx_depth_func(this._ctxHandle, func >>> 0);
    _checkErr(code, this._instance);
  }
  depthMask(flag) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_depth_mask !== 'function') {
      throw new Error('wasm_ctx_depth_mask not found');
    }
    const code = ex.wasm_ctx_depth_mask(this._ctxHandle, flag ? 1 : 0);
    _checkErr(code, this._instance);
  }
  colorMask(r, g, b, a) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_color_mask !== 'function') {
      throw new Error('wasm_ctx_color_mask not found');
    }
    const code = ex.wasm_ctx_color_mask(this._ctxHandle, r ? 1 : 0, g ? 1 : 0, b ? 1 : 0, a ? 1 : 0);
    _checkErr(code, this._instance);
  }
  polygonOffset(factor, units) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  sampleCoverage(value, invert) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  stencilFunc(func, ref, mask) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_stencil_func !== 'function') {
      throw new Error('wasm_ctx_stencil_func not found');
    }
    const code = ex.wasm_ctx_stencil_func(this._ctxHandle, func >>> 0, ref | 0, mask >>> 0);
    _checkErr(code, this._instance);
  }
  stencilFuncSeparate(face, func, ref, mask) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_stencil_func_separate !== 'function') {
      throw new Error('wasm_ctx_stencil_func_separate not found');
    }
    const code = ex.wasm_ctx_stencil_func_separate(this._ctxHandle, face >>> 0, func >>> 0, ref | 0, mask >>> 0);
    _checkErr(code, this._instance);
  }
  stencilOp(fail, zfail, zpass) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_stencil_op !== 'function') {
      throw new Error('wasm_ctx_stencil_op not found');
    }
    const code = ex.wasm_ctx_stencil_op(this._ctxHandle, fail >>> 0, zfail >>> 0, zpass >>> 0);
    _checkErr(code, this._instance);
  }
  stencilOpSeparate(face, fail, zfail, zpass) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_stencil_op_separate !== 'function') {
      throw new Error('wasm_ctx_stencil_op_separate not found');
    }
    const code = ex.wasm_ctx_stencil_op_separate(this._ctxHandle, face >>> 0, fail >>> 0, zfail >>> 0, zpass >>> 0);
    _checkErr(code, this._instance);
  }
  stencilMask(mask) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_stencil_mask !== 'function') {
      throw new Error('wasm_ctx_stencil_mask not found');
    }
    const code = ex.wasm_ctx_stencil_mask(this._ctxHandle, mask >>> 0);
    _checkErr(code, this._instance);
  }
  stencilMaskSeparate(face, mask) {
    this._assertNotDestroyed();
    const ex = this._instance.exports;
    if (!ex || typeof ex.wasm_ctx_stencil_mask_separate !== 'function') {
      throw new Error('wasm_ctx_stencil_mask_separate not found');
    }
    const code = ex.wasm_ctx_stencil_mask_separate(this._ctxHandle, face >>> 0, mask >>> 0);
    _checkErr(code, this._instance);
  }
}

/**
 * Thin wrapper for a WebGLTexture handle returned from WASM.
 * Holds a reference to the originating WasmWebGL2RenderingContext and the numeric handle.
 */
// WebGLTexture wrapper moved to `src/webgl2_texture.js`.

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

// ============================================================================
// WAT Testing Support (docs/1.9-wat-testing.md)
// ============================================================================

/**
 * Get the compiled WASM bytes for a shader in a program.
 * 
 * @param {number} ctxHandle - Context handle
 * @param {number} programHandle - Program handle
 * @param {number} shaderType - Shader type (VERTEX_SHADER or FRAGMENT_SHADER)
 * @returns {Uint8Array | null} WASM bytes or null if not available
 */
export function getShaderModule(ctxHandle, programHandle, shaderType) {
  const ctx = WasmWebGL2RenderingContext._contexts.get(ctxHandle);
  if (!ctx) {
    throw new Error('Invalid context handle');
  }

  const ex = ctx._instance.exports;
  if (!ex || typeof ex.wasm_ctx_get_program_wasm_ref !== 'function') {
    throw new Error('wasm_ctx_get_program_wasm_ref not found');
  }

  // Call the WASM function - it returns a packed u64 (BigInt or Number)
  const result = ex.wasm_ctx_get_program_wasm_ref(ctxHandle, programHandle, shaderType);

  // Unpack: low 32 bits = ptr, high 32 bits = len
  let ptr, len;
  if (typeof result === 'bigint') {
    ptr = Number(result & 0xFFFFFFFFn);
    len = Number((result >> 32n) & 0xFFFFFFFFn);
  } else {
    // Fallback for number (may lose precision for very large values)
    ptr = result >>> 0;  // Low 32 bits
    len = Math.floor(result / 0x100000000);  // High 32 bits
  }

  // Check for failure (0, 0)
  if (ptr === 0 || len === 0) {
    return null;
  }

  // Copy bytes from WASM memory into a new Uint8Array
  const mem = new Uint8Array(ex.memory.buffer);
  const bytes = new Uint8Array(len);
  bytes.set(mem.subarray(ptr, ptr + len));

  return bytes;
}

/**
 * Get the WAT (WebAssembly Text) representation for a shader in a program.
 * 
 * @param {number} ctxHandle - Context handle
 * @param {number} programHandle - Program handle
 * @param {number} shaderType - Shader type (VERTEX_SHADER or FRAGMENT_SHADER)
 * @returns {string | null} WAT text or null if not available
 */
export function getShaderWat(ctxHandle, programHandle, shaderType) {
  const ctx = WasmWebGL2RenderingContext._contexts.get(ctxHandle);
  if (!ctx) {
    throw new Error('Invalid context handle');
  }

  const ex = ctx._instance.exports;
  if (!ex || typeof ex.wasm_ctx_get_program_wat_ref !== 'function') {
    throw new Error('wasm_ctx_get_program_wat_ref not found');
  }

  // Call the WASM function - it returns a packed u64 (BigInt or Number)
  const result = ex.wasm_ctx_get_program_wat_ref(ctxHandle, programHandle, shaderType);

  // Unpack: low 32 bits = ptr, high 32 bits = len
  let ptr, len;
  if (typeof result === 'bigint') {
    ptr = Number(result & 0xFFFFFFFFn);
    len = Number((result >> 32n) & 0xFFFFFFFFn);
  } else {
    // Fallback for number (may lose precision for very large values)
    ptr = result >>> 0;  // Low 32 bits
    len = Math.floor(result / 0x100000000);  // High 32 bits
  }

  // Check for failure (0, 0)
  if (ptr === 0 || len === 0) {
    return null;
  }

  // Copy bytes from WASM memory and decode as UTF-8
  const mem = new Uint8Array(ex.memory.buffer);
  const bytes = mem.subarray(ptr, ptr + len);
  const decoder = new TextDecoder('utf-8');
  const watText = decoder.decode(bytes);

  return watText;
}

/**
 * Decompile WASM bytes to GLSL source code.
 * 
 * This uses the WASM-to-GLSL decompiler to convert compiled shader WASM
 * back into readable GLSL-like code.
 * 
 * @param {number} ctxHandle - Context handle
 * @param {number} programHandle - Program handle
 * @param {number} shaderType - Shader type (VERTEX_SHADER or FRAGMENT_SHADER)
 * @returns {string | null} GLSL source code or null if not available
 */
export function getShaderGlsl(ctxHandle, programHandle, shaderType) {
  const ctx = WasmWebGL2RenderingContext._contexts.get(ctxHandle);
  if (!ctx) {
    throw new Error('Invalid context handle');
  }

  // First get the WASM bytes for the shader
  const wasmBytes = getShaderModule(ctxHandle, programHandle, shaderType);
  if (!wasmBytes) {
    return null;
  }

  const ex = ctx._instance.exports;
  if (!ex || typeof ex.wasm_decompile_to_glsl !== 'function') {
    throw new Error('wasm_decompile_to_glsl not found');
  }

  // Allocate memory in WASM for the input bytes
  const wasmBytesLen = wasmBytes.length;
  const wasmBytesPtr = ex.wasm_alloc(wasmBytesLen);
  if (wasmBytesPtr === 0) {
    throw new Error('Failed to allocate memory for WASM bytes');
  }

  try {
    // Copy WASM bytes to linear memory
    const mem = new Uint8Array(ex.memory.buffer);
    mem.set(wasmBytes, wasmBytesPtr);

    // Call the decompiler
    const resultLen = ex.wasm_decompile_to_glsl(wasmBytesPtr, wasmBytesLen);

    if (resultLen === 0) {
      return null;
    }

    // Get the decompiled GLSL
    const glslPtr = ex.wasm_get_decompiled_glsl_ptr();
    const glslLen = ex.wasm_get_decompiled_glsl_len();

    if (glslPtr === 0 || glslLen === 0) {
      return null;
    }

    // Read the GLSL string
    const glslBytes = new Uint8Array(ex.memory.buffer).subarray(glslPtr, glslPtr + glslLen);
    const decoder = new TextDecoder('utf-8');
    return decoder.decode(glslBytes);
  } finally {
    // Free the allocated memory
    ex.wasm_free(wasmBytesPtr);
  }
}

/**
 * Decompile raw WASM bytes to GLSL source code.
 * 
 * This is a lower-level API that takes raw WASM bytes directly.
 * 
 * @param {WasmWebGL2RenderingContext} gl - WebGL2 context
 * @param {Uint8Array} wasmBytes - Raw WASM bytecode to decompile
 * @returns {string | null} GLSL source code or null on error
 */
export function decompileWasmToGlsl(gl, wasmBytes) {
  if (!gl || !gl._instance) {
    throw new Error('Invalid WebGL2 context');
  }

  const ex = gl._instance.exports;
  if (!ex || typeof ex.wasm_decompile_to_glsl !== 'function') {
    throw new Error('wasm_decompile_to_glsl not found');
  }

  // Allocate memory in WASM for the input bytes
  const wasmBytesLen = wasmBytes.length;
  const wasmBytesPtr = ex.wasm_alloc(wasmBytesLen);
  if (wasmBytesPtr === 0) {
    throw new Error('Failed to allocate memory for WASM bytes');
  }

  try {
    // Copy WASM bytes to linear memory
    const mem = new Uint8Array(ex.memory.buffer);
    mem.set(wasmBytes, wasmBytesPtr);

    // Call the decompiler
    const resultLen = ex.wasm_decompile_to_glsl(wasmBytesPtr, wasmBytesLen);

    if (resultLen === 0) {
      return null;
    }

    // Get the decompiled GLSL
    const glslPtr = ex.wasm_get_decompiled_glsl_ptr();
    const glslLen = ex.wasm_get_decompiled_glsl_len();

    if (glslPtr === 0 || glslLen === 0) {
      return null;
    }

    // Read the GLSL string
    const glslBytes = new Uint8Array(ex.memory.buffer).subarray(glslPtr, glslPtr + glslLen);
    const decoder = new TextDecoder('utf-8');
    return decoder.decode(glslBytes);
  } finally {
    // Free the allocated memory
    ex.wasm_free(wasmBytesPtr);
  }
}
