
// Environment detection
const isNode = typeof process !== 'undefined' && process.versions && process.versions.node;

// Animation state for browser
const animationState = {
    running: false,
    frameCount: 0,
    lastFpsTime: Date.now(),
    fps: 0,
    fpsElement: null,
    button: null,
    canvas: null,
    ctx: null,
    width: 640,
    height: 480,
    startTime: null
};

/** @type {GPUDevice} */
let device = null;
/** @type {GPU} */
let gpu = null;
/** @type {string} */
let presentationFormat = 'rgba8unorm';
/** @type {GPUCanvasContext} */
let context = null;
/** @type {GPURenderPipeline} */
let pipeline = null;
/** @type {GPUBuffer} */
let vertexBuffer = null;
/** @type {GPUBuffer} */
let uniformBuffer = null;
/** @type {GPUTexture} */
let texture = null;
/** @type {GPUTexture} */
let depthTexture = null;
/** @type {GPUSampler} */
let sampler = null;
/** @type {GPUBindGroup} */
let bindGroup = null;

async function initializeWebGPU() {
    if (device) return;

    let loadLocal = isNode || (
        typeof location !== 'undefined' &&
        typeof location?.hostname === 'string' &&
        (location.hostname.toString() === 'localhost' || location.hostname.toString() === '127.0.0.1')
    );

    const { webGPU, GPUBufferUsage, GPUTextureUsage, GPUShaderStage, GPUMapMode } = await import(loadLocal ? './index.js' : 'https://esm.run/webgl2');
    
    // index.js/webGPU returns the shim entry point
    gpu = await webGPU({ debug: true });
    const adapter = await gpu.requestAdapter();
    device = await adapter.requestDevice();

    const canvas = animationState.canvas || { width: 640, height: 480, getContext: () => null };
    const ctx = canvas.getContext ? (canvas.getContext('webgpu') || canvas.getContext('2d')) : null;
    presentationFormat = gpu.getPreferredCanvasFormat();
    context = ctx || { configure: () => {}, getCurrentTexture: () => null, present: () => {} };
    
    if (context.configure) {
        context.configure({
            device,
            format: presentationFormat,
            usage: GPUTextureUsage.RENDER_ATTACHMENT
        });
    }

    // Shaders (WGSL)
    const shaderModule = device.createShaderModule({
        code: `
        struct Uniforms {
            mvp: mat4x4<f32>,
        };
        @group(0) @binding(0) var<uniform> uniforms: Uniforms;
        @group(0) @binding(1) var t_diffuse: texture_2d<f32>;
        @group(0) @binding(2) var s_diffuse: sampler;

        struct VertexOutput {
            @builtin(position) clip_position: vec4<f32>,
            @location(0) uv: vec2<f32>,
        };

        @vertex
        fn vs_main(
            @location(0) position: vec3<f32>,
            @location(1) uv: vec2<f32>,
        ) -> VertexOutput {
            var out: VertexOutput;
            out.uv = uv;
            out.clip_position = uniforms.mvp * vec4<f32>(position, 1.0);
            return out;
        }

        @fragment
        fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
            return textureSample(t_diffuse, s_diffuse, in.uv);
        }
        `
    });

    // Bind Group Layout
    const bindGroupLayout = device.createBindGroupLayout({
        entries: [
            {
                binding: 0,
                visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT,
                buffer: {}
            },
            {
                binding: 1,
                visibility: GPUShaderStage.FRAGMENT,
                texture: {}
            },
            {
                binding: 2,
                visibility: GPUShaderStage.FRAGMENT,
                sampler: {}
            }
        ]
    });

    const pipelineLayout = device.createPipelineLayout({
        bindGroupLayouts: [bindGroupLayout]
    });

    // Render Pipeline
    pipeline = device.createRenderPipeline({
        layout: pipelineLayout,
        vertex: {
            module: shaderModule,
            entryPoint: 'vs_main',
            buffers: [{
                arrayStride: 20, // 3 floats pos + 2 floats uv
                attributes: [
                    { shaderLocation: 0, offset: 0, format: 'float32x3' },
                    { shaderLocation: 1, offset: 12, format: 'float32x2' }
                ]
            }]
        },
        fragment: {
            module: shaderModule,
            entryPoint: 'fs_main',
            targets: [{ format: presentationFormat }]
        },
        primitive: {
            topology: 'triangle-list'
        },
        depthStencil: {
            format: 'depth24plus',
            depthWriteEnabled: true,
            depthCompare: 'less',
        }
    });

    // Cube data
    const vertices = new Float32Array([
        // Front face
        -0.5, -0.5, 0.5, 0.0, 1.0,
        0.5, -0.5, 0.5, 1.0, 1.0,
        0.5, 0.5, 0.5, 1.0, 0.0,
        -0.5, -0.5, 0.5, 0.0, 1.0,
        0.5, 0.5, 0.5, 1.0, 0.0,
        -0.5, 0.5, 0.5, 0.0, 0.0,

        // Back face
        -0.5, -0.5, -0.5, 1.0, 1.0,
        -0.5, 0.5, -0.5, 1.0, 0.0,
        0.5, 0.5, -0.5, 0.0, 0.0,
        -0.5, -0.5, -0.5, 1.0, 1.0,
        0.5, 0.5, -0.5, 0.0, 0.0,
        0.5, -0.5, -0.5, 0.0, 1.0,

        // Top face
        -0.5, 0.5, -0.5, 0.0, 0.0,
        -0.5, 0.5, 0.5, 0.0, 1.0,
        0.5, 0.5, 0.5, 1.0, 1.0,
        -0.5, 0.5, -0.5, 0.0, 0.0,
        0.5, 0.5, 0.5, 1.0, 1.0,
        0.5, 0.5, -0.5, 1.0, 0.0,

        // Bottom face
        -0.5, -0.5, -0.5, 0.0, 1.0,
        0.5, -0.5, -0.5, 1.0, 1.0,
        0.5, -0.5, 0.5, 1.0, 0.0,
        -0.5, -0.5, -0.5, 0.0, 1.0,
        0.5, -0.5, 0.5, 1.0, 0.0,
        -0.5, -0.5, 0.5, 0.0, 0.0,

        // Right face
        0.5, -0.5, -0.5, 1.0, 1.0,
        0.5, 0.5, -0.5, 1.0, 0.0,
        0.5, 0.5, 0.5, 0.0, 0.0,
        0.5, -0.5, -0.5, 1.0, 1.0,
        0.5, 0.5, 0.5, 0.0, 0.0,
        0.5, -0.5, 0.5, 0.0, 1.0,

        // Left face
        -0.5, -0.5, -0.5, 0.0, 1.0,
        -0.5, -0.5, 0.5, 1.0, 1.0,
        -0.5, 0.5, 0.5, 1.0, 0.0,
        -0.5, -0.5, -0.5, 0.0, 1.0,
        -0.5, 0.5, 0.5, 1.0, 0.0,
        -0.5, 0.5, -0.5, 0.0, 0.0,
    ]);

    vertexBuffer = device.createBuffer({
        size: vertices.byteLength,
        usage: GPUBufferUsage.VERTEX,
        mappedAtCreation: true
    });
    new Float32Array(vertexBuffer.getMappedRange()).set(vertices);
    vertexBuffer.unmap();

    uniformBuffer = device.createBuffer({
        size: 64, // 4x4 matrix
        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });

    // Texture
    const texSize = 16;
    texture = device.createTexture({
        size: [texSize, texSize, 1],
        format: 'rgba8unorm',
        usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING
    });

    const texData = new Uint8Array(texSize * texSize * 4);
    for (let y = 0; y < texSize; y++) {
        for (let x = 0; x < texSize; x++) {
            const idx = (y * texSize + x) * 4;
            const isCheck = ((x >> 2) ^ (y >> 2)) & 1;
            if (isCheck) {
                texData[idx] = 255; texData[idx + 1] = 215; texData[idx + 2] = 0; texData[idx + 3] = 255; 
            } else {
                texData[idx] = 100; texData[idx + 1] = 149; texData[idx + 2] = 237; texData[idx + 3] = 255;
            }
        }
    }
    device.queue.writeTexture(
        { texture },
        texData,
        { bytesPerRow: texSize * 4, rowsPerImage: texSize },
        [texSize, texSize, 1]
    );

    sampler = device.createSampler({
        magFilter: 'nearest',
        minFilter: 'nearest',
    });

    bindGroup = device.createBindGroup({
        layout: bindGroupLayout,
        entries: [
            { binding: 0, resource: { buffer: uniformBuffer } },
            { binding: 1, resource: texture.createView() },
            { binding: 2, resource: sampler }
        ]
    });

    depthTexture = device.createTexture({
        size: [640, 480, 1],
        format: 'depth24plus',
        usage: GPUTextureUsage.RENDER_ATTACHMENT
    });
}

