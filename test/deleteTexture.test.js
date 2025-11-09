import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('deleteTexture does not throw for a valid handle (wrapper accepted)', async () => {
  const gl = await webGL2();
  try {
    const h = gl.createTexture();
    assert(h && typeof h === 'object' && typeof h._handle === 'number' && h._handle > 0);
    gl.deleteTexture(h);
    // wrapper should be marked deleted
    assert(h._handle === 0 || h._deleted === true);
  } finally {
    gl.destroy();
  }
});
