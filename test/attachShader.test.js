import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('attachShader does not throw', async () => {
  const gl = await webGL2();
  try {
    const program = gl.createProgram();
    const shader = gl.createShader(gl.VERTEX_SHADER);
    gl.attachShader(program, shader);
  } finally { gl.destroy(); }
});
