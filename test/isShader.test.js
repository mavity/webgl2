import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('isShader works', async () => {
  const gl = await webGL2();
  try {
    const s = gl.createShader(gl.VERTEX_SHADER);
    assert.equal(gl.isShader(s), true);
    gl.deleteShader(s);
    assert.equal(gl.isShader(s), false);
    assert.equal(gl.isShader(null), false);
  } finally {
    gl.destroy();
  }
});
