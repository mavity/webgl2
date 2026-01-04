import test from 'node:test';
import assert from 'node:assert/strict';
import { webGPU } from '../index.js';
import { GPUBufferUsage } from '../src/webgpu_context.js';

test('WebGPU Error Handling', async (t) => {
    const gpu = await webGPU();
    
    // We need to get the adapter and device first
    const adapter = await gpu.requestAdapter();
    const device = await adapter.requestDevice();

    await t.test('pushErrorScope / popErrorScope Binding', async () => {
        device.pushErrorScope('validation');
        // Create invalid buffer (size 0 is invalid if usage is not None, but let's try something definitely invalid)
        // Actually size 0 is valid in some cases.
        // Let's try to map a buffer that is not mapable.
        // Or create a buffer with invalid usage combination.
        
        // Using a very large size might trigger OOM, but we want validation.
        // Usage 0 is valid?
        
        // Let's try to create a buffer with mappedAtCreation=true but usage doesn't have MAP_READ or MAP_WRITE?
        // No, that's allowed.
        
        // Let's try to create a buffer with size not multiple of 4 if mappedAtCreation is true.
        device.createBuffer({ size: 3, usage: GPUBufferUsage.COPY_SRC, mappedAtCreation: true }); 
        
        const error = await device.popErrorScope();
        assert.ok(error, 'Should return an error');
        // assert.match(error.message, /Validation/, 'Should be a validation error');
        assert.ok(error.message.length > 0, 'Error message should not be empty');
    });

    await t.test('Uncaptured Error Event', async () => {
        let caught = null;
        device.onuncapturederror = (e) => { caught = e; };
        
        // Trigger an error without a scope
        device.createBuffer({ size: 3, usage: GPUBufferUsage.COPY_SRC, mappedAtCreation: true });
        
        // We need to wait a bit or ensure the event loop processes it.
        // Since our implementation is synchronous (WASM), it might fire immediately.
        assert.ok(caught, 'Should catch an uncaptured error');
        assert.match(caught.error.message, /aligned|Validation/, 'Should be a validation error');
    });

    await t.test('Poisoned Object Usage', async () => {
        device.pushErrorScope('validation');
        const badBuf = device.createBuffer({ size: 3, usage: GPUBufferUsage.COPY_SRC, mappedAtCreation: true });
        device.popErrorScope(); // Clear the creation error

        assert.ok(badBuf, 'Should return a buffer object even on error');

        device.pushErrorScope('validation');
        // Try to use the poisoned buffer
        // destroy() is safe to call on invalid buffer?
        badBuf.destroy();
        
        // Or try to map it?
        // await badBuf.mapAsync(GPUMapMode.READ);
        
        const error = await device.popErrorScope();
        assert.ok(error, 'Should return an error when using poisoned buffer');
        assert.match(error.message, /poisoned|Invalid|Validation/, 'Should mention poisoned handle or validation error');
    });
    
    device.destroy();
});
