import { test } from 'node:test';
import assert from 'node:assert';
import { webGL2 } from '../../index.js';
import { PACK_FLOAT_GLSL, unpackFloat, canonicalize } from './-core.js';

test('Math Builtin: modf', async (t) => {
  const gl = await webGL2({ size: { width: 1, height: 1 } });

  const vs = `#version 300 es
        in vec4 position;
        void main() { gl_Position = position; }
    `;

  const fsFrac = `#version 300 es
        precision highp float;
        uniform float u_x;
        out vec4 outColor;
        void main() {
            float f;
            float r = modf(u_x, f);
            ${PACK_FLOAT_GLSL('r')}
        }
    `;

  const fsInt = `#version 300 es
        precision highp float;
        uniform float u_x;
        out vec4 outColor;
        void main() {
            float f;
            float __r = modf(u_x, f);
            ${PACK_FLOAT_GLSL('float(int(f))')}
        }
    `;

  const xs = [0.0, 0.5, 1.2, -1.5, 3.9, -0.75];
  const expected = xs.map((x) => {
    const ip = x < 0 ? Math.trunc(x) : Math.trunc(x); // trunc toward 0
    const frac = x - ip;
    return { frac, ip };
  });

  // fractional parts
  const programF = gl.createProgram();
  const vShaderF = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vShaderF, vs);
  gl.compileShader(vShaderF);
  const fShaderF = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fShaderF, fsFrac);
  gl.compileShader(fShaderF);
  if (!gl.getShaderParameter(fShaderF, gl.COMPILE_STATUS)) {
    throw new Error(gl.getShaderInfoLog(fShaderF));
  }
  gl.attachShader(programF, vShaderF);
  gl.attachShader(programF, fShaderF);
  gl.linkProgram(programF);
  if (!gl.getProgramParameter(programF, gl.LINK_STATUS)) {
    throw new Error(gl.getProgramInfoLog(programF));
  }
  gl.useProgram(programF);
  const uXLoc = gl.getUniformLocation(programF, 'u_x');

  const fracResults = [];
  for (const x of xs) {
    gl.uniform1f(uXLoc, x);
    gl.drawArrays(gl.POINTS, 0, 1);
    const pixels = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    const result = unpackFloat(pixels);
    fracResults.push(canonicalize(result));
  }
  const fracExpected = expected.map((e) => canonicalize(e.frac));

  // integer parts
  const programI = gl.createProgram();
  const vShaderI = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vShaderI, vs);
  gl.compileShader(vShaderI);
  const fShaderI = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fShaderI, fsInt);
  gl.compileShader(fShaderI);
  if (!gl.getShaderParameter(fShaderI, gl.COMPILE_STATUS)) {
    throw new Error(gl.getShaderInfoLog(fShaderI));
  }
  gl.attachShader(programI, vShaderI);
  gl.attachShader(programI, fShaderI);
  gl.linkProgram(programI);
  if (!gl.getProgramParameter(programI, gl.LINK_STATUS)) {
    throw new Error(gl.getProgramInfoLog(programI));
  }
  gl.useProgram(programI);
  const uXLocI = gl.getUniformLocation(programI, 'u_x');

  const intResults = [];
  for (const x of xs) {
    gl.uniform1f(uXLocI, x);
    gl.drawArrays(gl.POINTS, 0, 1);
    const pixels = new Uint8Array(4);
    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    const result = unpackFloat(pixels);
    intResults.push(Math.round(result));
  }
  const intExpected = expected.map((e) => e.ip);

  assert.deepStrictEqual({ fracResults, intResults }, { fracResults: fracExpected, intResults: intExpected });
});