// Matrix math helpers
function perspective(fovy, aspect, near, far) {
    const f = 1.0 / Math.tan(fovy / 2);
    const nf = 1 / (near - far);
    return [
        f / aspect, 0, 0, 0,
        0, f, 0, 0,
        0, 0, far * nf, -1,
        0, 0, far * near * nf, 0
    ];
}

function multiply(a, b) {
    const out = new Float32Array(16);
    for (let col = 0; col < 4; col++) {
        for (let row = 0; row < 4; row++) {
            let sum = 0;
            for (let k = 0; k < 4; k++) {
                sum += a[k * 4 + row] * b[col * 4 + k];
            }
            out[col * 4 + row] = sum;
        }
    }
    return out;
}

function rotateY(m, angle) {
    const c = Math.cos(angle);
    const s = Math.sin(angle);
    const r = [
        c, 0, -s, 0,
        0, 1, 0, 0,
        s, 0, c, 0,
        0, 0, 0, 1
    ];
    return multiply(m, r);
}

function rotateX(m, angle) {
    const c = Math.cos(angle);
    const s = Math.sin(angle);
    const r = [
        1, 0, 0, 0,
        0, c, s, 0,
        0, -s, c, 0,
        0, 0, 0, 1
    ];
    return multiply(m, r);
}

