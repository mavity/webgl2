import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('framebufferRenderbuffer works', async () => {
  const gl = await webGL2();
  const fb = gl.createFramebuffer();
  const rb = gl.createRenderbuffer();
  gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
  gl.bindRenderbuffer(gl.RENDERBUFFER, rb);
  gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA4, 256, 256);
  gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rb);
  assert.equal(gl.getError(), gl.NO_ERROR);
  gl.destroy();
});
