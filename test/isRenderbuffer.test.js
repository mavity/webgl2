import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('isRenderbuffer works', async () => {
  const gl = await webGL2();
  try {
    const rb = gl.createRenderbuffer();
    assert.equal(gl.isRenderbuffer(rb), true);
    gl.deleteRenderbuffer(rb);
    assert.equal(gl.isRenderbuffer(rb), false);
    assert.equal(gl.isRenderbuffer(null), false);
  } finally {
    gl.destroy();
  }
});
