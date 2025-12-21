import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('vertexAttribPointer does not throw', async () => {
  const gl = await webGL2();
  try {
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);
  } finally { gl.destroy(); }
});
