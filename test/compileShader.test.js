import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('compileShader sets COMPILE_STATUS to true for valid shader', async () => {
  const gl = await webGL2();
  try {
    const shader = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(shader, '#version 300 es\nvoid main() { gl_Position = vec4(0); }');
    gl.compileShader(shader);
    const status = gl.getShaderParameter(shader, gl.COMPILE_STATUS);
    assert.strictEqual(status, true, 'COMPILE_STATUS should be true');
  } finally { gl.destroy(); }
});

test('compileShader sets COMPILE_STATUS to false for invalid shader', async () => {
  const gl = await webGL2();
  try {
    const shader = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(shader, '#version 300 es\ninvalid code');
    gl.compileShader(shader);
    const status = gl.getShaderParameter(shader, gl.COMPILE_STATUS);
    assert.strictEqual(status, false, 'COMPILE_STATUS should be false');
    const log = gl.getShaderInfoLog(shader);
    assert.ok(log.length > 0, 'Shader info log should not be empty');
  } finally { gl.destroy(); }
});
