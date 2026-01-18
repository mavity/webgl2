import test from 'node:test';
import assert from 'node:assert';
import { webGPU } from '../../index.js';

test('WebGPU Adapter Metadata', async () => {
    const gpu = await webGPU();
    const adapter = await gpu.requestAdapter();
    assert.ok(adapter, 'Adapter should be created');

    // Features
    assert.ok(adapter.features instanceof Set, 'features should be a Set');
    // Limits
    assert.ok(adapter.limits, 'limits should exist');
    assert.ok(adapter.limits.maxTextureDimension2D > 0, 'maxTextureDimension2D should be > 0');
    assert.ok(adapter.limits.maxBufferSize > 0, 'maxBufferSize should be > 0');
    
    // Preferred format
    const format = gpu.getPreferredCanvasFormat();
    assert.strictEqual(format, 'rgba8unorm', 'Preferred format should be rgba8unorm');
});
