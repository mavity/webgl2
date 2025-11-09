import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('bindTexture accepts a target and handle', async () => {
  const gl = await webGL2();
  try {
    const h = gl.createTexture();
    assert(Number.isInteger(h) && h > 0);
    // target 0 is fine for our thin wrapper smoke test
    gl.bindTexture(0, h);
  } finally {
    gl.destroy();
  }
});
