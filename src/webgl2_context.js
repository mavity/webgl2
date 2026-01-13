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
  COLOR_BUFFER_BIT = 0x00004000;
  DEPTH_BUFFER_BIT = 0x00000100;
  DEPTH_TEST = 0x0B71;   // <- add this
  STENCIL_BUFFER_BIT = 0x00000400;
  COMPILE_STATUS = 0x8B81;
  LINK_STATUS = 0x8B82;
  DELETE_STATUS = 0x8B80;
  VALIDATE_STATUS = 0x8B83;
  ARRAY_BUFFER = 0x8892;
  ELEMENT_ARRAY_BUFFER = 0x8893;
  STATIC_DRAW = 0x88E4;
  BYTE = 0x1400;
  UNSIGNED_BYTE = 0x1401;
  SHORT = 0x1402;
  UNSIGNED_SHORT = 0x1403;
  INT = 0x1404;
  UNSIGNED_INT = 0x1405;
  FLOAT = 0x1406;
  RGBA = 0x1908;
  VIEWPORT = 0x0BA2;
  COLOR_CLEAR_VALUE = 0x0C22;
  BUFFER_SIZE = 0x8764;
  MAX_VERTEX_ATTRIBS = 0x8869;
  NO_ERROR = 0;
  INVALID_ENUM = 0x0500;
  INVALID_VALUE = 0x0501;
  INVALID_OPERATION = 0x0502;
  OUT_OF_MEMORY = 0x0505;

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

  TEXTURE_2D = 0x0DE1;
  TEXTURE_WRAP_S = 0x2802;
  TEXTURE_WRAP_T = 0x2803;
  TEXTURE_MAG_FILTER = 0x2800;
  TEXTURE_MIN_FILTER = 0x2801;
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
   */
  constructor({ instance, ctxHandle, width, height, debugShaders = false }) {
    this._instance = instance;
    this._ctxHandle = ctxHandle;
    this._destroyed = false;
    /** @type {import('./webgl2_resources.js').WasmWebGLProgram | null} */
    this._currentProgram = null;
    // Explicit booleans for clarity
    this._debugShaders = !!debugShaders;
    this._drawingBufferWidth = width;
    this._drawingBufferHeight = height;

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

  _executeShader(type, attrPtr, uniformPtr, varyingPtr, privatePtr, texturePtr) {
    if (!this._currentProgram) {
      return;
    }
    const shaderInstance = type === this.VERTEX_SHADER ? this._currentProgram._vsInstance : this._currentProgram._fsInstance;
    if (shaderInstance && shaderInstance.exports.main) {
      // @ts-ignore
      shaderInstance.exports.main(type, attrPtr, uniformPtr, varyingPtr, privatePtr, texturePtr);
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

      // Mirror texture data in JS for fast texel fetches by shader imports
      this._textureData = this._textureData || new Map();
      const handle = this._boundTexture || 0;
      const copy = new Uint8Array(mem.slice(ptr, ptr + len));
      this._textureData.set(handle, { width: width >>> 0, height: height >>> 0, data: copy });
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
    // According to the WebGL 2.0 Spec (https://registry.khronos.org/webgl/specs/latest/2.0/#3.7.6),
    // linkProgram sets the LINK_STATUS parameter. If linking fails, no executable is generated.
    // We must check LINK_STATUS before attempting to instantiate the WASM, as getProgramWasm will return null.
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
            console.log(`DEBUG LOG: val=${funcIdx} (0x${(funcIdx >>> 0).toString(16)})`);
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


    program._vsInstance = new WebAssembly.Instance(vsModule, {
      env: {
        memory: this._instance.exports.memory,
        ...vsDebugEnv
      }
    });
    vsInstanceRef.current = program._vsInstance;

    let fsModule;
    fsModule = new WebAssembly.Module(fsWasm);
    const fsInstanceRef = { current: null };
    const fsDebugEnv = createDebugEnv(this.FRAGMENT_SHADER, fsInstanceRef);


    program._fsInstance = new WebAssembly.Instance(fsModule, {
      env: {
        memory: this._instance.exports.memory,
        ...fsDebugEnv
      }
    });
    fsInstanceRef.current = program._fsInstance;
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
  copyBufferSubData(readTarget, writeTarget, readOffset, writeOffset, size) { this._assertNotDestroyed(); throw new Error('not implemented'); }
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
  isBuffer(buffer) { this._assertNotDestroyed(); throw new Error('not implemented'); }

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
  generateMipmap(target) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  copyTexImage2D(target, level, internalformat, x, y, width, height, border) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  copyTexSubImage2D(target, level, xoffset, yoffset, x, y, width, height) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  texImage3D(target, level, internalformat, width, height, depth, border, format, type, pixels) { this._assertNotDestroyed(); throw new Error('not implemented'); }

  checkFramebufferStatus(target) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  blitFramebuffer(srcX0, srcY0, srcX1, srcY1, dstX0, dstY0, dstX1, dstY1, mask, filter) { this._assertNotDestroyed(); throw new Error('not implemented'); }
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
  getActiveUniform(program, index) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  getActiveAttrib(program, index) { this._assertNotDestroyed(); throw new Error('not implemented'); }

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

  isTexture(tex) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  isFramebuffer(fb) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  isProgram(p) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  isShader(s) { this._assertNotDestroyed(); throw new Error('not implemented'); }
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
  depthMask(flag) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  colorMask(r, g, b, a) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  polygonOffset(factor, units) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  sampleCoverage(value, invert) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  stencilFunc(func, ref, mask) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  stencilOp(fail, zfail, zpass) { this._assertNotDestroyed(); throw new Error('not implemented'); }
  stencilMask(mask) { this._assertNotDestroyed(); throw new Error('not implemented'); }
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
