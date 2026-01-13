import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('isTexture works', async () => {
  const gl = await webGL2();
  try {
    const tex = gl.createTexture();
    assert.equal(gl.isTexture(tex), true);
    gl.deleteTexture(tex);
    assert.equal(gl.isTexture(tex), false);
    assert.equal(gl.isTexture(null), false);
    assert.equal(gl.isTexture({}), false);
  } finally {
    gl.destroy();
  }
});
