import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('texImage3D basics', async () => {
  const gl = await webGL2();
  try {
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_3D, tex);
    gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA8, 2, 2, 2, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(2 * 2 * 2 * 4));
    assert.equal(gl.getError(), 0);
  } finally {
    gl.destroy();
  }
});

test('texImage3D texture 2d array', async () => {
  const gl = await webGL2();
  try {
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D_ARRAY, tex);
    gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGBA8, 4, 4, 3, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(4 * 4 * 3 * 4));
    assert.equal(gl.getError(), 0);
  } finally {
    gl.destroy();
  }
});

test('texImage3D validation', async () => {
  const gl = await webGL2();
  try {
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_3D, tex);
    // invalid target
    assert.throws(() => gl.texImage3D(gl.TEXTURE_2D, 0, gl.RGBA8, 1, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null));
    // size mismatch (too small buffer)
    assert.throws(() => gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA8, 2, 2, 2, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(1)));
  } finally {
    gl.destroy();
  }
});
