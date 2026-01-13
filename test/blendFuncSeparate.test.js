import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('blendFuncSeparate sets state', async () => {
  const gl = await webGL2();
  try {
    gl.blendFuncSeparate(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA, gl.ONE, gl.ZERO);
    assert.equal(gl.getError(), gl.NO_ERROR);
  } finally { gl.destroy(); }
});
