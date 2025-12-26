import test from 'node:test';
import assert from 'node:assert/strict';
import { webGPU } from '../index.js';

test('WebGPU device.destroy() does not throw', async () => {
  const gpu = await webGPU();
  const adapter = await gpu.requestAdapter();
  const device = await adapter.requestDevice();
  
  // should not throw
  device.destroy();
});

test('WebGPU device.destroy() is idempotent', async () => {
  const gpu = await webGPU();
  const adapter = await gpu.requestAdapter();
  const device = await adapter.requestDevice();
  
  device.destroy();
  // second destroy should be a no-op
  device.destroy();
});

test('WebGPU device.destroy() marks device as destroyed', async () => {
  const gpu = await webGPU();
  const adapter = await gpu.requestAdapter();
  const device = await adapter.requestDevice();
  
  assert.strictEqual(device._destroyed, false);
  device.destroy();
  assert.strictEqual(device._destroyed, true);
});

test('WebGPU multiple devices can be destroyed independently', async () => {
  const gpu = await webGPU();
  
  const adapter1 = await gpu.requestAdapter();
  const device1 = await adapter1.requestDevice();
  
  const adapter2 = await gpu.requestAdapter();
  const device2 = await adapter2.requestDevice();
  
  assert.notStrictEqual(device1.ctxHandle, device2.ctxHandle);
  
  device1.destroy();
  assert.strictEqual(device1._destroyed, true);
  assert.strictEqual(device2._destroyed, false);
  
  device2.destroy();
  assert.strictEqual(device2._destroyed, true);
});
