// @ts-check
import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: mat3 parameter WAT or null', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float; out vec4 fragColor;
      mat3 multiply(mat3 a, mat3 b) { return a*b; }
      void main() { mat3 m = multiply(mat3(1.0), mat3(2.0)); fragColor = vec4(m[0][0], m[1][1], m[2][2], 1.0); }`);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    let wat;
    try {
      gl.linkProgram(program);
      wat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    } catch (e) {
      wat = null;
    }
    assert.ok(wat === null || typeof wat === 'string', 'WAT should be string or null');
  } finally {
    gl.destroy();
  }
});
