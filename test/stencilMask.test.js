import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('stencilMask sets and gets state', async () => {
  const gl = await webGL2();
  try {
    // Default is all 1s
    assert.equal(gl.getParameter(gl.STENCIL_WRITEMASK), -1);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_WRITEMASK), -1);

    // Set both
    gl.stencilMask(0x0F);
    assert.equal(gl.getParameter(gl.STENCIL_WRITEMASK), 0x0F);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_WRITEMASK), 0x0F);

    // Set separately
    gl.stencilMaskSeparate(gl.BACK, 0xF0);
    assert.equal(gl.getParameter(gl.STENCIL_WRITEMASK), 0x0F);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_WRITEMASK), 0xF0);
  } finally {
    gl.destroy();
  }
});
