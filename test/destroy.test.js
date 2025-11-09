import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('destroy should not throw and marks context destroyed', async () => {
  const gl = await webGL2();
  try {
    gl.destroy();
    // subsequent calls should fail with context destroyed
    assert.throws(() => gl.createTexture(), /context has been destroyed/);
  } finally {
    // ensure idempotent
    gl.destroy();
  }
});
