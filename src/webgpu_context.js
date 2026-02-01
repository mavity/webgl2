// @ts-check

/**
 * @typedef {Int8Array
 * | Uint8Array | Uint8ClampedArray
 * | Int16Array | Uint16Array
 * | Int32Array | Uint32Array
 * | Float32Array
 * | Float64Array
 * | BigInt64Array | BigUint64Array
 * } TypedArray
 */

/**
 * @typedef {{
 *  powerPreference?: 'low-power' | 'high-performance'
 * }} RequestAdapterOptions
 */

/**
 * @typedef {{
 *  size: number,
 *  usage: number,
 *  mappedAtCreation?: boolean
 * }} GPUBufferDescriptor
 */

/**
 * @typedef {{
 *  layout?: 'auto' | { layoutHandle: any },
 *  vertex: RenderPipelineShaderDescriptor,
 *  fragment: RenderPipelineShaderDescriptor
 * }} RenderPipelineDescriptor
 */

/**
 * @typedef {{
 *  entryPoint: any,
 *  module: { moduleHandle: any },
 *  buffers?: {
 *    arrayStride: number,
 *    stepMode: string,
 *    attributes: { format: string, offset: number, shaderLocation: number }[]
 *  }[]
 * }} RenderPipelineShaderDescriptor
 */

/**
 * WebGPU API implementation for WebAssembly
 * 
 * This module provides a complete WebGPU API surface that runs entirely in
 * WebAssembly/Rust, enabling deterministic execution, advanced debugging,
 * and software rasterization of WebGPU workloads.
 */

export const GPUBufferUsage = /** @type {const} */({
  MAP_READ: 0x0001,
  MAP_WRITE: 0x0002,
  COPY_SRC: 0x0004,
  COPY_DST: 0x0008,
  INDEX: 0x0010,
  VERTEX: 0x0020,
  UNIFORM: 0x0040,
  STORAGE: 0x0080,
  INDIRECT: 0x0100,
  QUERY_RESOLVE: 0x0200,
});

export const GPUMapMode = /** @type {const} */({
  READ: 0x0001,
  WRITE: 0x0002,
});

export const GPUTextureUsage = /** @type {const} */({
  COPY_SRC: 0x01,
  COPY_DST: 0x02,
  TEXTURE_BINDING: 0x04,
  STORAGE_BINDING: 0x08,
  RENDER_ATTACHMENT: 0x10,
});

export const GPUShaderStage = /** @type {const} */({
  VERTEX: 0x1,
  FRAGMENT: 0x2,
  COMPUTE: 0x4,
});

// Polyfill globals if missing (e.g. in Node.js)
if (typeof globalThis !== 'undefined') {
  if (!globalThis.GPUBufferUsage) globalThis.GPUBufferUsage = GPUBufferUsage;
  if (!globalThis.GPUMapMode) globalThis.GPUMapMode = GPUMapMode;
  if (!globalThis.GPUTextureUsage) globalThis.GPUTextureUsage = GPUTextureUsage;
  if (!globalThis.GPUShaderStage) globalThis.GPUShaderStage = GPUShaderStage;
}

const activeDevices = new Set();

export class GPUUncapturedErrorEvent extends Event {
  constructor(type, eventInitDict) {
    super(type, eventInitDict);
    this.error = eventInitDict.error;
  }
}

const TEXTURE_FORMAT_MAP = /** @type {const} */({
  'r8unorm': 0,
  'r8snorm': 1,
  'r8uint': 2,
  'r8sint': 3,
  'r16float': 12,
  'rgba8unorm': 17,
  'rgba8unorm-srgb': 18,
  'bgra8unorm': 19,
  'bgra8unorm-srgb': 20,
  'rgba16float': 24,
  'r32float': 35,
  'depth32float': 38,
  'depth24plus': 39,
  'depth24plus-stencil8': 40,
});

// Map common features
const FEATURE_MAPPING = /** @type {const} */({
  'depth-clip-control': 0n,
  'depth32float-stencil8': 1n,
  'texture-compression-bc': 2n,
  'texture-compression-etc2': 3n,
  'texture-compression-astc': 4n,
  'indirect-first-instance': 5n,
  'shader-f16': 6n,
  'rg11b10ufloat-renderable': 7n,
  'bgra8unorm-storage': 8n,
  'float32-filterable': 9n,
  'float32-blendable': 10n,
  'clip-distances': 11n,
  'dual-source-blending': 12n,
});

const LIMIT_NAMES = /** @type {const} */([
  "maxTextureDimension1D", "maxTextureDimension2D", "maxTextureDimension3D", "maxTextureArrayLayers",
  "maxBindGroups", "maxBindGroupsPlusVertexBuffers", "maxBindingsPerBindGroup",
  "maxDynamicUniformBuffersPerPipelineLayout", "maxDynamicStorageBuffersPerPipelineLayout",
  "maxSampledTexturesPerShaderStage", "maxSamplersPerShaderStage", "maxStorageBuffersPerShaderStage",
  "maxStorageTexturesPerShaderStage", "maxUniformBuffersPerShaderStage", "maxUniformBufferBindingSize",
  "maxStorageBufferBindingSize", "maxVertexBuffers", "maxVertexAttributes", "maxVertexBufferArrayStride",
  "maxImmediateSize", "minUniformBufferOffsetAlignment", "minStorageBufferOffsetAlignment",
  "padding", "maxInterStageShaderVariables", "maxColorAttachments",
  "maxColorAttachmentBytesPerSample", "maxComputeWorkgroupStorageSize", "maxComputeInvocationsPerWorkgroup",
  "maxComputeWorkgroupSizeX", "maxComputeWorkgroupSizeY", "maxComputeWorkgroupSizeZ",
  "maxComputeWorkgroupsPerDimension"
]);

/**
 * Wrapper around a WebAssembly-backed WebGPU implementation.
 */
export class GPU {

