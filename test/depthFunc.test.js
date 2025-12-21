import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('depthFunc does not throw', async () => {
  const gl = await webGL2();
  try {
    gl.depthFunc(0x0203); // GL_LESS
  } finally {
    gl.destroy();
  }
});
