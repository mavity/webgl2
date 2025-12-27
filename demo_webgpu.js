
import { createWebGPU } from './src/webgpu_context.js';

// Environment detection
const isNode = typeof process !== 'undefined' && process.versions && process.versions.node;

// Import fs only in Node
let fs;
async function initFS() {
    if (isNode) {
        const fsModule = await import('fs');
        fs = fsModule.default;
    }
}

async function initWebGPU() {
    let wasmPath;
    if (isNode) {
        wasmPath = './target/wasm32-unknown-unknown/debug/webgl2.wasm';
    } else {
        wasmPath = 'webgl2.wasm'; // Assuming served from root
    }

    const wasmBuffer = fs ? fs.readFileSync(wasmPath) : await fetch(wasmPath).then(r => r.arrayBuffer());
    
    // Create memory but allow it to be overridden by exports
    const initialMemory = new WebAssembly.Memory({ initial: 256, maximum: 256 });
    let activeMemory = initialMemory;

    const importObject = {
        env: {
            memory: initialMemory,
            print: (ptr, len) => {
                const msg = new TextDecoder().decode(new Uint8Array(activeMemory.buffer, ptr, len));
                console.log(msg);
            },
            wasm_execute_shader: (ctx, type, attr_ptr, uniform_ptr, varying_ptr, private_ptr, texture_ptr) => {
                console.log("wasm_execute_shader called!");
            }
        }
    };

    const { instance } = await WebAssembly.instantiate(wasmBuffer, importObject);
    
    if (instance.exports.memory) {
        activeMemory = instance.exports.memory;
    }
    
    return createWebGPU(instance.exports, activeMemory);
}

async function main() {
    await initFS();
    const gpu = await initWebGPU();
    
    const adapter = await gpu.requestAdapter();
    const device = await adapter.requestDevice();
    
    console.log("WebGPU Device created");

    // Shader
    const shaderCode = `
    struct Uniforms {
        mvp: mat4x4<f32>,
    };

    @group(0) @binding(0) var<uniform> uniforms: Uniforms;
    @group(0) @binding(1) var t_diffuse: texture_2d<f32>;
    @group(0) @binding(2) var s_diffuse: sampler;

    struct VertexInput {
        @location(0) position: vec3<f32>,
        @location(1) uv: vec2<f32>,
    };

    struct VertexOutput {
        @builtin(position) clip_position: vec4<f32>,
        @location(0) uv: vec2<f32>,
    };

    @vertex
    fn vs_main(model: VertexInput) -> VertexOutput {
        var out: VertexOutput;
        out.uv = model.uv;
        out.clip_position = uniforms.mvp * vec4<f32>(model.position, 1.0);
        return out;
    }

    @fragment
    fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
        return textureSample(t_diffuse, s_diffuse, in.uv);
    }
    `;

    const shaderModule = device.createShaderModule({ code: shaderCode });
    console.log("Shader Module created");

    // Bind Group Layout
    const bindGroupLayout = device.createBindGroupLayout({
        entries: [
            { binding: 0, visibility: GPUShaderStage.VERTEX, buffer: { type: 'uniform' } },
            { binding: 1, visibility: GPUShaderStage.FRAGMENT, texture: {} },
            { binding: 2, visibility: GPUShaderStage.FRAGMENT, sampler: {} },
        ]
    });

    // Pipeline Layout
    const pipelineLayout = device.createPipelineLayout({
        bindGroupLayouts: [bindGroupLayout]
    });

    // Pipeline
    const pipeline = device.createRenderPipeline({
        layout: pipelineLayout,
        vertex: {
            module: shaderModule,
            entryPoint: 'vs_main',
            buffers: [{
                arrayStride: 20, // 3 float pos + 2 float uv = 5 * 4 = 20
                attributes: [
                    { format: 'float32x3', offset: 0, shaderLocation: 0 },
                    { format: 'float32x2', offset: 12, shaderLocation: 1 },
                ]
            }]
        },
        fragment: {
            module: shaderModule,
            entryPoint: 'fs_main',
            targets: [{ format: 'rgba8unorm' }]
        }
    });
    console.log("Render Pipeline created");

    // Vertex Buffer (Cube)
    const vertices = new Float32Array([
        // Front face
        -0.5, -0.5,  0.5,  0.0, 0.0,
         0.5, -0.5,  0.5,  1.0, 0.0,
         0.5,  0.5,  0.5,  1.0, 1.0,
        -0.5, -0.5,  0.5,  0.0, 0.0,
         0.5,  0.5,  0.5,  1.0, 1.0,
        -0.5,  0.5,  0.5,  0.0, 1.0,
        // ... (truncated for brevity, just one face is enough to test)
    ]);
    
    const vertexBuffer = device.createBuffer({
        size: vertices.byteLength,
        usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
        mappedAtCreation: true
    });
    new Float32Array(vertexBuffer.getMappedRange()).set(vertices);
    vertexBuffer.unmap();
    console.log("Vertex Buffer created");

    // Texture
    const texture = device.createTexture({
        size: [16, 16, 1],
        format: 'rgba8unorm',
        usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST
    });
    
    // Upload texture data (simplified: just map and copy if we had mapAtCreation, but here we use queue)
    // For now, skip texture data upload or use writeBuffer/copyBufferToTexture if implemented.
    // We implemented copyTextureToBuffer but not copyBufferToTexture in JS wrapper fully?
    // Actually we can use mappedAtCreation for a staging buffer and copy.
    // But let's just leave it black/uninitialized for the pipeline test.
    
    const sampler = device.createSampler();
    console.log("Texture & Sampler created");

    // Uniform Buffer
    const uniformBuffer = device.createBuffer({
        size: 64, // mat4
        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST
    });
    console.log("Uniform Buffer created");

    // Bind Group
    /*
    const bindGroupLayout = device.createBindGroupLayout({
        entries: [
            { binding: 0, visibility: GPUShaderStage.VERTEX, buffer: { type: 'uniform' } },
            { binding: 1, visibility: GPUShaderStage.FRAGMENT, texture: {} },
            { binding: 2, visibility: GPUShaderStage.FRAGMENT, sampler: {} },
        ]
    });
    */

    const bindGroup = device.createBindGroup({
        layout: bindGroupLayout,
        entries: [
            { binding: 0, resource: { buffer: uniformBuffer } },
            { binding: 1, resource: texture.createView() },
            { binding: 2, resource: sampler },
        ]
    });
    console.log("Bind Group created");

    // Render Pass
    const commandEncoder = device.createCommandEncoder();
    const textureView = texture.createView(); // Reuse texture as render target for simplicity? No, need a separate target.
    
    const renderTarget = device.createTexture({
        size: [640, 480, 1],
        format: 'rgba8unorm',
        usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
    });
    
    const passEncoder = commandEncoder.beginRenderPass({
        colorAttachments: [{
            view: renderTarget.createView(),
            loadOp: 'clear',
            clearValue: { r: 0.1, g: 0.1, b: 0.1, a: 1.0 },
            storeOp: 'store'
        }]
    });
    
    passEncoder.setPipeline(pipeline);
    passEncoder.setVertexBuffer(0, vertexBuffer);
    passEncoder.setBindGroup(0, bindGroup);
    passEncoder.draw(6, 1, 0, 0);
    passEncoder.end();
    
    const commandBuffer = commandEncoder.finish();
    device.queue.submit([commandBuffer]);
    
    console.log("Command Buffer submitted");
}

main().catch(console.error);
