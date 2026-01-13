import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 } from '../index.js';

test('getActiveAttrib works', async () => {
  const gl = await webGL2();
  try {
    const vsSource = `#version 300 es
        in vec3 aPos;
        in vec2 aTexCoord;
        void main() {
            gl_Position = vec4(aPos, 1.0);
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

    gl.bindAttribLocation(program, 0, 'aPos');
    gl.bindAttribLocation(program, 1, 'aTexCoord');

    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      throw new Error(`Link failed: ${gl.getProgramInfoLog(program)}`);
    }

    const info1 = gl.getActiveAttrib(program, 0);
    const info2 = gl.getActiveAttrib(program, 1);

    const attribs = [info1, info2].filter(x => x);
    assert.equal(attribs.length, 2);

    const aPos = attribs.find(a => a.name === 'aPos');
    const aTexCoord = attribs.find(a => a.name === 'aTexCoord');

    assert.ok(aPos, 'aPos not found');
    assert.equal(aPos.size, 1);
    assert.equal(aPos.type, gl.FLOAT_VEC3);

    assert.ok(aTexCoord, 'aTexCoord not found');
    assert.equal(aTexCoord.size, 1);
    assert.equal(aTexCoord.type, gl.FLOAT_VEC2);

    assert.equal(gl.getActiveAttrib(program, 99), null);

  } finally {
    gl.destroy();
  }
});
