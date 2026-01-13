import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('stencilOp sets and gets state', async () => {
  const gl = await webGL2();
  try {
    // Default is KEEP
    assert.equal(gl.getParameter(gl.STENCIL_FAIL), gl.KEEP);
    assert.equal(gl.getParameter(gl.STENCIL_PASS_DEPTH_FAIL), gl.KEEP);
    assert.equal(gl.getParameter(gl.STENCIL_PASS_DEPTH_PASS), gl.KEEP);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_FAIL), gl.KEEP);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_PASS_DEPTH_FAIL), gl.KEEP);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_PASS_DEPTH_PASS), gl.KEEP);

    // Set both
    gl.stencilOp(gl.REPLACE, gl.INCR, gl.DECR);
    assert.equal(gl.getParameter(gl.STENCIL_FAIL), gl.REPLACE);
    assert.equal(gl.getParameter(gl.STENCIL_PASS_DEPTH_FAIL), gl.INCR);
    assert.equal(gl.getParameter(gl.STENCIL_PASS_DEPTH_PASS), gl.DECR);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_FAIL), gl.REPLACE);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_PASS_DEPTH_FAIL), gl.INCR);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_PASS_DEPTH_PASS), gl.DECR);

    // Set separately
    gl.stencilOpSeparate(gl.FRONT, gl.ZERO, gl.INVERT, gl.REPLACE);
    assert.equal(gl.getParameter(gl.STENCIL_FAIL), gl.ZERO);
    assert.equal(gl.getParameter(gl.STENCIL_PASS_DEPTH_FAIL), gl.INVERT);
    assert.equal(gl.getParameter(gl.STENCIL_PASS_DEPTH_PASS), gl.REPLACE);
    // Back should remain unchanged
    assert.equal(gl.getParameter(gl.STENCIL_BACK_FAIL), gl.REPLACE);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_PASS_DEPTH_FAIL), gl.INCR);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_PASS_DEPTH_PASS), gl.DECR);
  } finally {
    gl.destroy();
  }
});
