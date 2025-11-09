import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('createTexture returns a handle', async () => {
  const gl = await webGL2();
  try {
    const h = gl.createTexture();
    assert(Number.isInteger(h) && h > 0, 'createTexture should return non-zero integer handle');
  } finally {
    gl.destroy();
  }
});
