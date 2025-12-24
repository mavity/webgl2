import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('bindVertexArray works', async () => {
  const gl = await webGL2();
  try {
    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);
    // Should not throw
    gl.bindVertexArray(null);
    // Should not throw
  } finally {
    gl.destroy();
  }
});
