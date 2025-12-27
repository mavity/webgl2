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
 * WebGPU API implementation for WebAssembly
 * 
 * This module provides a complete WebGPU API surface that runs entirely in
 * WebAssembly/Rust, enabling deterministic execution, advanced debugging,
 * and software rasterization of WebGPU workloads.
 */

export const GPUBufferUsage = {
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
};

export const GPUMapMode = {
    READ: 0x0001,
    WRITE: 0x0002,
};

export const GPUTextureUsage = {
    COPY_SRC: 0x01,
    COPY_DST: 0x02,
    TEXTURE_BINDING: 0x04,
    STORAGE_BINDING: 0x08,
    RENDER_ATTACHMENT: 0x10,
};

export const GPUShaderStage = {
    VERTEX: 0x1,
    FRAGMENT: 0x2,
    COMPUTE: 0x4,
};

// Polyfill globals if missing (e.g. in Node.js)
if (typeof globalThis !== 'undefined') {
    if (!globalThis.GPUBufferUsage) globalThis.GPUBufferUsage = GPUBufferUsage;
    if (!globalThis.GPUMapMode) globalThis.GPUMapMode = GPUMapMode;
    if (!globalThis.GPUTextureUsage) globalThis.GPUTextureUsage = GPUTextureUsage;
    if (!globalThis.GPUShaderStage) globalThis.GPUShaderStage = GPUShaderStage;
}

/**
 * Wrapper around a WebAssembly-backed WebGPU implementation.
 */
export class GPU {

    /**
     * @param {*} wasmModule - WebAssembly module exports implementing WebGPU.
     * @param {WebAssembly.Memory} wasmMemory - WebAssembly linear memory.
     */
    constructor(wasmModule, wasmMemory) {
        this.wasm = wasmModule;
        this.memory = wasmMemory;
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
        /** @type {Record<string, number>} */
        this.limits = {};
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

export class GPUDevice {
    /**
     * @param {*} wasmModule
     * @param {WebAssembly.Memory} wasmMemory
     * @param {number} ctxHandle
     * @param {number} deviceHandle
     */
    constructor(wasmModule, wasmMemory, ctxHandle, deviceHandle) {
        this.wasm = wasmModule;
        this.memory = wasmMemory;
        this.ctxHandle = ctxHandle;
        this.deviceHandle = deviceHandle;
        this.queue = new GPUQueue(wasmModule, wasmMemory, ctxHandle, deviceHandle);
        this._destroyed = false;
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
            throw new Error("Failed to create buffer");
        }
        return new GPUBuffer(this.wasm, this.memory, this.ctxHandle, this.deviceHandle, bufferHandle, descriptor.size);
    }

    /**
     * Create a shader module
     * @param {Object} descriptor - Shader module descriptor
     * @returns {GPUShaderModule}
     */
    createShaderModule(descriptor) {
        // TODO: Call wasm function to create shader module
        const moduleHandle = 1; // Placeholder
        return new GPUShaderModule(this.wasm, this.memory, this.ctxHandle, moduleHandle);
    }

    /**
     * Create a render pipeline
     * @param {Object} descriptor - Render pipeline descriptor
     * @returns {GPURenderPipeline}
     */
    createRenderPipeline(descriptor) {
        // TODO: Call wasm function to create render pipeline
        const pipelineHandle = 1; // Placeholder
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
            throw new Error("Failed to create command encoder");
        }
        return new GPUCommandEncoder(this.wasm, this.memory, this.ctxHandle, encoderHandle);
    }

    /**
     * Destroy the device
     */
    destroy() {
        if (this._destroyed) return;
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
        // TODO: Call wasm function to write buffer
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
        // TODO: Call wasm function to destroy buffer
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
     * Begin a render pass
     * @param {Object} descriptor - Render pass descriptor
     * @returns {GPURenderPassEncoder}
     */
    beginRenderPass(descriptor) {
        // TODO: Call wasm function to begin render pass
        const passHandle = 1; // Placeholder
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
        // TODO: Call wasm function to set pipeline
    }

    /**
     * Set vertex buffer
     * @param {number} slot
     * @param {GPUBuffer} buffer
     * @param {number} offset
     * @param {number} size
     */
    setVertexBuffer(slot, buffer, offset = 0, size) {
        // TODO: Call wasm function to set vertex buffer
    }

    /**
     * Draw vertices
     * @param {number} vertexCount
     * @param {number} instanceCount
     * @param {number} firstVertex
     * @param {number} firstInstance
     */
    draw(vertexCount, instanceCount = 1, firstVertex = 0, firstInstance = 0) {
        // TODO: Call wasm function to draw
    }

    /**
     * End the render pass
     */
    end() {
        // TODO: Call wasm function to end render pass
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

/**
 * Create a navigator.gpu object
 * @param {Object} wasmModule - WebAssembly module
 * @param {WebAssembly.Memory} wasmMemory - WebAssembly memory
 * @returns {GPU}
 */
export function createWebGPU(wasmModule, wasmMemory) {
    return new GPU(wasmModule, wasmMemory);
}
