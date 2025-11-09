// @ts-check

/*
Top-level npm entrypoint for the package.
This file is a copy of `runners/adapter.js` placed at project root so the
package `main` (index.js) exports the high-level `webGL2` facade from the
project root as requested.

It intentionally mirrors the `runners/adapter.js` implementation. The runtime
expects exactly one WASM file named `webgl2.wasm` placed next to this file
(`index.js`) — loading is opaque and automatic for callers.
*/

const isNode = typeof process !== 'undefined' && process.versions != null && process.versions.node != null;
// Scratch pointer in wasm linear memory for host->wasm transfers.
const WASM_SCRATCH = 0x5000;

class Adapter {
  // Default offsets (bytes) in linear memory. Keep aligned to 4 bytes.
  static OFFSETS = {
    STACK: 0x0000,
    ATTR: 0x1000,      // attributes
    UNIFORMS: 0x2000,  // uniforms
    VARYINGS: 0x3000,  // varyings
    OUT: 0x4000,       // outputs
  };

  constructor(instance, memory, opts = {}) {
    this.instance = instance;
    this.memory = memory;
    this.opts = opts;

    // Typed array views
    this.u8 = new Uint8Array(this.memory.buffer);
    this.f32 = new Float32Array(this.memory.buffer);

    // Host-provided hooks (overridable)
    this.textureSampler = opts.textureSampler || this._defaultTextureSampler.bind(this);
    this.logger = opts.logger || this._defaultLogger.bind(this);
  }

