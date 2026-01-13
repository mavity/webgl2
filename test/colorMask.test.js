import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('colorMask sets and gets state', async () => {
  const gl = await webGL2();
  try {
    // Default should be all true
    assert.deepEqual(gl.getParameter(gl.COLOR_WRITEMASK), [true, true, true, true]);

    // Set to all false
    gl.colorMask(false, false, false, false);
    assert.deepEqual(gl.getParameter(gl.COLOR_WRITEMASK), [false, false, false, false]);

    // Set mixed
    gl.colorMask(true, false, true, false);
    assert.deepEqual(gl.getParameter(gl.COLOR_WRITEMASK), [true, false, true, false]);
  } finally {
    gl.destroy();
  }
});
