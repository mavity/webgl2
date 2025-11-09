import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('framebufferTexture2D attaches a texture to a framebuffer', async () => {
  const gl = await webGL2();
  try {
    const tex = gl.createTexture();
    gl.bindTexture(0, tex);
    gl.texImage2D(0, 0, 0, 1, 1, 0, 0, 0, new Uint8Array([5,6,7,8]));

    const fb = gl.createFramebuffer();
    gl.bindFramebuffer(0, fb);
    gl.framebufferTexture2D(0, 0, 0, tex, 0);
  } finally {
    gl.destroy();
  }
});
