import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('isFramebuffer works', async () => {
  const gl = await webGL2();
  try {
    const fb = gl.createFramebuffer();
    assert.equal(gl.isFramebuffer(fb), true);
    gl.deleteFramebuffer(fb);
    assert.equal(gl.isFramebuffer(fb), false);
    assert.equal(gl.isFramebuffer(null), false);
    assert.equal(gl.isFramebuffer({}), false);
    assert.equal(gl.isFramebuffer(99999), false);
  } finally {
    gl.destroy();
  }
});
