import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('renderbufferStorage works', async () => {
  const gl = await webGL2();
  const rb = gl.createRenderbuffer();
  gl.bindRenderbuffer(gl.RENDERBUFFER, rb);
  gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA4, 256, 256);
  assert.equal(gl.getError(), gl.NO_ERROR);
  gl.destroy();
});
