import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('vertexAttribDivisor does not throw', async () => {
  const gl = await webGL2();
  try {
    gl.vertexAttribDivisor(0, 1);
  } finally {
    gl.destroy();
  }
});