  static dispatchUncapturedError(msg) {
    // In Node.js, Event might not be available globally or might behave differently.
    // But we are using 'node:test' which implies Node environment.
    // If Event is not defined, we might need a polyfill or just call the handler directly.

    const error = new GPUError(msg);

    for (const device of activeDevices) {
      if (typeof device.onuncapturederror === 'function') {
        device.onuncapturederror({ error });
      }
      // Also dispatch as event if supported
      if (typeof Event !== 'undefined') {
        const event = new GPUUncapturedErrorEvent('uncapturederror', { error });
        device.dispatchEvent(event);
      }
    }
  }

  /**
   * @param {*} wasmModule - WebAssembly module exports implementing WebGPU.
   * @param {WebAssembly.Memory} wasmMemory - WebAssembly linear memory.
   */
  constructor(wasmModule, wasmMemory) {
    this.wasm = wasmModule;
    this.memory = wasmMemory;
  }

  /**
   * @returns {string}
   */
  getPreferredCanvasFormat() {
    const format = this.wasm.wasm_webgpu_get_preferred_canvas_format();
    for (const k in TEXTURE_FORMAT_MAP) {
      if (TEXTURE_FORMAT_MAP[/** @type {keyof TEXTURE_FORMAT_MAP} */(k)] === format) return k;
    }
    return 'rgba8unorm';
  }


  /**
   * Request a GPUAdapter
   * @param {RequestAdapterOptions} options - Adapter request options
   * @returns {Promise<GPUAdapter | null>}
   */
  async requestAdapter(options = {}) {
    // Create a WebGPU context
    const ctxHandle = this.wasm.wasm_webgpu_create_context();
    if (!ctxHandle)
      return null;

    let powerPreference = 0; // None
    if (options.powerPreference === 'low-power') {
      powerPreference = 1;
    } else if (options.powerPreference === 'high-performance') {
      powerPreference = 2;
    }

    const adapterHandle = this.wasm.wasm_webgpu_request_adapter(ctxHandle, powerPreference);
    if (adapterHandle === 0) {
      this.wasm.wasm_webgpu_destroy_context(ctxHandle);
      return null;
    }

    return new GPUAdapter(this.wasm, this.memory, ctxHandle, adapterHandle);
  }
}

export class GPUAdapter {

  /**
   * @param {*} wasmModule
   * @param {WebAssembly.Memory} wasmMemory
   * @param {number} ctxHandle
   * @param {number} adapterHandle
   */
  constructor(wasmModule, wasmMemory, ctxHandle, adapterHandle) {
    this.wasm = wasmModule;
    this.memory = wasmMemory;
    this.ctxHandle = ctxHandle;
    this.adapterHandle = adapterHandle;

    /** @type {Set<string>} */
    this.features = new Set();
    const featureBits = this.wasm.wasm_webgpu_get_adapter_features(this.ctxHandle, this.adapterHandle);
    for (const [name, bit] of Object.entries(FEATURE_MAPPING)) {
      if (featureBits & (1n << bit)) {
        this.features.add(name);
      }
    }

    /** @type {Record<string, number>} */
    this.limits = {};
    const limitsSize = 36 * 4; // 34 u32s + 1 u64
    const limitsPtr = this.wasm.wasm_alloc(limitsSize);
    this.wasm.wasm_webgpu_get_adapter_limits(this.ctxHandle, this.adapterHandle, limitsPtr);
    const limitsView = new DataView(this.memory.buffer, limitsPtr, limitsSize);

    for (let i = 0; i < LIMIT_NAMES.length; i++) {
      if (LIMIT_NAMES[i] !== "padding") {
        this.limits[LIMIT_NAMES[i]] = limitsView.getUint32(i * 4, true);
      }
    }
    this.limits.maxBufferSize = Number(limitsView.getBigUint64(34 * 4, true));

    this.wasm.wasm_free(limitsPtr, limitsSize);
  }

  /**
   * Request a GPUDevice
   * @param {Object} descriptor - Device descriptor
   * @returns {Promise<GPUDevice | null>}
   */
  async requestDevice(descriptor = {}) {
    const deviceHandle = this.wasm.wasm_webgpu_request_device(this.ctxHandle, this.adapterHandle);
    if (deviceHandle === 0) {
      return null;
    }
    return new GPUDevice(this.wasm, this.memory, this.ctxHandle, deviceHandle);
  }
}

export class GPUDevice extends (typeof EventTarget !== 'undefined' ? EventTarget : Object) {
  /**
   * @param {*} wasmModule
   * @param {WebAssembly.Memory} wasmMemory
   * @param {number} ctxHandle
   * @param {number} deviceHandle
   */
  constructor(wasmModule, wasmMemory, ctxHandle, deviceHandle) {
    super();
    this.wasm = wasmModule;
    this.memory = wasmMemory;
    this.ctxHandle = ctxHandle;
    this.deviceHandle = deviceHandle;
    this.queue = new GPUQueue(wasmModule, wasmMemory, ctxHandle, deviceHandle);
    this._destroyed = false;
    activeDevices.add(this);
  }

  pushErrorScope(filter) {
    let filterCode = 0;
    if (filter === 'validation') filterCode = 0;
    else if (filter === 'out-of-memory') filterCode = 1;
    else if (filter === 'internal') filterCode = 2;

    this.wasm.wasm_webgpu_push_error_scope(filterCode);
  }

  async popErrorScope() {
    const hasError = this.wasm.wasm_webgpu_pop_error_scope();
    if (hasError) {
      const ptr = this.wasm.wasm_get_webgpu_error_msg_ptr();
      const msg = readString(this.memory, ptr);
      const filter = this.wasm.wasm_get_webgpu_error_filter();

      if (filter === 0) return new GPUValidationError(msg);
      if (filter === 1) return new GPUOutOfMemoryError(msg);
      if (filter === 2) return new GPUInternalError(msg);

      return new GPUError(msg);
    }
    return null;
  }

