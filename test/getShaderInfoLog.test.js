import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('getShaderInfoLog does not throw', async () => {
  const gl = await webGL2();
  try {
    const shader = gl.createShader(gl.VERTEX_SHADER);
    const log = gl.getShaderInfoLog(shader);
    assert.strictEqual(typeof log, 'string');
  } finally { gl.destroy(); }
});