function translate(m, x, y, z) {
    const r = [
        1, 0, 0, 0,
        0, 1, 0, 0,
        0, 0, 1, 0,
        x, y, z, 1
    ];
    return multiply(m, r);
}

async function renderFrame() {
    if (!device) return;

    const time = (Date.now() - (animationState.startTime || Date.now())) / 1000;
    
    // Update MVP
    let proj = perspective(Math.PI / 4, 640 / 480, 0.1, 100.0);
    let view = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, -2, 1]; // Translate [0,0,-2]
    let model = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
    model = rotateY(model, time);
    model = rotateX(model, time * 0.5);
    
    let mvp = multiply(proj, multiply(view, model));
    device.queue.writeBuffer(uniformBuffer, 0, mvp);

    const currentTexture = context.getCurrentTexture();
    if (!currentTexture) return;

    const commandEncoder = device.createCommandEncoder();
    const renderPass = commandEncoder.beginRenderPass({
        colorAttachments: [{
            view: currentTexture.createView(),
            clearValue: { r: 0.1, g: 0.1, b: 0.1, a: 1.0 },
            loadOp: 'clear',
            storeOp: 'store'
        }],
        depthStencilAttachment: {
            view: depthTexture.createView(),
            depthClearValue: 1.0,
            depthLoadOp: 'clear',
            depthStoreOp: 'store',
        },
    });

    renderPass.setPipeline(pipeline);
    renderPass.setBindGroup(0, bindGroup);
    renderPass.setVertexBuffer(0, vertexBuffer);
    renderPass.draw(36);
    renderPass.end();

    device.queue.submit([commandEncoder.finish()]);

    if (context.present) {
        context.present(currentTexture);
    }

    if (animationState.running) {
        animationState.frameCount++;
        const now = Date.now();
        if (now - animationState.lastFpsTime > 1000) {
            animationState.fps = Math.round((animationState.frameCount * 1000) / (now - animationState.lastFpsTime));
            animationState.frameCount = 0;
            animationState.lastFpsTime = now;
            if (animationState.fpsElement) {
                animationState.fpsElement.innerText = `FPS: ${animationState.fps}`;
            }
        }
        requestAnimationFrame(renderFrame);
    }
}

