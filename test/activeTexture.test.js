import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('activeTexture does not throw', async () => {
  const gl = await webGL2();
  try {
    gl.activeTexture(0x84C0); // GL_TEXTURE0
  } finally {
    gl.destroy();
  }
});
