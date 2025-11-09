import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('bindTexture accepts a target and handle', async () => {
  const gl = await webGL2();
  try {
    const h = gl.createTexture();
    assert(h && typeof h === 'object' && typeof h._handle === 'number' && h._handle > 0);
    // target 0 is fine for our thin wrapper smoke test; wrapper should be accepted
    gl.bindTexture(0, h);
  } finally {
    gl.destroy();
  }
});
