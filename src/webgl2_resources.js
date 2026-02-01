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

/**
 * Thin wrapper for a WebGLFramebuffer handle returned from WASM.
 * @implements {WebGLFramebuffer}
 */
export class WasmWebGLFramebuffer {
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
 * Thin wrapper for a WebGLVertexArrayObject handle returned from WASM.
 * @implements {WebGLVertexArrayObject}
 */
export class WasmWebGLVertexArrayObject {
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
 * Thin wrapper for a WebGLQuery handle returned from WASM.
 * @implements {WebGLQuery}
 */
export class WasmWebGLQuery {
  constructor(ctx, handle) {
    this._ctx = ctx;
    this._handle = handle;
    this._deleted = false;
  }
}

/**
 * Thin wrapper for a WebGLSampler handle returned from WASM.
 * @implements {WebGLSampler}
 */
export class WasmWebGLSampler {
  constructor(ctx, handle) {
    this._ctx = ctx;
    this._handle = handle;
    this._deleted = false;
  }
}

/**
 * Thin wrapper for a WebGLSync handle returned from WASM.
 * @implements {WebGLSync}
 */
export class WasmWebGLSync {
  constructor(ctx, handle) {
    this._ctx = ctx;
    this._handle = handle;
    this._deleted = false;
  }
}

/**
 * Thin wrapper for a WebGLTransformFeedback handle returned from WASM.
 * @implements {WebGLTransformFeedback}
 */
export class WasmWebGLTransformFeedback {
  constructor(ctx, handle) {
    this._ctx = ctx;
    this._handle = handle;
    this._deleted = false;
  }
}

/**
 * Thin wrapper for a WebGLUniformLocation handle returned from WASM.
 * @implements {WebGLUniformLocation}
 */
export class WasmWebGLUniformLocation {
  constructor(ctx, handle) {
    this._ctx = ctx;
    this._handle = handle;
  }
}
