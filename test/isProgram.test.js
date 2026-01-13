import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('isProgram works', async () => {
  const gl = await webGL2();
  try {
    const p = gl.createProgram();
    assert.equal(gl.isProgram(p), true);
    gl.deleteProgram(p);
    assert.equal(gl.isProgram(p), false);
    assert.equal(gl.isProgram(null), false);
  } finally {
    gl.destroy();
  }
});
