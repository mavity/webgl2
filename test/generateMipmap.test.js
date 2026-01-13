import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('generateMipmap should not crash', async () => {
  const gl = await webGL2({ width: 64, height: 64, debug: false });
  try {
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 64, 64, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

    // Should pass
    gl.generateMipmap(gl.TEXTURE_2D);

    // Check error
    assert.equal(gl.getError(), gl.NO_ERROR);

    gl.deleteTexture(tex);
  } finally {
    gl.destroy();
  }
});
