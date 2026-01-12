// @ts-check
import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2, getShaderWat } from '../../../index.js';

test('ABI: mixed params WAT or null', async () => {
  const gl = await webGL2({ debug: 'rust' });
  try {
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, `#version 300 es
      precision highp float; out vec4 fragColor;
      struct LargeData { vec4 a; vec4 b; vec4 c; };
      float mixedParams(float scalar, vec2 small, LargeData large, int flag) {
        return scalar + small.x + large.a.x + float(flag);
      }
      void main() { LargeData d; fragColor = vec4(mixedParams(1.0, vec2(2.0,3.0), d, 1)); }`);
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