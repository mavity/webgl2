import { test } from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../index.js';
import { PACK_FLOAT_GLSL, unpackFloat, canonicalize } from './-core.js';

test('Math Builtin: frexp', { skip: true }, async (t) => {
  const gl = await webGL2({ size: { width: 1, height: 1 } });

  const vs = `#version 300 es
        in vec4 position;
        void main() { gl_Position = position; }
    `;

  const fsMant = `#version 300 es
        precision highp float;
        uniform float u_x;
        out vec4 outColor;
        void main() {
            int e;
            float m = frexp(u_x, e);
            ${PACK_FLOAT_GLSL('m')}
        }
    `;

  const fsExp = `#version 300 es
        precision highp float;
        uniform float u_x;
        out vec4 outColor;
        void main() {
            int e;
            float __m = frexp(u_x, e);
            ${PACK_FLOAT_GLSL('float(e)')}
        }
    `;

  const xs = [0.0, 0.5, 1.0, 1.5, -1.5, 3.2, 8.0];
  const expected = xs.map((x) => {
    if (x === 0.0) return { m: 0.0, e: 0 };
    const e = Math.floor(Math.log2(Math.abs(x))) + 1;
    const m = x * Math.pow(2, -e);
    return { m, e };
  });

  // Test mantissas
  const programM = gl.createProgram();
  const vShaderM = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vShaderM, vs);
  gl.compileShader(vShaderM);
  const fShaderM = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fShaderM, fsMant);
  gl.compileShader(fShaderM);
  if (!gl.getShaderParameter(fShaderM, gl.COMPILE_STATUS)) {
    throw new Error(gl.getShaderInfoLog(fShaderM));
  }
  gl.attachShader(programM, vShaderM);
  gl.attachShader(programM, fShaderM);
  gl.linkProgram(programM);
  if (!gl.getProgramParameter(programM, gl.LINK_STATUS)) {
    throw new Error(gl.getProgramInfoLog(programM));
  }
  gl.useProgram(programM);
  const uXLoc = gl.getUniformLocation(programM, 'u_x');

  const mantResults = [];
  for (const x of xs) {
    gl.uniform1f(uXLoc, x);
    gl.drawArrays(gl.POINTS, 0, 1);
    const pixels = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    const result = unpackFloat(pixels);
    mantResults.push(canonicalize(result));
  }

  const mantExpected = expected.map((e) => canonicalize(e.m));

  // Test exponents
  const programE = gl.createProgram();
  const vShaderE = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vShaderE, vs);
  gl.compileShader(vShaderE);
  const fShaderE = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fShaderE, fsExp);
  gl.compileShader(fShaderE);
  if (!gl.getShaderParameter(fShaderE, gl.COMPILE_STATUS)) {
    throw new Error(gl.getShaderInfoLog(fShaderE));
  }
  gl.attachShader(programE, vShaderE);
  gl.attachShader(programE, fShaderE);
  gl.linkProgram(programE);
  if (!gl.getProgramParameter(programE, gl.LINK_STATUS)) {
    throw new Error(gl.getProgramInfoLog(programE));
  }
  gl.useProgram(programE);
  const uXLocE = gl.getUniformLocation(programE, 'u_x');

  const expResults = [];
  for (const x of xs) {
    gl.uniform1f(uXLocE, x);
    gl.drawArrays(gl.POINTS, 0, 1);
    const pixels = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    const result = unpackFloat(pixels);
    // integer exponent was packed as a float
    expResults.push(Math.round(result));
  }

  const expExpected = expected.map((e) => e.e);

  assert.deepStrictEqual({ mantResults, expResults }, { mantResults: mantExpected, expResults: expExpected });
});
