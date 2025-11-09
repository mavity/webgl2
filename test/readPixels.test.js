import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('readPixels reads back uploaded pixel', async () => {
  const gl = await webGL2();
  try {
    const tex = gl.createTexture();
    gl.bindTexture(0, tex);
    const pixel = new Uint8Array([100, 149, 237, 255]);
    gl.texImage2D(0, 0, 0, 1, 1, 0, 0, 0, pixel);

    const fb = gl.createFramebuffer();
    gl.bindFramebuffer(0, fb);
    gl.framebufferTexture2D(0, 0, 0, tex, 0);

    const out = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, 0, 0, out);
    assert.deepStrictEqual(Array.from(out), [100, 149, 237, 255]);
  } finally {
    gl.destroy();
  }
});
