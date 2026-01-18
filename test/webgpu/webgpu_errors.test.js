import test from 'node:test';
import assert from 'node:assert/strict';
import { webGPU } from '../../index.js';
import { GPUBufferUsage } from '../../src/webgpu_context.js';

test('WebGPU Error Handling', async (t) => {
    const gpu = await webGPU();

    // We need to get the adapter and device first
    const adapter = await gpu.requestAdapter();
    const device = await adapter.requestDevice();

    await t.test('pushErrorScope / popErrorScope Binding', async () => {
        device.pushErrorScope('validation');
        // Create invalid buffer (invalid usage combination)
        // MAP_READ is incompatible with anything except COPY_DST
        device.createBuffer({ size: 4, usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.VERTEX });

        const error = await device.popErrorScope();
        assert.ok(error, 'Should return an error');
        assert.ok(error.message.length > 0, 'Error message should not be empty');
    });

    await t.test('Uncaptured Error Event', async () => {
        let caught = null;
        device.onuncapturederror = (e) => { caught = e; };

        // Trigger an error without a scope
        device.createBuffer({ size: 4, usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.VERTEX });

        // We need to wait a bit or ensure the event loop processes it.
        // Since our implementation is synchronous (WASM), it might fire immediately.
        assert.ok(caught, 'Should catch an uncaptured error');
        assert.match(caught.error.message, /Validation/, 'Should be a validation error');
    });

    await t.test('Poisoned Object Usage', async () => {
        device.pushErrorScope('validation');
        const badBuf = device.createBuffer({ size: 4, usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.VERTEX });
        device.popErrorScope(); // Clear the creation error

        assert.ok(badBuf, 'Should return a buffer object even on error');

        device.pushErrorScope('validation');
        // Try to use the poisoned buffer
        // In our implementation, if creation fails, it returns NULL_HANDLE (0).
        // Let's see how handled in JS.
        badBuf.destroy();

        // Or try to map it?
        // await badBuf.mapAsync(GPUMapMode.READ);

        const error = await device.popErrorScope();
        assert.ok(error, 'Should return an error when using poisoned buffer');
        assert.match(error.message, /poisoned|Invalid|Validation/, 'Should mention poisoned handle or validation error');
    });

    device.destroy();
});
