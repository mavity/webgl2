import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('drawElements does not throw', async () => {
  const gl = await webGL2();
  try {
    const program = gl.createProgram();
    gl.useProgram(program);
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, buffer);
    gl.drawElements(gl.TRIANGLES, 0, gl.UNSIGNED_SHORT, 0);
  } finally { gl.destroy(); }
});