  /**
   * Create a buffer
   * @param {GPUBufferDescriptor} descriptor - Buffer descriptor
   * @returns {GPUBuffer}
   */
  createBuffer(descriptor) {
    const bufferHandle = this.wasm.wasm_webgpu_create_buffer(
      this.ctxHandle,
      this.deviceHandle,
      BigInt(descriptor.size),
      descriptor.usage,
      descriptor.mappedAtCreation || false
    );
    if (bufferHandle === 0) {
      // Error already captured by Rust (scope or uncaptured event)
    }
    return new GPUBuffer(this.wasm, this.memory, this.ctxHandle, this.deviceHandle, bufferHandle, descriptor.size);
  }

  /**
   * Create a texture
   * @param {{
   *  size: { width: number, height: number, depthOrArrayLayers?: number },
   *  mipLevelCount?: number,
   *  sampleCount?: number,
   *  usage?: any
   * }} descriptor - Texture descriptor
   * @returns {GPUTexture}
   */
  createTexture(descriptor) {
    let width, height, depthOrArrayLayers;
    if (Array.isArray(descriptor.size)) {
      width = descriptor.size[0];
      height = descriptor.size[1] || 1;
      depthOrArrayLayers = descriptor.size[2] || 1;
    } else {
      width = descriptor.size.width;
      height = descriptor.size.height || 1;
      depthOrArrayLayers = descriptor.size.depthOrArrayLayers || 1;
    }

    const textureHandle = this.wasm.wasm_webgpu_create_texture(
      this.ctxHandle,
      this.deviceHandle,
      width,
      height,
      depthOrArrayLayers,
      descriptor.mipLevelCount || 1,
      descriptor.sampleCount || 1,
      1, // dimension: 2D (1)
      TEXTURE_FORMAT_MAP[/** @type {keyof TEXTURE_FORMAT_MAP} */(descriptor.format)] || 17, // default to rgba8unorm
      descriptor.usage
    );
    if (textureHandle === 0) {
      // Error already captured by Rust
    }

    // Create a normalized descriptor for the GPUTexture
    const normalizedDescriptor = Object.assign({}, descriptor, {
      size: { width, height, depthOrArrayLayers }
    });

    return new GPUTexture(this.wasm, this.memory, this.ctxHandle, this.deviceHandle, textureHandle, normalizedDescriptor);
  }

  /**
   * Create a shader module
   * @param {{ code: string }} descriptor - Shader module descriptor
   * @returns {GPUShaderModule}
   */
  createShaderModule(descriptor) {
    const code = descriptor.code;
    const encoder = new TextEncoder();
    const codeBytes = encoder.encode(code);

    const ptr = this.wasm.wasm_alloc(codeBytes.length);
    const heapU8 = new Uint8Array(this.memory.buffer, ptr, codeBytes.length);
    heapU8.set(codeBytes);

    const moduleHandle = this.wasm.wasm_webgpu_create_shader_module(
      this.ctxHandle,
      this.deviceHandle,
      ptr,
      codeBytes.length
    );

    this.wasm.wasm_free(ptr, codeBytes.length);

    if (moduleHandle === 0) {
      // Error already captured by Rust
    }

    return new GPUShaderModule(this.wasm, this.memory, this.ctxHandle, moduleHandle);
  }

  createPipelineLayout(descriptor) {
    const layouts = descriptor.bindGroupLayouts.map(l => l.layoutHandle);
    const ptr = this.wasm.wasm_alloc(layouts.length * 4);
    new Uint32Array(this.memory.buffer, ptr, layouts.length).set(new Uint32Array(layouts));

    const handle = this.wasm.wasm_webgpu_create_pipeline_layout(
      this.ctxHandle,
      this.deviceHandle,
      ptr,
      layouts.length
    );

    this.wasm.wasm_free(ptr, layouts.length * 4);

    if (handle === 0) {
      // throw new Error("Failed to create pipeline layout");
    }

    return new GPUPipelineLayout(this.wasm, this.memory, this.ctxHandle, handle);
  }

