import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('getProgramParameter does not throw', async () => {
  const gl = await webGL2();
  try {
    const program = gl.createProgram();
    gl.getProgramParameter(program, gl.LINK_STATUS);
  } finally { gl.destroy(); }
});
