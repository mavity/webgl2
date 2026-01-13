import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('getActiveUniform works', async () => {
  const gl = await webGL2();
  try {
    const vsSource = `#version 300 es
        in vec4 position;
        uniform vec3 uColor;
        uniform mat4 uMatrix;
        void main() {
            gl_Position = uMatrix * position + vec4(uColor, 1.0);
        }`;
    const fsSource = `#version 300 es
        precision mediump float;
        out vec4 outColor;
        void main() {
            outColor = vec4(1.0);
        }`;

    const program = gl.createProgram();
    const vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, vsSource);
    gl.compileShader(vs);

    if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) {
      throw new Error(`VS failed: ${gl.getShaderInfoLog(vs)}`);
    }

    gl.attachShader(program, vs);

    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, fsSource);
    gl.compileShader(fs);

    if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) {
      throw new Error(`FS failed: ${gl.getShaderInfoLog(fs)}`);
    }

    gl.attachShader(program, fs);

    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      throw new Error(`Link failed: ${gl.getProgramInfoLog(program)}`);
    }

    const info1 = gl.getActiveUniform(program, 0);
    const info2 = gl.getActiveUniform(program, 1);

    const uniforms = [info1, info2].filter(x => x);
    assert.equal(uniforms.length, 2);

    const uColor = uniforms.find(u => u.name === 'uColor');
    const uMatrix = uniforms.find(u => u.name === 'uMatrix');

    assert.ok(uColor, 'uColor not found');
    assert.equal(uColor.size, 1);
    assert.equal(uColor.type, gl.FLOAT_VEC3);

    assert.ok(uMatrix, 'uMatrix not found');
    assert.equal(uMatrix.size, 1);
    assert.equal(uMatrix.type, gl.FLOAT_MAT4);

    assert.equal(gl.getActiveUniform(program, 99), null);

  } finally {
    gl.destroy();
  }
});
