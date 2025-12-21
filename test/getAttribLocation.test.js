import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('getAttribLocation does not throw', async () => {
  const gl = await webGL2();
  try {
    const program = gl.createProgram();
    gl.getAttribLocation(program, 'a_position');
  } finally { gl.destroy(); }
});
