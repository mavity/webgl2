import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('getProgramInfoLog does not throw', async () => {
  const gl = await webGL2();
  try {
    const program = gl.createProgram();
    const log = gl.getProgramInfoLog(program);
    assert.strictEqual(typeof log, 'string');
  } finally { gl.destroy(); }
});
