import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('deleteVertexArray works', async () => {
  const gl = await webGL2();
  try {
    const vao = gl.createVertexArray();
    gl.deleteVertexArray(vao);
    assert.equal(gl.isVertexArray(vao), false);
  } finally {
    gl.destroy();
  }
});
