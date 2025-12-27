import test from 'node:test';
import assert from 'node:assert/strict';
import { webGPU, GPUBufferUsage, GPUMapMode } from '../../index.js';

test('WebGPU CopyBuffer', async () => {
    const gpu = await webGPU();
    
    const adapter = await gpu.requestAdapter();
    assert(adapter, "Should get an adapter");

    const device = await adapter.requestDevice();
    assert(device, "Should get a device");

    const srcSize = 16;
    const srcBuffer = device.createBuffer({
        size: srcSize,
        usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.MAP_WRITE,
        mappedAtCreation: true,
    });

    // Write data to source buffer
    const srcArray = srcBuffer.getMappedRange();
    for (let i = 0; i < srcSize; i++) {
        srcArray[i] = i + 1;
    }
    srcBuffer.unmap();

    const dstBuffer = device.createBuffer({
        size: srcSize,
        usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
    });

    const encoder = device.createCommandEncoder();
    encoder.copyBufferToBuffer(srcBuffer, 0, dstBuffer, 0, srcSize);
    const commandBuffer = encoder.finish();

    device.queue.submit([commandBuffer]);

    await dstBuffer.mapAsync(GPUMapMode.READ);
    const dstArray = dstBuffer.getMappedRange();

    const expected = new Uint8Array(srcSize);
    for (let i = 0; i < srcSize; i++) {
        expected[i] = i + 1;
    }

    assert.deepStrictEqual(new Uint8Array(dstArray), expected, "Destination buffer should match source buffer");

    dstBuffer.unmap();
});
