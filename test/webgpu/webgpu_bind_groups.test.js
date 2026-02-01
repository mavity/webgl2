import test from 'node:test';
import assert from 'node:assert';
import { webGPU, GPUBufferUsage, GPUTextureUsage, GPUMapMode } from '../../index.js';

test('WebGPU Bind Groups and Textures', async (t) => {
  const gpu = await webGPU();
  const adapter = await gpu.requestAdapter();
  const device = await adapter.requestDevice();

  await t.test('Uniform Buffer Bind Group', async () => {
    const shaderCode = `
            struct Uniforms {
                color: vec4<f32>,
            };
            @group(0) @binding(0) var<uniform> u_data: Uniforms;

            @vertex
            fn vs_main(@location(0) pos: vec2<f32>) -> @builtin(position) vec4<f32> {
                return vec4<f32>(pos, 0.0, 1.0);
            }
            @fragment
            fn fs_main() -> @location(0) vec4<f32> {
                return u_data.color;
            }
        `;
    const module = device.createShaderModule({ code: shaderCode });

    const pipeline = device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module,
        entryPoint: 'vs_main',
        buffers: [{
          arrayStride: 8,
          attributes: [{
            format: 'float32x2',
            offset: 0,
            shaderLocation: 0
          }]
        }]
      },
      fragment: {
        module,
        entryPoint: 'fs_main',
        targets: [{ format: 'rgba8unorm' }]
      }
    });

    const uniformBuffer = device.createBuffer({
      size: 16,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });
    device.queue.writeBuffer(uniformBuffer, 0, new Float32Array([0.0, 1.0, 0.0, 1.0]));

    const bindGroup = device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [{
        binding: 0,
        resource: { buffer: uniformBuffer }
      }]
    });

    const vertexData = new Float32Array([
      -1, -1,
      1, -1,
      -1, 1,
      -1, 1,
      1, -1,
      1, 1,
    ]);
    const vertexBuffer = device.createBuffer({
      size: vertexData.byteLength,
      usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    });
    device.queue.writeBuffer(vertexBuffer, 0, vertexData);

    const texture = device.createTexture({
      size: [1, 1],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
    });

    const encoder = device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [{
        view: texture.createView(),
        clearValue: { r: 0, g: 0, b: 0, a: 1 },
        loadOp: 'clear',
        storeOp: 'store'
      }]
    });
    pass.setPipeline(pipeline);
    pass.setVertexBuffer(0, vertexBuffer);
    pass.setBindGroup(0, bindGroup);
    pass.draw(6);
    pass.end();
    device.queue.submit([encoder.finish()]);

    const readBuffer = device.createBuffer({
      size: 4,
      usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
    });
    const copyEncoder = device.createCommandEncoder();
    copyEncoder.copyTextureToBuffer(
      { texture },
      { buffer: readBuffer, bytesPerRow: 256 },
      [1, 1]
    );
    device.queue.submit([copyEncoder.finish()]);

    await readBuffer.mapAsync(GPUMapMode.READ);
    const result = new Uint8Array(readBuffer.getMappedRange());
    assert.deepStrictEqual(Array.from(result), [0, 255, 0, 255], 'Result should be green from uniform');
    readBuffer.unmap();
  });

  await t.test('Texture Sampling Bind Group', async () => {
    const shaderCode = `
            @group(0) @binding(0) var t_tex: texture_2d<f32>;
            @group(0) @binding(1) var s_tex: sampler;

            @vertex
            fn vs_main(@location(0) pos: vec2<f32>) -> @builtin(position) vec4<f32> {
                return vec4<f32>(pos, 0.0, 1.0);
            }
            @fragment
            fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
                return textureSample(t_tex, s_tex, vec2<f32>(0.5, 0.5));
            }
        `;
    const module = device.createShaderModule({ code: shaderCode });

    const pipeline = device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module,
        entryPoint: 'vs_main',
        buffers: [{
          arrayStride: 8,
          attributes: [{
            format: 'float32x2',
            offset: 0,
            shaderLocation: 0
          }]
        }]
      },
      fragment: {
        module,
        entryPoint: 'fs_main',
        targets: [{ format: 'rgba8unorm' }]
      }
    });

    const srcTexture = device.createTexture({
      size: [2, 2],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
    });
    const textureData = new Uint8Array([
      255, 0, 0, 255, 0, 255, 0, 255,
      0, 0, 255, 255, 255, 255, 0, 255,
    ]);
    device.queue.writeTexture(
      { texture: srcTexture },
      textureData,
      { bytesPerRow: 8 },
      [2, 2]
    );

    const sampler = device.createSampler({
      magFilter: 'linear',
      minFilter: 'linear',
    });

    const bindGroup = device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
        { binding: 0, resource: srcTexture.createView() },
        { binding: 1, resource: sampler }
      ]
    });

    const vertexData = new Float32Array([
      -1, -1,
      1, -1,
      -1, 1,
      -1, 1,
      1, -1,
      1, 1,
    ]);
    const vertexBuffer = device.createBuffer({
      size: vertexData.byteLength,
      usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    });
    device.queue.writeBuffer(vertexBuffer, 0, vertexData);

    const targetTexture = device.createTexture({
      size: [1, 1],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
    });

    const encoder = device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [{
        view: targetTexture.createView(),
        clearValue: { r: 0, g: 0, b: 0, a: 1 },
        loadOp: 'clear',
        storeOp: 'store'
      }]
    });
    pass.setPipeline(pipeline);
    pass.setVertexBuffer(0, vertexBuffer);
    pass.setBindGroup(0, bindGroup);
    pass.draw(6);
    pass.end();
    device.queue.submit([encoder.finish()]);

    const readBuffer = device.createBuffer({
      size: 4,
      usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
    });
    const copyEncoder = device.createCommandEncoder();
    copyEncoder.copyTextureToBuffer(
      { texture: targetTexture },
      { buffer: readBuffer, bytesPerRow: 256 },
      [1, 1]
    );
    device.queue.submit([copyEncoder.finish()]);

    await readBuffer.mapAsync(GPUMapMode.READ);
    try {
      const result = new Uint8Array(readBuffer.getMappedRange());
      // Bilinear sample of 2x2 texture at center (0.5, 0.5) should be average of all 4 pixels.
      // (255,0,0) + (0,255,0) + (0,0,255) + (255,255,0) = (510, 510, 255)
      // Avg = (127.5, 127.5, 63.75) -> [127, 127, 63, 255] or [128, 128, 64, 255] depending on rounding.
      // Check for approximate values.
      assert.ok(
        (result[0] >= 127 && result[0] <= 128) &&
        (result[1] >= 127 && result[1] <= 128) &&
        (result[2] >= 63 && result[2] <= 64) &&
        (result[3] == 255),
        `R:${result[0]} should be around 127/128, G:${result[1]} should be around 127/128, B:${result[2]} should be around 63/64, Alpha:${result[3]} should be 255`);
    } finally {
      readBuffer.unmap();
    }
  });
});
