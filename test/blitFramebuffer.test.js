import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('blitFramebuffer works (rgba8 color)', async () => {
  const gl = await webGL2();
  try {
    const fb1 = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb1);
    const tex1 = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex1);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 2, 2, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array([
      255, 0, 0, 255,   0, 255, 0, 255,
      0, 0, 255, 255,   255, 255, 255, 255
    ]));
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex1, 0);

    const fb2 = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb2);
    const tex2 = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex2);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 2, 2, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(16));
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex2, 0);

    gl.bindFramebuffer(gl.READ_FRAMEBUFFER, fb1);
    gl.bindFramebuffer(gl.DRAW_FRAMEBUFFER, fb2);
    
    // Blit 1:1
    gl.blitFramebuffer(0, 0, 2, 2, 0, 0, 2, 2, gl.COLOR_BUFFER_BIT, gl.NEAREST);

    // Read back from fb2
    const pixels = new Uint8Array(16);
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb2);
    gl.readPixels(0, 0, 2, 2, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    assert.deepEqual(Array.from(pixels), [
      255, 0, 0, 255,   0, 255, 0, 255,
      0, 0, 255, 255,   255, 255, 255, 255
    ]);
  } finally {
    gl.destroy();
  }
});

test('blitFramebuffer works with scaling (nearest)', async () => {
  const gl = await webGL2();
  try {
    const fb1 = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb1);
    const tex1 = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex1);
    // 1x1 red
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array([255, 0, 0, 255]));
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex1, 0);

    const fb2 = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb2);
    const tex2 = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex2);
    // 2x2 empty
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 2, 2, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(16));
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex2, 0);

    gl.bindFramebuffer(gl.READ_FRAMEBUFFER, fb1);
    gl.bindFramebuffer(gl.DRAW_FRAMEBUFFER, fb2);
    
    // Blit 1x1 to 2x2
    gl.blitFramebuffer(0, 0, 1, 1, 0, 0, 2, 2, gl.COLOR_BUFFER_BIT, gl.NEAREST);

    // Read back from fb2
    gl.bindFramebuffer(gl.READ_FRAMEBUFFER, fb2);
    const pixels = new Uint8Array(16);
    gl.readPixels(0, 0, 2, 2, gl.RGBA, gl.UNSIGNED_BYTE, pixels);

    // Should be all red
    assert.deepEqual(Array.from(pixels), [
      255, 0, 0, 255,   255, 0, 0, 255,
      255, 0, 0, 255,   255, 0, 0, 255
    ]);
  } finally {
    gl.destroy();
  }
});
