import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('texImage2D uploads pixel data without throwing', async () => {
  const gl = await webGL2();
  try {
    const h = gl.createTexture();
    gl.bindTexture(0, h);
    const pixelData = new Uint8Array([1, 2, 3, 4]);
    // minimal 1x1 texture
    gl.texImage2D(0, 0, 0, 1, 1, 0, 0, 0, pixelData);
  } finally {
    gl.destroy();
  }
});
