import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('createShader returns a shader object', async () => {
  const gl = await webGL2();
  try {
    const shader = gl.createShader(0x8B31); // VERTEX_SHADER
    assert.ok(shader, 'Shader should be created');
    assert.strictEqual(typeof shader, 'object', 'Shader should be an object');
  } finally { gl.destroy(); }
});
