import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('shaderSource does not throw for valid shader', async () => {
  const gl = await webGL2();
  try {
    const shader = gl.createShader(0x8B31);
    gl.shaderSource(shader, '#version 300 es\nvoid main() {}');
  } finally { gl.destroy(); }
});
