import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('deleteFramebuffer does not throw for a valid handle', async () => {
  const gl = await webGL2();
  try {
    const fb = gl.createFramebuffer();
    assert(fb && typeof fb === 'object');
    gl.deleteFramebuffer(fb);
  } finally {
    gl.destroy();
  }
});