  /**
   * Create a render pipeline
   * @param {RenderPipelineDescriptor} descriptor - Render pipeline descriptor
   * @returns {GPURenderPipeline}
   */
  createRenderPipeline(descriptor) {
    const vertexEntry = descriptor.vertex.entryPoint;
    const fragmentEntry = descriptor.fragment.entryPoint;

    const encoder = new TextEncoder();
    const vBytes = encoder.encode(vertexEntry);
    const fBytes = encoder.encode(fragmentEntry);

    const vPtr = this.wasm.wasm_alloc(vBytes.length);
    new Uint8Array(this.memory.buffer, vPtr, vBytes.length).set(vBytes);

    const fPtr = this.wasm.wasm_alloc(fBytes.length);
    new Uint8Array(this.memory.buffer, fPtr, fBytes.length).set(fBytes);

    // Encode vertex buffers
    // Format: [count, stride, stepMode, attrCount, format, offset, location, ...]
    const layoutData = [];
    const buffers = descriptor.vertex.buffers || [];
    layoutData.push(buffers.length);

    for (const buffer of buffers) {
      layoutData.push(buffer.arrayStride);
      layoutData.push(buffer.stepMode === 'instance' ? 1 : 0);
      layoutData.push(buffer.attributes.length);
      for (const attr of buffer.attributes) {
        // Map format string to enum if needed, or assume simple mapping for now
        // For this prototype, we'll assume the Rust side handles string parsing or we pass simple IDs
        // Let's pass format as a simple ID for now: 
        // float32x3 = 0, float32x2 = 1, etc.
        // Actually, let's just pass the raw values and let Rust figure it out or use a simplified mapping
        let formatId = 0;
        if (attr.format === 'float32x3') formatId = 1;
        else if (attr.format === 'float32x2') formatId = 2;
        else if (attr.format === 'float32x4') formatId = 3;

        layoutData.push(formatId);
        layoutData.push(attr.offset);
        layoutData.push(attr.shaderLocation);
      }
    }

    const lPtr = this.wasm.wasm_alloc(layoutData.length * 4);
    new Uint32Array(this.memory.buffer, lPtr, layoutData.length).set(layoutData);

    let layoutHandle = 0;
    if (descriptor.layout && descriptor.layout !== 'auto') {
      layoutHandle = descriptor.layout.layoutHandle;
    }

    const primitiveTopology = {
      'point-list': 1,
      'line-list': 2,
      'line-strip': 3,
      'triangle-list': 4,
      'triangle-strip': 5,
    }[descriptor.primitive?.topology || 'triangle-list'] || 4;

    const depthStencil = descriptor.depthStencil;
    const depthFormat = {
      'depth32float': 1,
      'depth24plus': 2,
      'depth24plus-stencil8': 3,
    }[depthStencil?.format] || 0;

    const depthCompare = {
      'never': 1,
      'less': 2,
      'equal': 3,
      'less-equal': 4,
      'greater': 5,
      'not-equal': 6,
      'greater-equal': 7,
      'always': 8,
    }[depthStencil?.depthCompare || 'less'] || 2;

    const blendFactorMap = {
      'zero': 0, 'one': 1, 'src': 2, 'one-minus-src': 3,
      'src-alpha': 4, 'one-minus-src-alpha': 5,
      'dst': 6, 'one-minus-dst': 7, 'dst-alpha': 8, 'one-minus-dst-alpha': 9,
    };

    const blendOpMap = {
      'add': 0, 'subtract': 1, 'reverse-subtract': 2, 'min': 3, 'max': 4,
    };

    const fragmentTarget = descriptor.fragment.targets?.[0];
    const blend = fragmentTarget?.blend;

    const pipelineHandle = this.wasm.wasm_webgpu_create_render_pipeline(
      this.ctxHandle,
      this.deviceHandle,
      descriptor.vertex.module.moduleHandle,
      vPtr,
      vBytes.length,
      descriptor.fragment.module.moduleHandle,
      fPtr,
      fBytes.length,
      lPtr,
      layoutData.length,
      layoutHandle,
      primitiveTopology,
      depthFormat,
      depthStencil?.depthWriteEnabled ? 1 : 0,
      depthCompare,
      blend ? 1 : 0,
      blendFactorMap[blend?.color?.srcFactor] || 0,
      blendFactorMap[blend?.color?.dstFactor] || 0,
      blendOpMap[blend?.color?.operation] || 0,
      blendFactorMap[blend?.alpha?.srcFactor] || 0,
      blendFactorMap[blend?.alpha?.dstFactor] || 0,
      blendOpMap[blend?.alpha?.operation] || 0
    );

    this.wasm.wasm_free(vPtr, vBytes.length);
    this.wasm.wasm_free(fPtr, fBytes.length);
    this.wasm.wasm_free(lPtr, layoutData.length * 4);

    if (pipelineHandle === 0) {
      // Error already captured by Rust
    }

    return new GPURenderPipeline(this.wasm, this.memory, this.ctxHandle, pipelineHandle);
  }

  /**
   * Create a command encoder
   * @param {Object} descriptor - Command encoder descriptor
   * @returns {GPUCommandEncoder}
   */
  createCommandEncoder(descriptor = {}) {
    const encoderHandle = this.wasm.wasm_webgpu_create_command_encoder(this.ctxHandle, this.deviceHandle);
    if (encoderHandle === 0) {
      // Error already captured by Rust
    }
    return new GPUCommandEncoder(this.wasm, this.memory, this.ctxHandle, encoderHandle);
  }

  /**
   * Create a bind group layout
   * @param {Object} descriptor
   * @returns {GPUBindGroupLayout}
   */
  createBindGroupLayout(descriptor) {
    const entries = descriptor.entries || [];
    const data = [entries.length];

    for (const entry of entries) {
      data.push(entry.binding);
      data.push(entry.visibility);

      // Type: 0=Buffer, 1=Texture, 2=Sampler
      let typeId = 0;
      if (entry.buffer) typeId = 0;
      else if (entry.texture) typeId = 1;
      else if (entry.sampler) typeId = 2;

      data.push(typeId);
    }

    const ptr = this.wasm.wasm_alloc(data.length * 4);
    new Uint32Array(this.memory.buffer, ptr, data.length).set(data);

    const handle = this.wasm.wasm_webgpu_create_bind_group_layout(
      this.ctxHandle,
      this.deviceHandle,
      ptr,
      data.length
    );

    this.wasm.wasm_free(ptr, data.length * 4);

    if (handle === 0) {
      // Error already captured by Rust
    }

    return new GPUBindGroupLayout(this.wasm, this.memory, this.ctxHandle, handle);
  }

  /**
   * Create a bind group
   * @param {Object} descriptor
   * @returns {GPUBindGroup}
   */
  createBindGroup(descriptor) {
    const entries = descriptor.entries || [];
    const data = [entries.length];

    for (const entry of entries) {
      data.push(entry.binding);

      // Resource Type: 0=Buffer, 1=TextureView, 2=Sampler
      let resType = 0;
      let resHandle = 0;

      if (entry.resource.buffer) {
        resType = 0;
        resHandle = entry.resource.buffer.bufferHandle;
      } else if (entry.resource instanceof GPUTextureView) {
        resType = 1;
        resHandle = entry.resource.viewHandle;
      } else if (entry.resource instanceof GPUSampler) {
        resType = 2;
        resHandle = entry.resource.samplerHandle;
      } else if (entry.resource.constructor.name === 'GPUSampler') {
        resType = 2;
        resHandle = entry.resource.samplerHandle;
      }

      data.push(resType);
      data.push(resHandle);
    }

    const ptr = this.wasm.wasm_alloc(data.length * 4);
    new Uint32Array(this.memory.buffer, ptr, data.length).set(data);

    const handle = this.wasm.wasm_webgpu_create_bind_group(
      this.ctxHandle,
      this.deviceHandle,
      descriptor.layout.layoutHandle,
      ptr,
      data.length
    );

    this.wasm.wasm_free(ptr, data.length * 4);

    if (handle === 0) {
      // Error already captured by Rust
    }

    return new GPUBindGroup(this.wasm, this.memory, this.ctxHandle, handle);
  }

