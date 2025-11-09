/*
Unified WebAssembly loader + lightweight WebGL2-like shim for Browser and Node.js

Primary entry: `webGL2(opts)` — returns a compact WebGL2-like rendering context.
The public API intentionally does not accept or expose any WASM/URL/path/Buffer
parameters. WASM instantiation and program wiring are handled internally by
the runtime; callers interact only with the WebGL2-like facade.

Usage (ES module):
  import { webGL2 } from './runners/adapter.js';
  const gl = await webGL2({ width: 1024, height: 1024 });
  gl.writeFloats(gl.OFFSETS.ATTR, [0,1,0]);
  // the runtime wires programs internally; draw calls will work once a
  // program has been installed by the internal test harness or runtime.

Usage (CommonJS / Node):
  const { webGL2 } = require('./runners/adapter');
  const gl = await webGL2({ width: 512 });

Notes:
  - The returned context implements a minimal WebGL2-like subset (textures,
    texImage2D, clear, drawArrays, readPixels) intended for Phase‑0 testing
    and CI. It's a compatibility shim, not a full GPU implementation.
  - Default memory layout is `OFFSETS` (see `gl.OFFSETS`). Host functions
    `texture_sample`/`log` are provided and wired to the context when a WASM
    program is loaded by the internal runtime.
*/

const isNode = typeof process !== 'undefined' && process.versions != null && process.versions.node != null;

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

    // Determine how to instantiate across environments
    let module, instance;
    if (isNode) {
      // Node: WebAssembly.instantiate works with BufferSource
      module = await WebAssembly.compile(buffer);
      // Create a temporary memory if module expects imported memory
      // We'll inspect module imports
      const needsMemory = module.imports && module.imports.some(i => i.module === 'env' && i.name === 'memory');
      if (needsMemory) {
        importObj.env.memory = new WebAssembly.Memory({ initial: 4, maximum: 256 });
      }
      instance = await WebAssembly.instantiate(module, importObj);
    } else {
      // Browser
      module = await WebAssembly.compile(buffer);
      const needsMemory = module.imports && module.imports.some(i => i.module === 'env' && i.name === 'memory');
      if (needsMemory) {
        importObj.env.memory = new WebAssembly.Memory({ initial: 4, maximum: 256 });
      }
      instance = await WebAssembly.instantiate(module, importObj);
    }

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
    this.u8.set(bytes, ptr);
  }

  writeFloats(ptr, floatArray) {
    // floatArray: Float32Array or Array of numbers
    if (!(floatArray instanceof Float32Array)) {
      floatArray = new Float32Array(floatArray);
    }
    // copy into f32 view (byte offset = ptr)
    const offset = ptr / 4;
    this.f32.set(floatArray, offset);
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
// High-level entry point: returns a WebGL2-like context object.
// Usage:
//   const gl = await webGL2({ width: 1024, height: 1024 });
//   await gl.loadWasm('shader.wasm');
//   gl.writeFloats(gl.OFFSETS.ATTR, [ ... ]);
//   await gl.drawArrays();
async function webGL2(opts = {}) {
  const width = opts.width || 256;
  const height = opts.height || 256;
  // if opts.wasm provided, we'll auto-load it before returning

  // internal state
  const state = {
    width,
    height,
    framebuffer: new Uint8ClampedArray(width * height * 4), // RGBA
    _textures: Object.create(null),
    _nextTexId: 1,
    _boundTexture: null,
    _adapter: null,
  };

  // Helper: pack float [0..1] to byte
  function floatToByte(v) {
    const n = Math.round(Math.max(0, Math.min(1, v)) * 255);
    return n;
  }

  // Default async loader for a wasm "program" (vertex+fragment exports).
  // Accepts path (Node), url (browser) or raw bytes/ArrayBuffer/Uint8Array/Buffer.
  // Internal only: the facade auto-loads the WASM when `opts.wasm` is provided
  // to `webGL2(opts)`. This function is intentionally not exported
  // as part of the public WebGL2-like surface.
  async function loadWasm(source, loadOpts = {}) {
    let adapter;
    if (typeof source === 'string') {
      if (isNode) adapter = await Adapter.instantiateFromFile(source, loadOpts);
      else adapter = await Adapter.instantiateFromUrl(source, loadOpts);
    } else {
      adapter = await Adapter.instantiate(source, loadOpts);
    }

    // wire texture sampler to use this context's textures map
    adapter.setTextureSampler((texId, u, v, outPtr, adapterInstance) => {
      const tex = state._textures[texId];
      if (!tex || !tex.pixels) {
        // write transparent black
        const off = outPtr / 4;
        adapterInstance.f32[off + 0] = 0.0;
        adapterInstance.f32[off + 1] = 0.0;
        adapterInstance.f32[off + 2] = 0.0;
        adapterInstance.f32[off + 3] = 1.0;
        return;
      }

      // nearest sampling for now
      const x = Math.floor(u * (tex.width - 1));
      const y = Math.floor(v * (tex.height - 1));
      const idx = (y * tex.width + x) * tex.bytesPerPixel;
      // tex.pixels is expected to be Uint8Array RGBA or Float32Array (0..1)
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
    return adapter;
  }

  function createTexture() {
    const id = state._nextTexId++;
    state._textures[id] = { width: 0, height: 0, pixels: null, bytesPerPixel: 4, type: 'u8' };
    return id;
  }

  function bindTexture(target, texId) {
    state._boundTexture = texId;
  }

  // texImage2D simplified: accepts typed pixel arrays in RGBA order (Uint8Array or Float32Array)
  function texImage2D(target, level, internalformat, width, height, border, format, type, pixels) {
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
      const idx = (py * state.width + px) * 4;
      state.framebuffer[idx + 0] = floatToByte(col[0] ?? 0);
      state.framebuffer[idx + 1] = floatToByte(col[1] ?? 0);
      state.framebuffer[idx + 2] = floatToByte(col[2] ?? 0);
      state.framebuffer[idx + 3] = floatToByte(col[3] ?? 1);
    }
  }

  function readPixels(x, y, w, h, out) {
    // out must be a Uint8ClampedArray with at least w*h*4
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