  static async instantiate(bytes, opts = {}) {
    // bytes: Uint8Array or ArrayBuffer or Buffer (Node)
    let buffer;
    if (bytes instanceof ArrayBuffer) buffer = new Uint8Array(bytes);
    else if (bytes.buffer && bytes.byteLength !== undefined) buffer = new Uint8Array(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    else buffer = new Uint8Array(bytes);

    const importObj = { env: {} };

    // Provide host functions
    importObj.env.texture_sample = function(texId, u, v, outPtr) {
      // We'll call back into adapter instance once instantiated; this is a placeholder.
      // The adapter replaces this.importTextureSample after instantiation.
      if (typeof Adapter.__texture_stub === 'function') {
        Adapter.__texture_stub(texId, u, v, outPtr);
      }
    };

    importObj.env.log = function(level, ptr, len) {
      if (typeof Adapter.__log_stub === 'function') {
        Adapter.__log_stub(level, ptr, len);
      }
    };

    importObj.env.now_ms = function() {
      return Date.now();
    };

    // Compile & instantiate from the provided bytes for all environments.
    // Avoid re-entering file-based helpers which can cause recursion.
    let module, instance;
    module = await WebAssembly.compile(buffer);
    const needsMemory = module.imports && module.imports.some(i => i.module === 'env' && i.name === 'memory');
    if (needsMemory) {
      importObj.env.memory = new WebAssembly.Memory({ initial: 4, maximum: 256 });
    }
    instance = await WebAssembly.instantiate(module, importObj);

    // Resolve memory: prefer exported memory if present
    let memory = null;
    if (instance.exports && instance.exports.memory) {
      memory = instance.exports.memory;
    } else if (importObj.env.memory) {
      memory = importObj.env.memory;
    }
    if (!memory) throw new Error('WASM module did not export or import memory');

    const adapter = new Adapter(instance, memory, opts);

    // Replace the static stubs to forward into this adapter instance
    Adapter.__texture_stub = (texId, u, v, outPtr) => adapter._texture_sample(texId, u, v, outPtr);
    Adapter.__log_stub = (level, ptr, len) => adapter._log(level, ptr, len);

    return adapter;
  }

  // Basic memory helpers
  writeBytes(ptr, bytes) {
    // Recreate view from the current memory buffer to avoid using stale
    // typed array views if the memory grew and the underlying ArrayBuffer
    // was replaced.
    const view = new Uint8Array(this.memory.buffer);
    view.set(bytes, ptr);
  }

  writeFloats(ptr, floatArray) {
    // floatArray: Float32Array or Array of numbers
    if (!(floatArray instanceof Float32Array)) {
      floatArray = new Float32Array(floatArray);
    }
    // copy into f32 view (byte offset = ptr)
    const f32view = new Float32Array(this.memory.buffer);
    const offset = ptr / 4;
    f32view.set(floatArray, offset);
  }

  readFloats(ptr, count) {
    const offset = ptr / 4;
    return new Float32Array(this.f32.buffer, ptr, count).slice();
  }

  // Logging helper that reads string from memory
  _log(level, ptr, len) {
    const bytes = this.u8.subarray(ptr, ptr + len);
    try {
      const s = new TextDecoder('utf-8').decode(bytes);
      this.logger(level, s);
    } catch (e) {
      this.logger(level, '<invalid utf8>');
    }
  }

  _defaultLogger(level, msg) {
    const prefix = level === 0 ? 'ERROR' : level === 1 ? 'WARN' : 'INFO';
    if (isNode) console.log(`[wasm ${prefix}] ${msg}`);
    else console.log(`[wasm ${prefix}] ${msg}`);
  }

  // Default texture sampler: writes black to out_ptr
  _texture_sample(texId, u, v, outPtr) {
    // write 4 f32 r,g,b,a into memory at outPtr
    const off = outPtr / 4; // f32 offset
    this.f32[off + 0] = 0.0;
    this.f32[off + 1] = 0.0;
    this.f32[off + 2] = 0.0;
    this.f32[off + 3] = 1.0;
  }

  // Read a 4-component vec from OUT area (returns Float32Array(4))
  readOutVec() {
    return this.readFloats(Adapter.OFFSETS.OUT, 4);
  }

  // Call vertex function by name; writes position to OUT ptr and returns Float32Array
  async callVertex(name = 'vertex_main') {
    const fn = this.instance.exports[name];
    if (typeof fn !== 'function') throw new Error(`export ${name} not found`);
    // Call with (attr_ptr, uniform_ptr, out_ptr)
    const res = fn(Adapter.OFFSETS.ATTR, Adapter.OFFSETS.UNIFORMS, Adapter.OFFSETS.OUT);
    // Some engines return multi-value; we ignore res and read memory
    return this.readOutVec();
  }

  async callFragment(name = 'fragment_main') {
    const fn = this.instance.exports[name];
    if (typeof fn !== 'function') throw new Error(`export ${name} not found`);
    const res = fn(Adapter.OFFSETS.VARYINGS, Adapter.OFFSETS.UNIFORMS, Adapter.OFFSETS.OUT);
    return this.readOutVec();
  }

  // Expose a way to set texture sampler override
  setTextureSampler(fn) {
    this.textureSampler = fn;
  }

  setLogger(fn) {
    this.logger = fn;
  }

  // internal hook invoked by static import stub
  _texture_sample(texId, u, v, outPtr) {
    // allow async samplers? keep sync for now
    try {
      this.textureSampler(texId, u, v, outPtr, this);
    } catch (e) {
      console.warn('textureSampler threw', e);
      // default to black
      this._defaultTextureSampler(texId, u, v, outPtr);
    }
  }

  _defaultTextureSampler(texId, u, v, outPtr) {
    const off = outPtr / 4;
    this.f32[off + 0] = 0.0;
    this.f32[off + 1] = 0.0;
    this.f32[off + 2] = 0.0;
    this.f32[off + 3] = 1.0;
  }

  // Helper to load a wasm file by path (Node only)
  static async instantiateFromFile(path, opts = {}) {
    if (!isNode) throw new Error('instantiateFromFile is only supported in Node.js');
    const fs = require('fs');
    const bytes = fs.readFileSync(path);
    return Adapter.instantiate(bytes, opts);
  }

  // Helper to load from URL (browser)
  static async instantiateFromUrl(url, opts = {}) {
    const resp = await fetch(url);
    const ab = await resp.arrayBuffer();
    return Adapter.instantiate(ab, opts);
  }
}

// Do NOT export Adapter or expose internals; only export the facade below.
if (!isNode) {
  // ensure Adapter isn't leaked to the global in browser
}


/**
 * Create a WebGL2-like context.
 * @returns {WebGL2RenderingContext}
 */
async function webGL2(opts = {}) {
  const width = opts.width || 256;
  const height = opts.height || 256;
  // if opts.wasm provided, we'll auto-load it before returning

  // internal state
  const state = {
    width,
    height,
    // Keep a tiny JS-side framebuffer only as a fallback; primary runtime
    // state (textures/framebuffers) is expected to live in WASM. This
    // array is used only when the WASM runtime doesn't provide the
    // corresponding exports (rare in packaged builds).
    framebuffer: new Uint8ClampedArray(width * height * 4), // RGBA
    _adapter: null,
    // Minimal fallback maps to avoid undefined property accesses when the
    // runtime lacks wasm exports. These are not used when WASM is present.
    _textures: {},
    _framebuffers: {},
    _boundTexture: null,
    _boundFramebuffer: null,
  };

  // Helper: pack float [0..1] to byte
  function floatToByte(v) {
    const n = Math.round(Math.max(0, Math.min(1, v)) * 255);
    return n;
  }

  // Immediately load the single WASM file (opaque to callers).
  // Policy: there is exactly one wasm file, named `webgl2.wasm`, placed next
  // to `index.js`. webGL2() will attempt to synchronously (from the caller
  // perspective) load and instantiate that file. On failure the function
  // throws so callers can detect missing/invalid packaging.
  async function _autoLoadWasm() {
    // helper to wire adapter to this context
    function _wireAdapter(adapter) {
      adapter.setTextureSampler((texId, u, v, outPtr, adapterInstance) => {
        const tex = state._textures[texId];
        if (!tex || !tex.pixels) {
          const off = outPtr / 4;
          adapterInstance.f32[off + 0] = 0.0;
          adapterInstance.f32[off + 1] = 0.0;
          adapterInstance.f32[off + 2] = 0.0;
          adapterInstance.f32[off + 3] = 1.0;
          return;
        }
        const x = Math.floor(u * (tex.width - 1));
        const y = Math.floor(v * (tex.height - 1));
        const idx = (y * tex.width + x) * tex.bytesPerPixel;
        const outOff = outPtr / 4;
        if (tex.type === 'f32') {
          adapterInstance.f32[outOff + 0] = tex.pixels[idx + 0];
          adapterInstance.f32[outOff + 1] = tex.pixels[idx + 1];
          adapterInstance.f32[outOff + 2] = tex.pixels[idx + 2];
          adapterInstance.f32[outOff + 3] = tex.pixels[idx + 3];
        } else {
          adapterInstance.f32[outOff + 0] = tex.pixels[idx + 0] / 255;
          adapterInstance.f32[outOff + 1] = tex.pixels[idx + 1] / 255;
          adapterInstance.f32[outOff + 2] = tex.pixels[idx + 2] / 255;
          adapterInstance.f32[outOff + 3] = tex.pixels[idx + 3] / 255;
        }
      });

      state._adapter = adapter;

      // Initialize higher-level WASM-side runtime state (textures,
      // framebuffers, bindings). Prefer wasm_init_context if exported.
      try {
        const ex = adapter.instance && adapter.instance.exports ? adapter.instance.exports : {};
        if (typeof ex.wasm_init_context === 'function') {
          try {
            ex.wasm_init_context(state.width >>> 0, state.height >>> 0);
          } catch (e) {
            // ignore initialization errors from wasm
          }
        }
      } catch (e) {
        // ignore
      }

      // Initialize wasm-side framebuffer if the tiny test exports exist.
      try {
        const ex = adapter.instance && adapter.instance.exports ? adapter.instance.exports : {};
        if (typeof ex.wasm_fb_init === 'function') {
          try {
            ex.wasm_fb_init(state.width, state.height);
          } catch (e) {
            // ignore init errors
          }
        }
      } catch (e) {
        // ignore
      }
    }

    if (isNode) {
      const path = require('path');
      const fs = require('fs');
      const wasmPath = path.join(__dirname, 'webgl2.wasm');
      if (!fs.existsSync(wasmPath)) throw new Error(`WASM not found at ${wasmPath}`);
      const bytes = fs.readFileSync(wasmPath);
      const adapter = await Adapter.instantiate(bytes);
      _wireAdapter(adapter);
      return adapter;
    } else {
      // Browser: fetch the wasm relative to the current module
      const resp = await fetch('./webgl2.wasm');
      const ab = await resp.arrayBuffer();
      const adapter = await Adapter.instantiate(ab);
      _wireAdapter(adapter);
      return adapter;
    }
  }

  function createTexture() {
    if (!state._adapter) throw new Error('no program loaded');
    const ex = state._adapter.instance && state._adapter.instance.exports ? state._adapter.instance.exports : null;
    if (ex && typeof ex.wasm_create_texture === 'function') {
      return ex.wasm_create_texture() >>> 0;
    }
    throw new Error('wasm runtime does not support texture creation');
  }

  function bindTexture(target, texId) {
    if (!state._adapter) throw new Error('no program loaded');
    const ex = state._adapter.instance && state._adapter.instance.exports ? state._adapter.instance.exports : null;
    if (ex && typeof ex.wasm_bind_texture === 'function') {
      ex.wasm_bind_texture(texId >>> 0);
      return;
    }
    throw new Error('wasm runtime does not support bindTexture');
  }

  // Framebuffer API (minimal): create/bind framebuffer and attach texture.
  function createFramebuffer() {
    if (!state._adapter) throw new Error('no program loaded');
    const ex = state._adapter.instance && state._adapter.instance.exports ? state._adapter.instance.exports : null;
    if (ex && typeof ex.wasm_create_framebuffer === 'function') {
      return ex.wasm_create_framebuffer() >>> 0;
    }
    throw new Error('wasm runtime does not support framebuffer creation');
  }

  function bindFramebuffer(target, fb) {
    // target ignored in this minimal implementation
    if (!state._adapter) {
      state._boundFramebuffer = fb || null;
      return;
    }
    const ex = state._adapter.instance && state._adapter.instance.exports ? state._adapter.instance.exports : null;
    if (ex && typeof ex.wasm_bind_framebuffer === 'function') {
      try {
        ex.wasm_bind_framebuffer(fb >>> 0);
      } catch (e) {
        state._boundFramebuffer = fb || null;
      }
      state._boundFramebuffer = fb || null;
      return;
    }
    state._boundFramebuffer = fb || null;
  }

  function framebufferTexture2D(target, attachment, textarget, texture, level) {
    // Minimal: only support COLOR_ATTACHMENT0 and textarget ignored
    if (!state._adapter) {
      const fb = state._framebuffers[state._boundFramebuffer];
      if (!fb) throw new Error('no framebuffer bound');
      fb.colorAttachment = texture;
      return;
    }
    const ex = state._adapter.instance && state._adapter.instance.exports ? state._adapter.instance.exports : null;
    if (ex && typeof ex.wasm_framebuffer_texture2d === 'function') {
      ex.wasm_framebuffer_texture2d(state._boundFramebuffer >>> 0, texture >>> 0);
      return;
    }
    const fb = state._framebuffers[state._boundFramebuffer];
    if (!fb) throw new Error('no framebuffer bound');
    fb.colorAttachment = texture;
  }

  // texImage2D simplified: accepts typed pixel arrays in RGBA order (Uint8Array or Float32Array)
  function texImage2D(target, level, internalformat, width, height, border, format, type, pixels) {
    const ex = state._adapter && state._adapter.instance && state._adapter.instance.exports ? state._adapter.instance.exports : null;
    // If wasm supports tex image upload, write pixels into wasm memory and call it.
    // WASM expects to use the currently bound texture if passed u32::MAX (0xFFFFFFFF)
    if (ex && typeof ex.wasm_tex_image_2d === 'function') {
      // normalize pixels to Uint8Array
      let bytes;
      if (!pixels) bytes = new Uint8Array(width * height * 4);
      else if (pixels instanceof Uint8Array) bytes = pixels;
      else bytes = new Uint8Array(pixels);
      // copy into wasm memory at scratch
      const ptr = WASM_SCRATCH;
      state._adapter.writeBytes(ptr, bytes);
      // use 0xFFFFFFFF as "use bound texture" sentinel
      ex.wasm_tex_image_2d(0xFFFFFFFF >>> 0, width >>> 0, height >>> 0, ptr >>> 0);
      return;
    }
    // fallback to JS-side texture storage (only used if WASM runtime lacks
    // the high-level texture upload export)
    const texId = state._boundTexture;
    if (!texId) throw new Error('no bound texture');
    const tex = state._textures[texId];
    tex.width = width;
    tex.height = height;
    if (!pixels) {
      tex.pixels = new Uint8Array(width * height * 4);
      tex.type = 'u8';
      tex.bytesPerPixel = 4;
    } else if (pixels instanceof Float32Array) {
      tex.pixels = pixels;
      tex.type = 'f32';
      tex.bytesPerPixel = 4; // floats per channel
    } else {
      // Node Buffer or Uint8Array
      tex.pixels = new Uint8Array(pixels);
      tex.type = 'u8';
      tex.bytesPerPixel = 4;
    }
  }

  function clearColor(r, g, b, a) {
    state._clearColor = [r, g, b, a];
  }

  function clear(mask) {
    const c = state._clearColor || [0, 0, 0, 0];
    for (let i = 0; i < state.framebuffer.length; i += 4) {
      state.framebuffer[i + 0] = floatToByte(c[0]);
      state.framebuffer[i + 1] = floatToByte(c[1]);
      state.framebuffer[i + 2] = floatToByte(c[2]);
      state.framebuffer[i + 3] = floatToByte(c[3]);
    }
  }

  // Minimal drawArrays implementation: calls vertex entry for 'count' vertices.
  // For Phase 0 this will rasterize by emitting one pixel per vertex at the
  // transformed position returned in OUT (vec4) and the color in OUT as well
  // (the shader can place color into the OUT area). This is intentionally
  // simplistic — it's a compatibility shim for tests/CI rather than a full
  // rasterizer.
  async function drawArrays(mode = 0 /* TRIANGLES/ignored */, first = 0, count = 1) {
  if (!state._adapter) throw new Error('no program loaded; runtime not initialized');
  const adapter = state._adapter;
  const ex = adapter.instance && adapter.instance.exports ? adapter.instance.exports : null;

    for (let i = 0; i < count; i++) {
      // In many simple tests we'll place attributes directly into OFFSETS.ATTR
      // before calling drawArrays. The adapter.callVertex reads from that area.
      const pos = await adapter.callVertex(); // Float32Array(4)
      const col = await adapter.readOutVec(); // optionally shader writes color here

      // Map NDC [-1,1] -> pixel coords
      const nx = pos[0];
      const ny = pos[1];
      const px = Math.floor((nx * 0.5 + 0.5) * (state.width - 1));
      const py = Math.floor((1.0 - (ny * 0.5 + 0.5)) * (state.height - 1));

      if (px < 0 || px >= state.width || py < 0 || py >= state.height) continue;

      // Prefer to write pixels into the WASM-side framebuffer if supported.
      if (ex && typeof ex.wasm_set_pixel === 'function') {
        const r = floatToByte(col[0] ?? 0);
        const g = floatToByte(col[1] ?? 0);
        const b = floatToByte(col[2] ?? 0);
        const a = floatToByte(col[3] ?? 1);
        try {
          ex.wasm_set_pixel(px >>> 0, py >>> 0, r >>> 0, g >>> 0, b >>> 0, a >>> 0);
          continue;
        } catch (e) {
          // fall back to JS framebuffer if wasm call fails
        }
      }

      // JS fallback: write into the local framebuffer
      const idx = (py * state.width + px) * 4;
      state.framebuffer[idx + 0] = floatToByte(col[0] ?? 0);
      state.framebuffer[idx + 1] = floatToByte(col[1] ?? 0);
      state.framebuffer[idx + 2] = floatToByte(col[2] ?? 0);
      state.framebuffer[idx + 3] = floatToByte(col[3] ?? 1);
    }
  }

  function readPixels(x, y, w, h, out) {
    // out must be a Uint8ClampedArray with at least w*h*4
    const ex = state._adapter && state._adapter.instance && state._adapter.instance.exports ? state._adapter.instance.exports : null;
    if (ex && typeof ex.wasm_read_pixels === 'function') {
      // allocate output in scratch and ask wasm to fill it
      const outPtr = WASM_SCRATCH + 0x1000;
      try {
        ex.wasm_read_pixels(x >>> 0, y >>> 0, w >>> 0, h >>> 0, outPtr >>> 0);
        try {
          // Re-create a fresh view into the current memory buffer and copy
          // bytes one-by-one into the caller-provided buffer. This avoids
          // issues with detached ArrayBuffers in some runtimes when views
          // are reallocated during memory growth.
          const memView = new Uint8Array(state._adapter.memory.buffer);
          const len = w * h * 4;
          for (let i = 0; i < len; i++) {
            out[i] = memView[outPtr + i] || 0;
          }
          return;
        } catch (err) {
          // Some runtimes may replace the underlying ArrayBuffer (memory.grow)
          // which can lead to a detached buffer when views are reused. Fall
          // back to a safe per-pixel read using wasm_get_pixel when possible.
          if (typeof ex.wasm_get_pixel === 'function') {
            let dstOff = 0;
            for (let row = 0; row < h; row++) {
              const sy = y + row;
              for (let col = 0; col < w; col++) {
                const sx = x + col;
                const packed = ex.wasm_get_pixel(sx >>> 0, sy >>> 0) >>> 0;
                out[dstOff + 0] = (packed >>> 24) & 0xff;
                out[dstOff + 1] = (packed >>> 16) & 0xff;
                out[dstOff + 2] = (packed >>> 8) & 0xff;
                out[dstOff + 3] = packed & 0xff;
                dstOff += 4;
              }
            }
            return;
          }
          // otherwise fall through to other fallback paths below
        }
      } catch (e) {
        // fall through to other paths
      }
    }
    // If a framebuffer is bound and it has a color attachment that's a
    // texture, read from the texture pixels.
    if (state._boundFramebuffer) {
      const fb = state._framebuffers && state._framebuffers[state._boundFramebuffer];
      if (fb && fb.colorAttachment) {
        const tex = state._textures[fb.colorAttachment];
        if (tex && tex.pixels) {
          // read from texture storage
          let dstOff = 0;
          for (let row = 0; row < h; row++) {
            const sy = y + row;
            for (let col = 0; col < w; col++) {
              const sx = x + col;
              const idx = (sy * tex.width + sx) * tex.bytesPerPixel;
              if (tex.type === 'f32') {
                out[dstOff + 0] = Math.round((tex.pixels[idx + 0] || 0) * 255);
                out[dstOff + 1] = Math.round((tex.pixels[idx + 1] || 0) * 255);
                out[dstOff + 2] = Math.round((tex.pixels[idx + 2] || 0) * 255);
                out[dstOff + 3] = Math.round((tex.pixels[idx + 3] || 1) * 255);
              } else {
                out[dstOff + 0] = tex.pixels[idx + 0] || 0;
                out[dstOff + 1] = tex.pixels[idx + 1] || 0;
                out[dstOff + 2] = tex.pixels[idx + 2] || 0;
                out[dstOff + 3] = tex.pixels[idx + 3] || 0;
              }
              dstOff += 4;
            }
          }
          return;
        }
      }
    }

    if (ex && typeof ex.wasm_get_pixel === 'function') {
      // Read each pixel via wasm_get_pixel which returns packed RGBA in a u32
      let dstOff = 0;
      for (let row = 0; row < h; row++) {
        const sy = y + row;
        for (let col = 0; col < w; col++) {
          const sx = x + col;
          try {
            const packed = ex.wasm_get_pixel(sx >>> 0, sy >>> 0) >>> 0; // ensure unsigned
            const r = (packed >>> 24) & 0xff;
            const g = (packed >>> 16) & 0xff;
            const b = (packed >>> 8) & 0xff;
            const a = packed & 0xff;
            out[dstOff + 0] = r;
            out[dstOff + 1] = g;
            out[dstOff + 2] = b;
            out[dstOff + 3] = a;
          } catch (e) {
            // on error, write transparent black
            out[dstOff + 0] = 0;
            out[dstOff + 1] = 0;
            out[dstOff + 2] = 0;
            out[dstOff + 3] = 0;
          }
          dstOff += 4;
        }
      }
      return;
    }

    const src = state.framebuffer;
    const rowBytes = state.width * 4;
    for (let row = 0; row < h; row++) {
      const sy = y + row;
      const dstRowOff = row * w * 4;
      const srcOff = sy * rowBytes + x * 4;
      out.set(src.subarray(srcOff, srcOff + w * 4), dstRowOff);
    }
  }

  // Expose a compact WebGL2-like surface (minimal subset)
  const gl = {
    // basic info
    drawingBufferWidth: state.width,
    drawingBufferHeight: state.height,
    OFFSETS: Adapter.OFFSETS,

    // resource methods
    createTexture,
    bindTexture,
    texImage2D,
    createFramebuffer,
    bindFramebuffer,
    framebufferTexture2D,

    // simple drawing API
    clearColor,
    clear,
  drawArrays,

    // helpers
    writeBytes(ptr, bytes) { if (!state._adapter) throw new Error('no program loaded'); state._adapter.writeBytes(ptr, bytes); },
    writeFloats(ptr, floats) { if (!state._adapter) throw new Error('no program loaded'); state._adapter.writeFloats(ptr, floats); },
    readFloats(ptr, count) { if (!state._adapter) throw new Error('no program loaded'); return state._adapter.readFloats(ptr, count); },
    readPixels,

    // advanced accessors (hidden internals are intentionally omitted from the
    // public facade — keep the API minimal and WebGL2-like)
  };

  // The public facade does not accept any WASM/source options. WASM
  // instantiation and program wiring are performed by the internal runtime
  // or test harness and are intentionally not exposed here.

  // Opaque automatic WASM load: instantiate the single wasm next to index.js
  // This is unconditional; failure to load will reject so callers notice
  // missing/invalid packaging immediately.
  await _autoLoadWasm();

  return gl;
}

// Export only the high-level `webGL2` facade. Internals (Adapter, memory,
// low-level instance) are intentionally not exported to keep the public API
// minimal and stable.
if (isNode) {
  module.exports = { webGL2 };
} else {
  window.webGL2 = webGL2;
}

// CLI demo: when run as `node index.js` attempt to load the built wasm and
// call a simple exported function (hello/greet/vertex_main/main). This is a
// best-effort demo — it will print available exports if no simple entrypoint
// is present.
if (isNode && require.main === module) {
  (async () => {
    try {
      console.log('Demo: initializing webGL2 facade...');
      const gl = await webGL2({ width: 16, height: 16 });
      console.log('Demo: gl ready — drawingBuffer', gl.drawingBufferWidth, 'x', gl.drawingBufferHeight);

  // CornflowerBlue RGB ~= (100,149,237)
  // We'll upload a 1x1 texture with the color, attach it to a framebuffer
  // and read back via readPixels using only WebGL-like APIs.
  const tex = gl.createTexture();
  gl.bindTexture(0, tex);
  const pixel = new Uint8Array([100, 149, 237, 255]);
  gl.texImage2D(0, 0, 0, 1, 1, 0, 0, 0, pixel);

  const fb = gl.createFramebuffer();
  gl.bindFramebuffer(0, fb);
  gl.framebufferTexture2D(0, 0, 0, tex, 0);

  const out = new Uint8ClampedArray(4);
  gl.readPixels(0, 0, 1, 1, out);
  console.log(`Demo: pixel(0,0) -> r=${out[0]}, g=${out[1]}, b=${out[2]}, a=${out[3]}`);

      process.exit(0);
    } catch (e) {
      console.error('Demo: failed to run webGL2 demo:', e && e.message ? e.message : e);
      process.exit(1);
    }
  })();
}
