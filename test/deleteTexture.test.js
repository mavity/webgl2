import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('deleteTexture does not throw for a valid handle', async () => {
  const gl = await webGL2();
  try {
    const h = gl.createTexture();
    assert(Number.isInteger(h) && h > 0);
    gl.deleteTexture(h);
  } finally {
    gl.destroy();
  }
});