async function captureScreenshot() {
    await initializeWebGPU();
    
    // Set fixed time for deterministic screenshot
    animationState.startTime = Date.now() - 1500; 
    await renderFrame();

    // After renderFrame, we need to read back the context's internal data
    // context.present(currentTexture) in SoftApi triggers an async readback to canvas.
    // In Node.js, we can intercept that or manually trigger a readback.
    
    if (isNode) {
        console.log("Capturing WebGPU screenshot...");
        const fs = await import('fs');
        const { PNG } = await import('pngjs');

        // Manual readback for headless node
        const width = 640;
        const height = 480;
        const renderTexture = device.createTexture({
            size: [width, height, 1],
            format: presentationFormat,
            usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
        });

        const readBuffer = device.createBuffer({
            size: width * height * 4,
            usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
        });

        const encoder = device.createCommandEncoder();
        const pass = encoder.beginRenderPass({
            colorAttachments: [{
                view: renderTexture.createView(),
                clearValue: { r: 1.0, g: 0.0, b: 1.0, a: 1.0 }, // Magenta background
                loadOp: 'clear',
                storeOp: 'store'
            }],
            depthStencilAttachment: {
                view: depthTexture.createView(),
                depthClearValue: 1.0,
                depthLoadOp: 'clear',
                depthStoreOp: 'store',
            },
        });
        pass.setViewport(0, 0, width, height, 0, 1);
        pass.setPipeline(pipeline);
        pass.setBindGroup(0, bindGroup);
        pass.setVertexBuffer(0, vertexBuffer);
        pass.draw(36);
        pass.end();

        encoder.copyTextureToBuffer(
            { texture: renderTexture },
            { buffer: readBuffer, bytesPerRow: width * 4 },
            { width, height, depthOrArrayLayers: 1 }
        );
        device.queue.submit([encoder.finish()]);

        await readBuffer.mapAsync(GPUMapMode.READ);
        const dataArr = new Uint8Array(readBuffer.getMappedRange());
        
        // Convert BGRA to RGBA if necessary
        const pngData = new Uint8Array(width * height * 4);
        if (presentationFormat === 'bgra8unorm') {
            for (let i = 0; i < dataArr.length; i += 4) {
                pngData[i] = dataArr[i + 2];     // R
                pngData[i + 1] = dataArr[i + 1]; // G
                pngData[i + 2] = dataArr[i];     // B
                pngData[i + 3] = dataArr[i + 3]; // A
            }
        } else {
            pngData.set(dataArr);
        }

        const png = new PNG({ width, height });
        png.data.set(pngData);
        fs.writeFileSync('output-webgpu.png', PNG.sync.write(png));
        console.log("Saved output-webgpu.png");
        
        console.log("Presentation Format:", presentationFormat);
        
        let sum = 0;
        for (let i = 0; i < pngData.length; i++) sum += pngData[i];
        console.log("Sum of pixel values:", sum);

        // Console animation logic (if running in terminal)
        console.log("Terminal ASCII Preview:");
        const chars = " .:-=+*#%@";
        console.log("Lines to print:", Math.floor(height / 20));
        for (let y = 0; y < height; y += 20) {
            let line = "";
            for (let x = 0; x < width; x += 10) {
                const idx = (y * width + x) * 4;
                const r = pngData[idx];
                const g = pngData[idx+1];
                const b = pngData[idx+2];
                const brightness = (r + g + b) / 3 / 255;
                const charIdx = Math.floor(brightness * (chars.length - 1));
                line += chars[charIdx] || '?';
            }
            console.log(line);
        }
        console.log("End of Preview");
        
        readBuffer.unmap();
        process.exit(0);
    }
}

if (isNode) {
    captureScreenshot();
} else {
    window.toggleAnimation = async (btn) => {
        animationState.button = btn;
        animationState.canvas = document.getElementById('gl-canvas');
        animationState.fpsElement = document.getElementById('fps-counter');

        if (!animationState.running) {
            await initializeWebGPU();
            animationState.running = true;
            animationState.startTime = Date.now();
            btn.innerText = "Stop Animation";
            renderFrame();
        } else {
            animationState.running = false;
            btn.innerText = "Start Animation";
        }
    };
}
