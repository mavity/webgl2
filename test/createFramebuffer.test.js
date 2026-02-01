import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('createFramebuffer returns a handle', async () => {
  const gl = await webGL2();
  try {
    const fb = gl.createFramebuffer();
    assert(fb && typeof fb === 'object' && fb._handle > 0, 'createFramebuffer should return a wrapper object');
  } finally {
    gl.destroy();
  }
});
