/**
 * Thin wrapper for a WebGLShader handle returned from WASM.
 * @implements {WebGLShader}
 */
export class WasmWebGLShader {
  /**
   * @param {import('./webgl2_context.js').WasmWebGL2RenderingContext} ctx
   * @param {number} handle
   */
  constructor(ctx, handle) {
    this._ctx = ctx;
    this._handle = handle;
    this._deleted = false;
  }
}

/**
 * Thin wrapper for a WebGLProgram handle returned from WASM.
 * @implements {WebGLProgram}
 */
export class WasmWebGLProgram {
  /**
   * @param {import('./webgl2_context.js').WasmWebGL2RenderingContext} ctx
   * @param {number} handle
   */
  constructor(ctx, handle) {
    this._ctx = ctx;
    this._handle = handle;
    this._deleted = false;
    /** @type {WebAssembly.Instance | null} */
    this._vsInstance = null;
    /** @type {WebAssembly.Instance | null} */
    this._fsInstance = null;
  }
}

/**
 * Thin wrapper for a WebGLBuffer handle returned from WASM.
 * @implements {WebGLBuffer}
 */
export class WasmWebGLBuffer {
  /**
   * @param {import('./webgl2_context.js').WasmWebGL2RenderingContext} ctx
   * @param {number} handle
   */
  constructor(ctx, handle) {
    this._ctx = ctx;
    this._handle = handle;
    this._deleted = false;
  }
}

/**
 * Thin wrapper for a WebGLRenderbuffer handle returned from WASM.
 * @implements {WebGLRenderbuffer}
 */
export class WasmWebGLRenderbuffer {
  /**
   * @param {import('./webgl2_context.js').WasmWebGL2RenderingContext} ctx
   * @param {number} handle
   */
  constructor(ctx, handle) {
    this._ctx = ctx;
    this._handle = handle;
    this._deleted = false;
  }
}
