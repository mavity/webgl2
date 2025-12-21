import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('createProgram returns a program object', async () => {
  const gl = await webGL2();
  try {
    const program = gl.createProgram();
    assert.ok(program, 'Program should be created');
  } finally { gl.destroy(); }
});
