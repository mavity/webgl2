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
        this.queue = new GPUQueue(wasmModule, wasmMemory, ctxHandle, 1); // Placeholder queue handle
        this._destroyed = false;
    }

    /**
     * Create a buffer
     * @param {GPUBufferDescriptor} descriptor - Buffer descriptor
     * @returns {GPUBuffer}
     */
    createBuffer(descriptor) {
        // TODO: Call wasm function to create buffer
        const bufferHandle = 1; // Placeholder
        return new GPUBuffer(this.wasm, this.memory, this.ctxHandle, bufferHandle, descriptor.size);
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
        // TODO: Call wasm function to create command encoder
        const encoderHandle = 1; // Placeholder
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
        // TODO: Call wasm function to submit command buffers
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
     * @param {number} bufferHandle
     * @param {number} size
     */
    constructor(wasmModule, wasmMemory, ctxHandle, bufferHandle, size) {
        this.wasm = wasmModule;
        this.memory = wasmMemory;
        this.ctxHandle = ctxHandle;
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
        // TODO: Implement buffer mapping
        return Promise.resolve();
    }

    /**
     * Get mapped range
     * @param {number} offset
     * @param {number} size
     * @returns {ArrayBuffer}
     */
    getMappedRange(offset = 0, size) {
        // TODO: Return mapped range
        return new ArrayBuffer(size || this.size);
    }

    /**
     * Unmap the buffer
     */
    unmap() {
        // TODO: Implement unmapping
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
        // TODO: Call wasm function to finish encoding
        const commandBufferHandle = 1; // Placeholder
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
