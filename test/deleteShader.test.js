import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('deleteShader does not throw', async () => {
  const gl = await webGL2();
  try {
    const shader = gl.createShader(gl.VERTEX_SHADER);
    gl.deleteShader(shader);
  } finally { gl.destroy(); }
});
