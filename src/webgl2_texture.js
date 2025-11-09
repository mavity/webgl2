/**
 * Thin wrapper for a WebGLTexture handle returned from WASM.
 * @implements {WebGLTexture}
 */
export class WasmWebGLTexture {
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
