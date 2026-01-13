import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('depthMask sets and gets state', async () => {
  const gl = await webGL2();
  try {
    // Default should be true
    assert.equal(gl.getParameter(gl.DEPTH_WRITEMASK), true);

    // Set to false
    gl.depthMask(false);
    assert.equal(gl.getParameter(gl.DEPTH_WRITEMASK), false);

    // Set to true
    gl.depthMask(true);
    assert.equal(gl.getParameter(gl.DEPTH_WRITEMASK), true);
  } finally {
    gl.destroy();
  }
});
