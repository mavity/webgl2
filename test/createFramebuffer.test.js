import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('createFramebuffer returns a handle', async () => {
  const gl = await webGL2();
  try {
    const fb = gl.createFramebuffer();
    assert(Number.isInteger(fb) && fb > 0, 'createFramebuffer should return non-zero integer handle');
  } finally {
    gl.destroy();
  }
});
