import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('copyTexImage2D works', async () => {
  const gl = await webGL2({ width: 64, height: 64 });
  try {
    // Create FBO
    const fbo = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);

    const colorTex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, colorTex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 64, 64, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, colorTex, 0);

    // Clear FBO to Green
    gl.clearColor(0, 1, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    // Now copy from FBO to another texture
    const targetTex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, targetTex);

    gl.copyTexImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 0, 0, 32, 32, 0);
    assert.equal(gl.getError(), gl.NO_ERROR);

    // Cleanup
    gl.deleteFramebuffer(fbo);
    gl.deleteTexture(colorTex);
    gl.deleteTexture(targetTex);
  } finally {
    gl.destroy();
  }
});