  /**
   * Create a sampler
   * @param {Object} descriptor
   * @returns {GPUSampler}
   */
  createSampler(descriptor = {}) {
    const addressModeMapping = {
      'clamp-to-edge': 0,
      'repeat': 1,
      'mirror-repeat': 2,
    };
    const filterMapping = {
      'nearest': 0,
      'linear': 1,
    };
    const compareMapping = {
      'never': 1,
      'less': 2,
      'equal': 3,
      'less-equal': 4,
      'greater': 5,
      'not-equal': 6,
      'greater-equal': 7,
      'always': 8,
    };

    const handle = this.wasm.wasm_webgpu_create_sampler(
      this.ctxHandle,
      this.deviceHandle,
      addressModeMapping[descriptor.addressModeU] || 0,
      addressModeMapping[descriptor.addressModeV] || 0,
      addressModeMapping[descriptor.addressModeW] || 0,
      filterMapping[descriptor.magFilter] || 0,
      filterMapping[descriptor.minFilter] || 0,
      filterMapping[descriptor.mipmapFilter] || 0,
      descriptor.lodMinClamp || 0.0,
      descriptor.lodMaxClamp || 32.0,
      descriptor.compare ? compareMapping[descriptor.compare] || 0 : 0,
      descriptor.maxAnisotropy || 1
    );

    if (handle === 0) {
      // Error already captured by Rust
    }
    return new GPUSampler(this.wasm, this.memory, this.ctxHandle, handle);
  }

  /**
   * Destroy the device
   */
  destroy() {
    if (this._destroyed) return;
    activeDevices.delete(this);
    if (typeof this.wasm.wasm_webgpu_destroy_context === 'function') {
      this.wasm.wasm_webgpu_destroy_context(this.ctxHandle);
    }
    this._destroyed = true;
  }
}

export class GPUQueue {
  /**
   * @param {*} wasmModule
   * @param {WebAssembly.Memory} wasmMemory
   * @param {number} ctxHandle
   * @param {number} queueHandle
   */
  constructor(wasmModule, wasmMemory, ctxHandle, queueHandle) {
    this.wasm = wasmModule;
    this.memory = wasmMemory;
    this.ctxHandle = ctxHandle;
    this.queueHandle = queueHandle;
  }

  /**
   * Submit command buffers to the queue
   * @param {Array<GPUCommandBuffer>} commandBuffers
   */
  submit(commandBuffers) {
    const handles = new Uint32Array(commandBuffers.map(cb => cb.commandBufferHandle));
    const ptr = this.wasm.wasm_alloc(handles.byteLength);
    const heapU32 = new Uint32Array(this.memory.buffer, ptr, handles.length);
    heapU32.set(handles);

    this.wasm.wasm_webgpu_queue_submit(this.ctxHandle, this.queueHandle, ptr, handles.length);

    this.wasm.wasm_free(ptr, handles.byteLength);
  }

  /**
   * Write data to a buffer
   * @param {GPUBuffer} buffer
   * @param {number} bufferOffset
   * @param {ArrayBuffer|TypedArray} data
   * @param {number} dataOffset
   * @param {number} size
   */
  writeBuffer(buffer, bufferOffset, data, dataOffset = 0, size) {
    const srcData = data instanceof ArrayBuffer ? new Uint8Array(data) : new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
    const actualSize = size !== undefined ? size : srcData.byteLength - dataOffset;
    const subData = srcData.subarray(dataOffset, dataOffset + actualSize);

    const ptr = this.wasm.wasm_alloc(subData.byteLength);
    const heap = new Uint8Array(this.memory.buffer, ptr, subData.byteLength);
    heap.set(subData);

    this.wasm.wasm_webgpu_queue_write_buffer(
      this.ctxHandle,
      this.queueHandle,
      buffer.bufferHandle,
      BigInt(bufferOffset),
      ptr,
      subData.byteLength
    );

    this.wasm.wasm_free(ptr, subData.byteLength);
  }

  /**
   * Write data to a texture
   * @param {Object} destination
   * @param {ArrayBuffer|TypedArray} data
   * @param {Object} dataLayout
   * @param {Object} size
   */
  writeTexture(destination, data, dataLayout, size) {
    const srcData = data instanceof ArrayBuffer ? new Uint8Array(data) : new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
    const subData = dataLayout.offset ? srcData.subarray(dataLayout.offset) : srcData;

    const ptr = this.wasm.wasm_alloc(subData.byteLength);
    const heap = new Uint8Array(this.memory.buffer, ptr, subData.byteLength);
    heap.set(subData);

    let width, height, depthOrArrayLayers;
    if (Array.isArray(size)) {
      width = size[0];
      height = size[1] || 1;
      depthOrArrayLayers = size[2] || 1;
    } else {
      width = size.width;
      height = size.height || 1;
      depthOrArrayLayers = size.depthOrArrayLayers || 1;
    }

    this.wasm.wasm_webgpu_queue_write_texture(
      this.ctxHandle,
      this.queueHandle,
      destination.texture.textureHandle,
      ptr,
      subData.byteLength,
      dataLayout.bytesPerRow || 0,
      dataLayout.rowsPerImage || 0,
      width,
      height,
      depthOrArrayLayers
    );

    this.wasm.wasm_free(ptr, subData.byteLength);
  }
}

export class GPUBuffer {
  /**
   * @param {*} wasmModule
   * @param {WebAssembly.Memory} wasmMemory
   * @param {number} ctxHandle
   * @param {number} deviceHandle
   * @param {number} bufferHandle
   * @param {number} size
   */
  constructor(wasmModule, wasmMemory, ctxHandle, deviceHandle, bufferHandle, size) {
    this.wasm = wasmModule;
    this.memory = wasmMemory;
    this.ctxHandle = ctxHandle;
    this.deviceHandle = deviceHandle;
    this.bufferHandle = bufferHandle;
    this.size = size;
  }

  /**
   * Map buffer for reading/writing
   * @param {number} mode - Map mode
   * @param {number} offset
   * @param {number} size
   * @returns {Promise<void>}
   */
  async mapAsync(mode, offset = 0, size) {
    const result = this.wasm.wasm_webgpu_buffer_map_async(
      this.ctxHandle,
      this.deviceHandle,
      this.bufferHandle,
      mode,
      BigInt(offset),
      BigInt(size || this.size)
    );
    if (result !== 0) {
      throw new Error("Failed to map buffer");
    }
    return Promise.resolve();
  }

