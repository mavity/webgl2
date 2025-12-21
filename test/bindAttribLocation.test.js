import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('bindAttribLocation does not throw', async () => {
  const gl = await webGL2();
  try {
    const program = gl.createProgram();
    gl.bindAttribLocation(program, 0, 'a_position');
  } finally { gl.destroy(); }
});
