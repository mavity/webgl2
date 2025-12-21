import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('useProgram does not throw for valid program', async () => {
  const gl = await webGL2();
  try {
    const program = gl.createProgram();
    gl.useProgram(program);
  } finally { gl.destroy(); }
});
