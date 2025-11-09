import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('createTexture returns a WebGLTexture wrapper', async () => {
  const gl = await webGL2();
  try {
    const h = gl.createTexture();
    assert(h && typeof h === 'object', 'createTexture should return an object wrapper');
    assert(typeof h._handle === 'number' && h._handle > 0, 'wrapper should contain a positive handle');
  } finally {
    gl.destroy();
  }
});
