// @ts-check
import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: vec4 return WAT or null', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float;
      out vec4 fragColor;
      vec4 makeColor(float r, float g, float b) { return vec4(r,g,b,1.0); }
      void main() { fragColor = makeColor(1.0, 0.5, 0.0); }`);
    gl.compileShader(fs);

    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, `#version 300 es
      void main() { gl_Position = vec4(0); }`);
    gl.compileShader(vs);

    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    const wat = getShaderWat(gl._ctxHandle, program._handle, gl.FRAGMENT_SHADER);
    assert.ok(wat === null || typeof wat === 'string', 'WAT should be string or null');
  } finally {
    gl.destroy();
  }
});
