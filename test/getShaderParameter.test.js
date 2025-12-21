import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('getShaderParameter does not throw', async () => {
  const gl = await webGL2();
  try {
    const shader = gl.createShader(gl.VERTEX_SHADER);
    gl.getShaderParameter(shader, gl.COMPILE_STATUS);
  } finally { gl.destroy(); }
});
