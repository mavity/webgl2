import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('blendEquation sets state', async () => {
  const gl = await webGL2();
  try {
    gl.blendEquation(gl.FUNC_ADD);
    assert.equal(gl.getError(), gl.NO_ERROR);
  } finally { gl.destroy(); }
});
