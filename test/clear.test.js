import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('clear fills the bound framebuffer with clear color', async () => {
  const gl = await webGL2();
  try { 
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array([0, 0, 0, 0]));

    const fb = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);

    gl.clearColor(0.5, 0.5, 0.5, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);

    const out = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, out);
    
    // 0.5 * 255 = 127.5 -> 128 (Round to nearest)
    assert.strictEqual(out[0], 128);
    assert.strictEqual(out[1], 128);
    assert.strictEqual(out[2], 128);
    assert.strictEqual(out[3], 255);
  } finally { gl.destroy(); }
});