  /**
   * Get mapped range
   * @param {number} offset
   * @param {number} size
   * @returns {Uint8Array}
   */
  getMappedRange(offset = 0, size) {
    const ptr = this.wasm.wasm_webgpu_buffer_get_mapped_range(
      this.ctxHandle,
      this.bufferHandle,
      BigInt(offset),
      BigInt(size || this.size)
    );
    if (ptr === 0) {
      throw new Error("Failed to get mapped range");
    }
    return new Uint8Array(this.memory.buffer, ptr, (size || this.size));
  }

  /**
   * Unmap the buffer
   */
  unmap() {
    this.wasm.wasm_webgpu_buffer_unmap(this.ctxHandle, this.bufferHandle);
  }

  /**
   * Destroy the buffer
   */
  destroy() {
    this.wasm.wasm_webgpu_buffer_destroy(this.ctxHandle, this.bufferHandle);
  }
}

export class GPUShaderModule {
  /**
   * @param {*} wasmModule
   * @param {WebAssembly.Memory} wasmMemory
   * @param {number} ctxHandle
   * @param {number} moduleHandle
   */
  constructor(wasmModule, wasmMemory, ctxHandle, moduleHandle) {
    this.wasm = wasmModule;
    this.memory = wasmMemory;
    this.ctxHandle = ctxHandle;
    this.moduleHandle = moduleHandle;
  }
}

export class GPURenderPipeline {
  /**
   * @param {*} wasmModule
   * @param {WebAssembly.Memory} wasmMemory
   * @param {number} ctxHandle
   * @param {number} pipelineHandle
   */
  constructor(wasmModule, wasmMemory, ctxHandle, pipelineHandle) {
    this.wasm = wasmModule;
    this.memory = wasmMemory;
    this.ctxHandle = ctxHandle;
    this.pipelineHandle = pipelineHandle;
  }

  /**
   * @param {number} index
   * @returns {GPUBindGroupLayout}
   */
  getBindGroupLayout(index) {
    const layoutHandle = this.wasm.wasm_webgpu_pipeline_get_bind_group_layout(this.ctxHandle, this.pipelineHandle, index);
    return new GPUBindGroupLayout(this.wasm, this.memory, this.ctxHandle, layoutHandle);
  }
}

export class GPUCommandEncoder {
  /**
   * @param {*} wasmModule
   * @param {WebAssembly.Memory} wasmMemory
   * @param {number} ctxHandle
   * @param {number} encoderHandle
   */
  constructor(wasmModule, wasmMemory, ctxHandle, encoderHandle) {
    this.wasm = wasmModule;
    this.memory = wasmMemory;
    this.ctxHandle = ctxHandle;
    this.encoderHandle = encoderHandle;
  }

  /**
   * Copy data from one buffer to another
   * @param {GPUBuffer} source
   * @param {number} sourceOffset
   * @param {GPUBuffer} destination
   * @param {number} destinationOffset
   * @param {number} size
   */
  copyBufferToBuffer(source, sourceOffset, destination, destinationOffset, size) {
    this.wasm.wasm_webgpu_command_encoder_copy_buffer_to_buffer(
      this.ctxHandle,
      this.encoderHandle,
      source.bufferHandle,
      BigInt(sourceOffset),
      destination.bufferHandle,
      BigInt(destinationOffset),
      BigInt(size)
    );
  }

  /**
   * Copy texture to buffer
   * @param {Object} source
   * @param {Object} destination
   * @param {Object} size
   */
  copyTextureToBuffer(source, destination, size) {
    let width, height, depthOrArrayLayers;
    if (Array.isArray(size)) {
      width = size[0];
      height = size[1] || 1;
      depthOrArrayLayers = size[2] || 1;
    } else {
      width = size.width;
      height = size.height || 1;
      depthOrArrayLayers = size.depthOrArrayLayers || 1;
    }

    this.wasm.wasm_webgpu_command_encoder_copy_texture_to_buffer(
      this.ctxHandle,
      this.encoderHandle,
      source.texture.textureHandle,
      destination.buffer.bufferHandle,
      BigInt(destination.offset || 0),
      destination.bytesPerRow || 0,
      destination.rowsPerImage || 0,
      width,
      height,
      depthOrArrayLayers
    );
  }

  /**
   * Begin a render pass
   * @param {Object} descriptor - Render pass descriptor
   * @returns {GPURenderPassEncoder}
   */
  beginRenderPass(descriptor) {
    const att = descriptor.colorAttachments[0];
    const loadOp = att.loadOp === 'clear' ? 1 : 0;
    const storeOp = att.storeOp === 'discard' ? 1 : 0;
    const clearColor = att.clearValue || { r: 0, g: 0, b: 0, a: 0 };

    const passHandle = this.wasm.wasm_webgpu_command_encoder_begin_render_pass(
      this.ctxHandle,
      this.encoderHandle,
      att.view.viewHandle,
      loadOp,
      storeOp,
      clearColor.r,
      clearColor.g,
      clearColor.b,
      clearColor.a
    );

    return new GPURenderPassEncoder(this.wasm, this.memory, this.ctxHandle, passHandle);
  }

  /**
   * Finish encoding and create a command buffer
   * @returns {GPUCommandBuffer}
   */
  finish() {
    const commandBufferHandle = this.wasm.wasm_webgpu_command_encoder_finish(this.ctxHandle, this.encoderHandle);
    if (commandBufferHandle === 0) {
      throw new Error("Failed to finish command encoder");
    }
    return new GPUCommandBuffer(this.wasm, this.memory, this.ctxHandle, commandBufferHandle);
  }
}

export class GPURenderPassEncoder {
  /**
   * @param {*} wasmModule
   * @param {WebAssembly.Memory} wasmMemory
   * @param {number} ctxHandle
   * @param {number} passHandle
   */
  constructor(wasmModule, wasmMemory, ctxHandle, passHandle) {
    this.wasm = wasmModule;
    this.memory = wasmMemory;
    this.ctxHandle = ctxHandle;
    this.passHandle = passHandle;
  }

