import test from 'node:test';
import assert from 'node:assert/strict';
import { webGPU, GPUBufferUsage, GPUMapMode, GPUTextureUsage } from '../index.js';

test('WebGPU: Clear texture and copy to buffer', async () => {
  const gpu = await webGPU();
  const adapter = await gpu.requestAdapter();
  assert(adapter, 'Failed to request adapter');
  
  const device = await adapter.requestDevice();
  assert(device, 'Failed to request device');

  // 1. Create a texture
  const width = 64; // 64 * 4 bytes = 256 bytes (aligned)
  const height = 4;
  const texture = device.createTexture({
    size: [width, height, 1],
    format: 'rgba8unorm',
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT,
  });
  assert(texture, 'Failed to create texture');

  // 2. Create a buffer to read back data
  // bytesPerRow must be 256-byte aligned
  const bytesPerRow = width * 4;
  const bufferSize = bytesPerRow * height;
  const buffer = device.createBuffer({
    size: bufferSize,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
  });
  assert(buffer, 'Failed to create buffer');

  // 3. Create command encoder
  const encoder = device.createCommandEncoder();
  assert(encoder, 'Failed to create command encoder');

  // 4. Begin render pass to clear texture to Red
  const pass = encoder.beginRenderPass({
    colorAttachments: [{
      view: texture.createView(),
      loadOp: 'clear',
      storeOp: 'store',
      clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
    }],
  });
  pass.end();

  // 5. Copy texture to buffer
  encoder.copyTextureToBuffer(
    { texture: texture },
    { buffer: buffer, bytesPerRow: width * 4 },
    { width: width, height: height }
  );

  // 6. Submit commands
  const commandBuffer = encoder.finish();
  device.queue.submit([commandBuffer]);

  // 7. Map buffer and verify data
  await buffer.mapAsync(GPUMapMode.READ);
  const arrayBuffer = buffer.getMappedRange();
  const data = new Uint8Array(arrayBuffer);

  // Verify first pixel is Red (255, 0, 0, 255)
  assert.equal(data[0], 255, 'Red channel should be 255');
  assert.equal(data[1], 0, 'Green channel should be 0');
  assert.equal(data[2], 0, 'Blue channel should be 0');
  assert.equal(data[3], 255, 'Alpha channel should be 255');

  // Verify last pixel
  const last = data.length - 4;
  assert.equal(data[last], 255, 'Red channel should be 255');
  assert.equal(data[last+1], 0, 'Green channel should be 0');
  assert.equal(data[last+2], 0, 'Blue channel should be 0');
  assert.equal(data[last+3], 255, 'Alpha channel should be 255');

  buffer.unmap();
});
