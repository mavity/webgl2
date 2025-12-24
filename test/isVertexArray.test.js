import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('isVertexArray works', async () => {
  const gl = await webGL2();
  try {
    const vao = gl.createVertexArray();
    assert.equal(gl.isVertexArray(vao), true);
    gl.deleteVertexArray(vao);
    assert.equal(gl.isVertexArray(vao), false);
    assert.equal(gl.isVertexArray(null), false);
  } finally {
    gl.destroy();
  }
});