  /**
   * Set the render pipeline
   * @param {GPURenderPipeline} pipeline
   */
  setPipeline(pipeline) {
    this.wasm.wasm_webgpu_render_pass_set_pipeline(this.ctxHandle, this.passHandle, pipeline.pipelineHandle);
  }

  /**
   * Set vertex buffer
   * @param {number} slot
   * @param {GPUBuffer} buffer
   * @param {number} offset
   * @param {number} size
   */
  setVertexBuffer(slot, buffer, offset = 0, size) {
    this.wasm.wasm_webgpu_render_pass_set_vertex_buffer(
      this.ctxHandle,
      this.passHandle,
      slot,
      buffer.bufferHandle,
      BigInt(offset),
      BigInt(size || buffer.size)
    );
  }

  /**
   * Set bind group
   * @param {number} index
   * @param {GPUBindGroup} bindGroup
   * @param {Array<number>} dynamicOffsets
   */
  setBindGroup(index, bindGroup, dynamicOffsets = []) {
    this.wasm.wasm_webgpu_render_pass_set_bind_group(this.ctxHandle, this.passHandle, index, bindGroup.bindGroupHandle);
  }

  /**
   * Set index buffer
   * @param {GPUBuffer} buffer
   * @param {string} indexFormat
   * @param {number} offset
   * @param {number} size
   */
  setIndexBuffer(buffer, indexFormat, offset = 0, size) {
    const formatId = indexFormat === 'uint32' ? 2 : 1;
    this.wasm.wasm_webgpu_render_pass_set_index_buffer(
      this.ctxHandle,
      this.passHandle,
      buffer.bufferHandle,
      formatId,
      BigInt(offset),
      BigInt(size || (buffer.size - offset))
    );
  }

  /**
   * Draw vertices
   * @param {number} vertexCount
   * @param {number} instanceCount
   * @param {number} firstVertex
   * @param {number} firstInstance
   */
  draw(vertexCount, instanceCount = 1, firstVertex = 0, firstInstance = 0) {
    this.wasm.wasm_webgpu_render_pass_draw(this.ctxHandle, this.passHandle, vertexCount, instanceCount, firstVertex, firstInstance);
  }

  /**
   * Draw indexed vertices
   * @param {number} indexCount
   * @param {number} instanceCount
   * @param {number} firstIndex
   * @param {number} baseVertex
   * @param {number} firstInstance
   */
  drawIndexed(indexCount, instanceCount = 1, firstIndex = 0, baseVertex = 0, firstInstance = 0) {
    this.wasm.wasm_webgpu_render_pass_draw_indexed(this.ctxHandle, this.passHandle, indexCount, instanceCount, firstIndex, baseVertex, firstInstance);
  }

  /**
   * Set viewport
   * @param {number} x
   * @param {number} y
   * @param {number} width
   * @param {number} height
   * @param {number} minDepth
   * @param {number} maxDepth
   */
  setViewport(x, y, width, height, minDepth, maxDepth) {
    this.wasm.wasm_webgpu_render_pass_set_viewport(this.ctxHandle, this.passHandle, x, y, width, height, minDepth, maxDepth);
  }

  /**
   * Set scissor rectangle
   * @param {number} x
   * @param {number} y
   * @param {number} width
   * @param {number} height
   */
  setScissorRect(x, y, width, height) {
    this.wasm.wasm_webgpu_render_pass_set_scissor_rect(this.ctxHandle, this.passHandle, x, y, width, height);
  }

  /**
   * End the render pass
   */
  end() {
    this.wasm.wasm_webgpu_render_pass_end(this.ctxHandle, this.passHandle);
  }
}

export class GPUCommandBuffer {
  /**
   * @param {*} wasmModule
   * @param {WebAssembly.Memory} wasmMemory
   * @param {number} ctxHandle
   * @param {number} commandBufferHandle
   */
  constructor(wasmModule, wasmMemory, ctxHandle, commandBufferHandle) {
    this.wasm = wasmModule;
    this.memory = wasmMemory;
    this.ctxHandle = ctxHandle;
    this.commandBufferHandle = commandBufferHandle;
  }
}

export class GPUTexture {
  /**
   * @param {*} wasmModule
   * @param {WebAssembly.Memory} wasmMemory
   * @param {number} ctxHandle
   * @param {number} deviceHandle
   * @param {number} textureHandle
   * @param {Object} descriptor
   */
  constructor(wasmModule, wasmMemory, ctxHandle, deviceHandle, textureHandle, descriptor) {
    this.wasm = wasmModule;
    this.memory = wasmMemory;
    this.ctxHandle = ctxHandle;
    this.deviceHandle = deviceHandle;
    this.textureHandle = textureHandle;
    this.width = descriptor.size.width;
    this.height = descriptor.size.height;
    this.depthOrArrayLayers = descriptor.size.depthOrArrayLayers || 1;
    this.format = descriptor.format;
    this.usage = descriptor.usage;
  }

  /**
   * Create a view
   * @param {Object} descriptor
   * @returns {GPUTextureView}
   */
  createView(descriptor = {}) {
    const format = TEXTURE_FORMAT_MAP[descriptor.format] || 0; // 0 for default
    const dimensionMap = {
      '1d': 1,
      '2d': 2,
      '2d-array': 3,
      'cube': 4,
      'cube-array': 5,
      '3d': 6
    };
    const aspectMap = {
      'all': 0,
      'stencil-only': 1,
      'depth-only': 2
    };

    const viewHandle = this.wasm.wasm_webgpu_create_texture_view(
      this.ctxHandle,
      this.textureHandle,
      format,
      dimensionMap[descriptor.dimension] || 0,
      descriptor.baseMipLevel || 0,
      descriptor.mipLevelCount || 0,
      descriptor.baseArrayLayer || 0,
      descriptor.arrayLayerCount || 0,
      aspectMap[descriptor.aspect] || 0
    );
    if (viewHandle === 0) {
      // Error already captured by Rust
    }
    return new GPUTextureView(this.wasm, this.memory, this.ctxHandle, viewHandle, this);
  }

