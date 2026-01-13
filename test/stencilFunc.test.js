import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('stencilFunc sets and gets state', async () => {
  const gl = await webGL2();
  try {
    // Default values
    assert.equal(gl.getParameter(gl.STENCIL_FUNC), gl.ALWAYS); 
    assert.equal(gl.getParameter(gl.STENCIL_REF), 0);
    assert.equal(gl.getParameter(gl.STENCIL_VALUE_MASK), -1); // 0xFFFFFFFF as i32
    assert.equal(gl.getParameter(gl.STENCIL_BACK_FUNC), gl.ALWAYS);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_REF), 0);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_VALUE_MASK), -1);

    // Set both front and back
    gl.stencilFunc(gl.LEQUAL, 5, 0xAA);
    
    assert.equal(gl.getParameter(gl.STENCIL_FUNC), gl.LEQUAL);
    assert.equal(gl.getParameter(gl.STENCIL_REF), 5);
    assert.equal(gl.getParameter(gl.STENCIL_VALUE_MASK), 0xAA);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_FUNC), gl.LEQUAL);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_REF), 5);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_VALUE_MASK), 0xAA);

    // Set separately using stencilFuncSeparate
    gl.stencilFuncSeparate(gl.FRONT, gl.GREATER, 2, 0x55);
    assert.equal(gl.getParameter(gl.STENCIL_FUNC), gl.GREATER);
    assert.equal(gl.getParameter(gl.STENCIL_REF), 2);
    assert.equal(gl.getParameter(gl.STENCIL_VALUE_MASK), 0x55);
    // Back should remain unchanged from previous
    assert.equal(gl.getParameter(gl.STENCIL_BACK_FUNC), gl.LEQUAL);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_REF), 5);
    assert.equal(gl.getParameter(gl.STENCIL_BACK_VALUE_MASK), 0xAA);
  } finally {
    gl.destroy();
  }
});
