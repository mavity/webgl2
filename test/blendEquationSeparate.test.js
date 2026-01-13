import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('blendEquationSeparate sets state', async () => {
  const gl = await webGL2();
  try {
    gl.blendEquationSeparate(gl.FUNC_ADD, gl.FUNC_SUBTRACT);
    assert.equal(gl.getError(), gl.NO_ERROR);
  } finally { gl.destroy(); }
});