  destroy() {
    this.wasm.wasm_webgpu_destroy_texture(this.ctxHandle, this.textureHandle);
    this.textureHandle = 0;
  }
}

export class GPUTextureView {
  /**
   * @param {*} wasmModule
   * @param {WebAssembly.Memory} wasmMemory
   * @param {number} ctxHandle
   * @param {number} viewHandle
   * @param {GPUTexture} texture
   */
  constructor(wasmModule, wasmMemory, ctxHandle, viewHandle, texture) {
    this.wasm = wasmModule;
    this.memory = wasmMemory;
    this.ctxHandle = ctxHandle;
    this.viewHandle = viewHandle;
    this.texture = texture;
  }
}

export class GPUBindGroupLayout {
  constructor(wasmModule, wasmMemory, ctxHandle, layoutHandle) {
    this.wasm = wasmModule;
    this.memory = wasmMemory;
    this.ctxHandle = ctxHandle;
    this.layoutHandle = layoutHandle;
  }
}

export class GPUBindGroup {
  constructor(wasmModule, wasmMemory, ctxHandle, bindGroupHandle) {
    this.wasm = wasmModule;
    this.memory = wasmMemory;
    this.ctxHandle = ctxHandle;
    this.bindGroupHandle = bindGroupHandle;
  }
}

export class GPUSampler {
  constructor(wasmModule, wasmMemory, ctxHandle, samplerHandle) {
    this.wasm = wasmModule;
    this.memory = wasmMemory;
    this.ctxHandle = ctxHandle;
    this.samplerHandle = samplerHandle;
  }
}

/**
 * Create a navigator.gpu object
 * @param {Object} wasmModule - WebAssembly module
 * @param {WebAssembly.Memory} wasmMemory - WebAssembly memory
 * @returns {GPU}
 */
export function createWebGPU(wasmModule, wasmMemory) {
  return new GPU(wasmModule, wasmMemory);
}

export class GPUCanvasContext {
  /**
   * @param {*} wasmModule
   * @param {WebAssembly.Memory} wasmMemory
   * @param {HTMLCanvasElement} canvas
   */
  constructor(wasmModule, wasmMemory, canvas) {
    this.wasm = wasmModule;
    this.memory = wasmMemory;
    this.canvas = canvas;
    this.device = null;
    this.format = 'rgba8unorm';
    this.usage = GPUTextureUsage.RENDER_ATTACHMENT;
    this.width = canvas.width;
    this.height = canvas.height;
  }

  /**
   * Configure the context
   * @param {Object} descriptor
   */
  configure(descriptor) {
    this.device = descriptor.device;
    this.format = descriptor.format || 'rgba8unorm';
    this.usage = descriptor.usage || GPUTextureUsage.RENDER_ATTACHMENT;
    this.alphaMode = descriptor.alphaMode || 'opaque';

    // Resize canvas internal buffer if needed
    if (this.canvas.width !== this.width || this.canvas.height !== this.height) {
      this.width = this.canvas.width;
      this.height = this.canvas.height;
    }
  }

  unconfigure() {
    this.device = null;
  }

  /**
   * Get the current texture to render into
   * @returns {GPUTexture}
   */
  getCurrentTexture() {
    if (!this.device) {
      throw new Error("Context not configured");
    }

    // Create a temporary texture that represents the canvas surface
    // In a real implementation, this would be a managed swapchain texture.
    // For SoftApi, we just create a regular texture that we will present later.
    return this.device.createTexture({
      size: { width: this.width, height: this.height, depthOrArrayLayers: 1 },
      format: this.format,
      usage: this.usage | GPUTextureUsage.COPY_SRC
    });
  }

  /**
   * Present the current texture to the canvas
   * This is a non-standard method for our Soft-GPU to bridge to the browser display.
   * @param {GPUTexture} texture
   */
  present(texture) {
    const ctx2d = this.canvas.getContext('2d');
    if (!ctx2d) return;

    const width = texture.width;
    const height = texture.height;

    // Use readPixels-like logic to get data from WASM
    const len = width * height * 4;

    // We need a buffer to copy the texture to
    const buffer = this.device.createBuffer({
      size: len,
      usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ
    });

    const encoder = this.device.createCommandEncoder();
    encoder.copyTextureToBuffer(
      { texture: texture },
      { buffer: buffer, bytesPerRow: width * 4 },
      { width, height, depthOrArrayLayers: 1 }
    );
    this.device.queue.submit([encoder.finish()]);

    // Map and copy to canvas
    buffer.mapAsync(GPUMapMode.READ).then(() => {
      const data = buffer.getMappedRange();
      const clamped = new Uint8ClampedArray(data.buffer, data.byteOffset, data.byteLength);
      const imageData = new ImageData(clamped, width, height);
      ctx2d.putImageData(imageData, 0, 0);
      buffer.unmap();
      buffer.destroy();
      texture.destroy();
    });
  }
}

export class GPUPipelineLayout {
  constructor(wasmModule, wasmMemory, ctxHandle, layoutHandle) {
    this.wasm = wasmModule;
    this.memory = wasmMemory;
    this.ctxHandle = ctxHandle;
    this.layoutHandle = layoutHandle;
  }
}

function readString(memory, ptr) {
  if (!ptr) return null;
  const view = new Uint8Array(memory.buffer);
  let end = ptr;
  while (view[end]) end++;
  return new TextDecoder().decode(view.subarray(ptr, end));
}

export class GPUError {
  constructor(message) {
    this.message = message;
  }
}

export class GPUValidationError extends GPUError {
  constructor(message) {
    super(message);
    this.name = 'GPUValidationError';
  }
}

export class GPUInternalError extends GPUError {
  constructor(message) {
    super(message);
    this.name = 'GPUInternalError';
  }
}

export class GPUOutOfMemoryError extends GPUError {
  constructor(message) {
    super(message);
    this.name = 'GPUOutOfMemoryError';
  }
}
