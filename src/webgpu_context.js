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
     * Create a texture
     * @param {Object} descriptor - Texture descriptor
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
            0, // format (ignored by backend for now)
            descriptor.usage
        );
        if (textureHandle === 0) {
            throw new Error("Failed to create texture");
        }

        // Create a normalized descriptor for the GPUTexture
        const normalizedDescriptor = Object.assign({}, descriptor, {
            size: { width, height, depthOrArrayLayers }
        });

        return new GPUTexture(this.wasm, this.memory, this.ctxHandle, this.deviceHandle, textureHandle, normalizedDescriptor);
    }

    /**
     * Create a shader module
     * @param {Object} descriptor - Shader module descriptor
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
            throw new Error("Failed to create shader module");
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
            throw new Error("Failed to create pipeline layout");
        }

        return new GPUPipelineLayout(this.wasm, this.memory, this.ctxHandle, handle);
    }

    /**
     * Create a render pipeline
     * @param {Object} descriptor - Render pipeline descriptor
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
            layoutHandle
        );

        this.wasm.wasm_free(vPtr, vBytes.length);
        this.wasm.wasm_free(fPtr, fBytes.length);
        this.wasm.wasm_free(lPtr, layoutData.length * 4);

        if (pipelineHandle === 0) {
            throw new Error("Failed to create render pipeline");
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
            throw new Error("Failed to create command encoder");
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

        if (handle === 0) throw new Error("Failed to create bind group layout");

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

        if (handle === 0) throw new Error("Failed to create bind group");

        return new GPUBindGroup(this.wasm, this.memory, this.ctxHandle, handle);
    }

    /**
     * Create a sampler
     * @param {Object} descriptor
     * @returns {GPUSampler}
     */
    createSampler(descriptor = {}) {
        const handle = this.wasm.wasm_webgpu_create_sampler(this.ctxHandle, this.deviceHandle);
        if (handle === 0) throw new Error("Failed to create sampler");
        return new GPUSampler(this.wasm, this.memory, this.ctxHandle, handle);
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
        return new GPURenderPassEncoder(this.wasm, this.memory, this.ctxHandle, this.encoderHandle, descriptor);
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
     * @param {number} encoderHandle
     * @param {Object} descriptor
     */
    constructor(wasmModule, wasmMemory, ctxHandle, encoderHandle, descriptor) {
        this.wasm = wasmModule;
        this.memory = wasmMemory;
        this.ctxHandle = ctxHandle;
        this.encoderHandle = encoderHandle;
        this.descriptor = descriptor;
        this.commands = [];
    }

    /**
     * Set the render pipeline
     * @param {GPURenderPipeline} pipeline
     */
    setPipeline(pipeline) {
        this.commands.push(1, pipeline.pipelineHandle);
    }

    /**
     * Set vertex buffer
     * @param {number} slot
     * @param {GPUBuffer} buffer
     * @param {number} offset
     * @param {number} size
     */
    setVertexBuffer(slot, buffer, offset = 0, size) {
        this.commands.push(2, slot, buffer.bufferHandle, offset, size || buffer.size);
    }

    /**
     * Set bind group
     * @param {number} index
     * @param {GPUBindGroup} bindGroup
     * @param {Array<number>} dynamicOffsets
     */
    setBindGroup(index, bindGroup, dynamicOffsets = []) {
        this.commands.push(4, index, bindGroup.bindGroupHandle);
    }

    /**
     * Draw vertices
     * @param {number} vertexCount
     * @param {number} instanceCount
     * @param {number} firstVertex
     * @param {number} firstInstance
     */
    draw(vertexCount, instanceCount = 1, firstVertex = 0, firstInstance = 0) {
        this.commands.push(3, vertexCount, instanceCount, firstVertex, firstInstance);
    }

    /**
     * End the render pass
     */
    end() {
        // Execute the render pass with all buffered commands
        const att = this.descriptor.colorAttachments[0];
        const loadOp = att.loadOp === 'clear' ? 1 : 0;
        const storeOp = att.storeOp === 'discard' ? 1 : 0;
        const clearColor = att.clearValue || { r: 0, g: 0, b: 0, a: 0 };

        const ptr = this.wasm.wasm_alloc(this.commands.length * 4);
        const heapU32 = new Uint32Array(this.memory.buffer, ptr, this.commands.length);
        heapU32.set(this.commands);

        this.wasm.wasm_webgpu_command_encoder_run_render_pass(
            this.ctxHandle,
            this.encoderHandle,
            att.view.viewHandle,
            loadOp,
            storeOp,
            clearColor.r,
            clearColor.g,
            clearColor.b,
            clearColor.a,
            ptr,
            this.commands.length
        );

        this.wasm.wasm_free(ptr, this.commands.length * 4);
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
        const viewHandle = this.wasm.wasm_webgpu_create_texture_view(
            this.ctxHandle,
            this.textureHandle
        );
        if (viewHandle === 0) {
            throw new Error("Failed to create texture view");
        }
        return new GPUTextureView(this.wasm, this.memory, this.ctxHandle, viewHandle, this);
    }

    destroy() {
        // TODO
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

export class GPUPipelineLayout {
    constructor(wasmModule, wasmMemory, ctxHandle, layoutHandle) {
        this.wasm = wasmModule;
        this.memory = wasmMemory;
        this.ctxHandle = ctxHandle;
        this.layoutHandle = layoutHandle;
    }
}
