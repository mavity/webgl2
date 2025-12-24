import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('createVertexArray returns a vertex array object', async () => {
  const gl = await webGL2();
  try {
    const vao = gl.createVertexArray();
    assert.ok(vao);
    assert.equal(typeof vao, 'object');
    assert.ok(gl.isVertexArray(vao));
  } finally {
    gl.destroy();
  }
});
